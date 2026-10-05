//! The host side of Aura's MCP tools (ADR 0006): every call goes through the
//! privacy policy (allow / ask the user / deny), is logged, and returns a
//! structured error the agent can explain (OT-004).

use crate::attachments::{AttachmentService, blocks_to_text};
use crate::consent::{CONSENT_TIMEOUT, ConsentAnswer, ConsentBroker};
use crate::events::{ConsentRequest, HostEvent};
use crate::platform::Platform;
use crate::privacy::PrivacyRepo;
use aura_capture::source::{CaptureError, Target as CapTarget, read_screen_text};
use aura_capture::{CaptureOutcome, capture_with_policy};
use aura_extensions::mcp_config::{ApprovalMode, EnvValue, McpServerSpec, Transport};
use aura_mcp::tools::{
    ACTION_DONE, ACTION_LIST, ACTION_SAVE, ACTIVE_WINDOW_INFO, ATTACHMENT_READ, AUDIO_RECENT,
    CLOCK_NOW, EXCLUSION_ADD, EXCLUSION_LIST, EXTENSIONS_LIST, MCP_SERVER_SAVE, MEETING_BRIEF_SAVE,
    MEETING_GET, MEETING_SEARCH, NOTE_SAVE, NOTE_SEARCH, OPEN_WINDOWS, PROFILE_LIST, PROFILE_SAVE,
    QUICK_COMMAND_SAVE, RECIPE_LIST, RECIPE_SAVE, REMINDER_CREATE, REMINDER_DELETE, REMINDER_LIST,
    SCREEN_CAPTURE, SCREEN_RECENT, SCREEN_TEXT, SETTINGS_APPLY, SETTINGS_DESCRIBE,
    SETTINGS_PROPOSE, SETTINGS_UNDO, SKILL_SAVE,
};
use aura_mcp::tools::{MEETING_SET_PROJECT, MEETING_STATS, PROJECT_LIST, PROJECT_SAVE};
use aura_mcp::{BoxFut, CallContext, Content, ToolHandler, ToolOutput};
use aura_policy::{
    AccessRequest, Decision, DenyReason, Grants, Policy, Requester, Source, Target, decide,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock, RwLock, Weak};
use tokio::sync::broadcast;

/// Longest image side sent to the model (keeps a screenshot ≈ 1–2k tokens).
pub const MAX_IMAGE_SIDE: u32 = 1568;

/// Labelled PNG frames (`t−01:20`, bytes), oldest first.
pub type LabelledFrames = Vec<(String, Vec<u8>)>;

/// Recent screen/audio buffers. Decoding segments is OS-specific (Media
/// Foundation for H.264); the default reports the feature as unavailable.
pub trait RecentMedia: Send + Sync {
    fn screen_frames<'a>(
        &'a self,
        minutes: f64,
        max_frames: usize,
    ) -> BoxFut<'a, Result<LabelledFrames, String>>;
    fn audio_transcript<'a>(
        &'a self,
        minutes: f64,
        source: &'a str,
    ) -> BoxFut<'a, Result<String, String>>;
}

pub struct NoRecentMedia;

impl RecentMedia for NoRecentMedia {
    fn screen_frames<'a>(
        &'a self,
        _m: f64,
        _n: usize,
    ) -> BoxFut<'a, Result<LabelledFrames, String>> {
        Box::pin(async {
            Err("o buffer de tela recente ainda não está disponível nesta versão".into())
        })
    }
    fn audio_transcript<'a>(&'a self, _m: f64, _s: &'a str) -> BoxFut<'a, Result<String, String>> {
        Box::pin(async {
            Err("o buffer de áudio recente ainda não está disponível nesta versão".into())
        })
    }
}

/// What the agent may change in the user's extensions (017). The host
/// implements it; Codex asks the user before each write (`WRITE_TOOLS`).
pub trait ExtensionsAccess: Send + Sync {
    /// Skills, quick commands and MCP servers as JSON for the model.
    fn list(&self) -> BoxFut<'_, Result<Value, String>>;
    fn save_skill(
        &self,
        name: &str,
        description: &str,
        instructions: &str,
        replace: bool,
    ) -> Result<(), String>;
    fn save_quick_command(&self, name: &str, template: &str, replace: bool) -> Result<(), String>;
    fn save_mcp_server(&self, spec: McpServerSpec) -> Result<(), String>;
    /// "Configurar com IA" (022): read, validate, apply and undo settings.
    fn settings_describe(&self) -> Value;
    fn settings_propose(&self, changes: &Value) -> Value;
    fn settings_apply(&self, changes: &Value) -> Result<Value, String>;
    fn settings_undo(&self) -> Result<Value, String>;
    /// Saved meetings (023).
    fn meeting_search(
        &self,
        query: &str,
        meeting: Option<&str>,
        project: Option<&str>,
    ) -> Result<Value, String>;
    /// Projects (039).
    fn meeting_stats(&self, meeting_id: &str) -> Result<Value, String>;
    fn project_list(&self) -> Value;
    fn project_save(
        &self,
        id: Option<&str>,
        name: &str,
        instructions: &str,
    ) -> Result<Value, String>;
    fn meeting_set_project(&self, meeting_id: &str, project: Option<&str>) -> Result<(), String>;
    fn meeting_get(&self, id: Option<&str>) -> Result<Value, String>;
    fn meeting_brief_save(&self, briefing: &str) -> Result<(), String>;
    /// Privacy exclusions and app profiles (022): `open_windows`,
    /// `exclusion_list|add`, `profile_list|save`.
    fn config_tool(&self, tool: &str, args: &Value) -> Result<Value, String>;
    /// Commitments (047).
    fn action_save(
        &self,
        meeting_id: Option<&str>,
        text: &str,
        owner: &str,
        due: Option<&str>,
        minute: Option<&str>,
    ) -> Result<Value, String>;
    fn action_list(&self, status: Option<&str>, owner: Option<&str>) -> Value;
    fn action_done(&self, id: &str, done: bool) -> Result<(), String>;
    /// Recipes (025).
    fn recipe_list(&self) -> Value;
    fn recipe_save(
        &self,
        id: &str,
        name: &str,
        description: &str,
        notes_template: &str,
        help_level: &str,
        replace: bool,
    ) -> Result<(), String>;
    /// Reminders and notes (020).
    fn clock_now(&self) -> String;
    fn reminder_create(
        &self,
        text: &str,
        at: Option<&str>,
        delay_minutes: Option<i64>,
        repeat: &str,
        agent_prompt: Option<&str>,
    ) -> Result<Value, String>;
    fn reminder_list(&self) -> Value;
    fn reminder_delete(&self, id: &str) -> Result<(), String>;
    fn note_save(&self, kind: &str, text: &str) -> Result<Value, String>;
    fn note_search(&self, kind: &str, query: &str) -> Result<Value, String>;
}

/// Filled once the host exists (it owns the tools' MCP router).
pub type ExtensionsSlot = Arc<OnceLock<Weak<dyn ExtensionsAccess>>>;

/// The server the agent described, without secret values: servers that need
/// one are saved off until the user types it in Settings.
pub fn mcp_spec_from_args(args: &Value) -> Result<McpServerSpec, String> {
    let name = args["name"].as_str().unwrap_or_default().trim().to_string();
    let strings = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    };
    let secrets = strings(&args["secret_env"]);
    let bearer = args["bearer"].as_bool().unwrap_or(false);
    let transport = match args["transport"].as_str() {
        Some("stdio") => {
            let mut env: BTreeMap<String, EnvValue> = args["env"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| {
                    v.as_str().map(|value| {
                        (
                            k.clone(),
                            EnvValue::Plain {
                                value: value.to_string(),
                            },
                        )
                    })
                })
                .collect();
            for var in &secrets {
                env.insert(var.clone(), EnvValue::Secret);
            }
            Transport::Stdio {
                command: args["command"].as_str().unwrap_or_default().to_string(),
                args: strings(&args["args"]),
                env,
                cwd: None,
            }
        }
        Some("http") => Transport::Http {
            url: args["url"].as_str().unwrap_or_default().to_string(),
            bearer_secret: bearer,
            headers: BTreeMap::new(),
        },
        _ => return Err("transport deve ser \"stdio\" ou \"http\"".into()),
    };
    let approval_mode = match args["approval"].as_str() {
        Some("alwaysAsk") => ApprovalMode::AlwaysAsk,
        Some("auto") => ApprovalMode::Auto,
        _ => ApprovalMode::AskForWrites,
    };
    let spec = McpServerSpec {
        name,
        transport,
        enabled: secrets.is_empty() && !bearer,
        disabled_tools: vec![],
        approval_mode,
        startup_timeout_sec: None,
        tool_timeout_sec: None,
    };
    spec.validate().map_err(|e| e.to_string())?;
    Ok(spec)
}

pub struct HostTools {
    /// The user's extensions (017); empty until the host is built.
    pub extensions: ExtensionsSlot,
    pub platform: Platform,
    pub policy: Arc<RwLock<Policy>>,
    pub grants: Arc<RwLock<Grants>>,
    pub privacy: PrivacyRepo,
    pub consent: Arc<ConsentBroker>,
    pub events: broadcast::Sender<HostEvent>,
    pub attachments: Arc<AttachmentService>,
    pub recent: Arc<dyn RecentMedia>,
    pub ocr_language: String,
    /// Seals the access-log thumbnail of what the agent received.
    pub vault: Arc<aura_store::Vault>,
}

/// Longest side of the access-log thumbnail.
const LOG_THUMB_SIDE: u32 = 240;

fn deny_output(reason: &DenyReason) -> ToolOutput {
    let msg = match reason {
        DenyReason::Paused => "A privacidade está pausada pelo usuário.".to_string(),
        DenyReason::SourceOff => {
            "O usuário desligou esta fonte nas configurações de privacidade.".to_string()
        }
        DenyReason::Excluded { app } => {
            format!("A janela ativa ({app}) está excluída pelo usuário.")
        }
        DenyReason::AgentNever => "O usuário não permite que o agente use esta fonte.".to_string(),
        DenyReason::NotRecording => "Esta fonte não está gravando em segundo plano.".to_string(),
    };
    ToolOutput::error(reason.code(), &msg)
}

fn source_key(s: Source) -> &'static str {
    match s {
        Source::Screen | Source::Selection => "screen",
        Source::Mic => "mic",
        Source::SystemAudio => "systemAudio",
    }
}

enum Gate {
    /// Allowed; carries the access-log entry id to complete after delivery.
    Go(Grants, Option<i64>),
    Stop(ToolOutput),
}

impl HostTools {
    fn snapshot(&self) -> (Policy, Grants) {
        (
            self.policy.read().unwrap().clone(),
            self.grants.read().unwrap().clone(),
        )
    }

    /// Decides, asks the user when needed, logs. On `Go` the returned grants
    /// include a one-off grant so `capture_with_policy` allows the capture.
    async fn gate(
        &self,
        tool: &str,
        ctx: &CallContext,
        source: Source,
        target: Target,
        reason: &str,
    ) -> Gate {
        let (policy, grants) = self.snapshot();
        let visible = match &target {
            Target::Monitor { area } => self.platform.inventory.visible_windows(*area),
            Target::Window { window } => vec![window.clone()],
            Target::Range => vec![],
        };
        let app = match &target {
            Target::Window { window } => Some(window.process.clone()),
            _ => self.platform.foreground.current().map(|a| a.process_name),
        };
        let req = AccessRequest {
            source,
            requester: Requester::Agent {
                tool: tool.into(),
                conversation: ctx.conversation.clone(),
            },
            target,
            visible_windows: visible,
            background: false,
        };
        let decision = decide(&policy, &grants, &req);
        let entry = self.privacy.log(&req, &decision).ok();
        let complete = |decision: &str, why: &str| {
            if let Some(id) = entry {
                let _ = self.privacy.complete(id, Some(decision), Some(why), None);
            }
        };
        match decision {
            Decision::Deny { reason } => Gate::Stop(deny_output(&reason)),
            Decision::Allow | Decision::AllowRedacted { .. } => Gate::Go(grants, entry),
            Decision::Ask => {
                let id = uuid::Uuid::new_v4().to_string();
                let rx = self.consent.register(&id);
                let _ = self.events.send(HostEvent::Consent(ConsentRequest {
                    id: id.clone(),
                    conversation: ctx.conversation.clone(),
                    tool: tool.into(),
                    source: source_key(source).into(),
                    reason: reason.into(),
                    app,
                }));
                let answer = tokio::time::timeout(CONSENT_TIMEOUT, rx).await;
                self.consent.forget(&id);
                let _ = self.events.send(HostEvent::ConsentResolved { id });
                match answer {
                    Ok(Ok(ConsentAnswer::Once)) => {
                        complete("allow", "consent");
                        let mut g = grants;
                        g.grant(&ctx.conversation, source);
                        Gate::Go(g, entry)
                    }
                    Ok(Ok(ConsentAnswer::Conversation)) => {
                        complete("allow", "consent");
                        let _ = self.privacy.grant(&ctx.conversation, source);
                        self.grants
                            .write()
                            .unwrap()
                            .grant(&ctx.conversation, source);
                        Gate::Go(self.grants.read().unwrap().clone(), entry)
                    }
                    Ok(Ok(ConsentAnswer::Deny)) | Ok(Err(_)) => {
                        complete("deny", "user");
                        Gate::Stop(ToolOutput::error("denied", "O usuário negou o acesso."))
                    }
                    Err(_) => {
                        complete("deny", "timeout");
                        Gate::Stop(ToolOutput::error(
                            "timeout",
                            "O usuário não respondeu ao pedido de permissão.",
                        ))
                    }
                }
            }
        }
    }

    async fn screen_capture(&self, args: Value, ctx: CallContext) -> ToolOutput {
        let reason = args["reason"].as_str().unwrap_or("").to_string();
        let want_window = args["target"].as_str() == Some("window");
        let Some(app) = self.platform.foreground.current() else {
            return ToolOutput::error("unavailable", "Não há janela ativa para capturar.");
        };
        let (cap_target, pol_target) = if want_window {
            let Some(info) = self.platform.inventory.window(app.window) else {
                return ToolOutput::error("unavailable", "A janela ativa não existe mais.");
            };
            (
                CapTarget::Window { window: app.window },
                Target::Window { window: info },
            )
        } else {
            let Some(area) = self.platform.foreground.monitor_area(&app.monitor_id) else {
                return ToolOutput::error("unavailable", "Monitor não encontrado.");
            };
            (
                CapTarget::Monitor {
                    id: app.monitor_id.clone(),
                    area,
                },
                Target::Monitor { area },
            )
        };
        let (grants, entry) = match self
            .gate(SCREEN_CAPTURE, &ctx, Source::Screen, pol_target, &reason)
            .await
        {
            Gate::Go(g, entry) => (g, entry),
            Gate::Stop(out) => return out,
        };
        let policy = self.policy.read().unwrap().clone();
        let requester = Requester::Agent {
            tool: SCREEN_CAPTURE.into(),
            conversation: ctx.conversation.clone(),
        };
        let platform = self.platform.clone();
        let res = tokio::task::spawn_blocking(move || {
            capture_with_policy(
                &policy,
                &grants,
                requester,
                &cap_target,
                platform.frames.as_ref(),
                platform.inventory.as_ref(),
            )
        })
        .await;
        match res {
            Ok(Ok(CaptureOutcome::Captured { frame, redacted })) => {
                let frame = frame.downscale(MAX_IMAGE_SIDE);
                let png = frame.to_png();
                if let Some(id) = entry
                    && let Ok(sealed) = self
                        .vault
                        .seal_bytes("access-thumb", &frame.downscale(LOG_THUMB_SIDE).to_png())
                {
                    let _ = self.privacy.complete(id, None, None, Some(&sealed));
                }
                let mut note = format!(
                    "Captura de {} ({}×{}).",
                    app.process_name, frame.width, frame.height
                );
                if redacted {
                    note.push_str(" Algumas áreas foram cobertas por privacidade.");
                }
                ToolOutput {
                    content: vec![
                        Content::Text(note),
                        Content::Image {
                            png_or_jpeg: png,
                            mime: "image/png".into(),
                        },
                    ],
                    is_error: false,
                }
            }
            Ok(Ok(CaptureOutcome::Denied { reason })) => deny_output(&reason),
            Ok(Ok(CaptureOutcome::NeedsPermission)) => {
                ToolOutput::error("denied", "Permissão necessária.")
            }
            Ok(Err(CaptureError::WindowMinimized)) => {
                ToolOutput::error("unavailable", "A janela está minimizada.")
            }
            Ok(Err(e)) => ToolOutput::error("unavailable", &e.to_string()),
            Err(e) => ToolOutput::error("unavailable", &e.to_string()),
        }
    }

    fn active_window_info(&self) -> ToolOutput {
        let policy = self.policy.read().unwrap().clone();
        if policy.paused {
            return deny_output(&DenyReason::Paused);
        }
        let Some(app) = self.platform.foreground.current() else {
            return ToolOutput::error("unavailable", "Não há janela ativa.");
        };
        if let Some(info) = self.platform.inventory.window(app.window)
            && policy
                .exclusions
                .iter()
                .any(|r| r.enabled && r.matches(&info))
        {
            return deny_output(&DenyReason::Excluded { app: info.process });
        }
        let url = self.platform.foreground.url_of(app.window);
        ToolOutput::text(
            json!({"app": app.process_name, "title": app.title, "url": url}).to_string(),
        )
    }

    async fn screen_text(&self, args: Value, ctx: CallContext) -> ToolOutput {
        let mode = args["source"].as_str().unwrap_or("auto").to_string();
        let max = args["max_chars"]
            .as_u64()
            .unwrap_or(8000)
            .clamp(100, 20_000) as usize;
        let Some(app) = self.platform.foreground.current() else {
            return ToolOutput::error("unavailable", "Não há janela ativa.");
        };
        let Some(info) = self.platform.inventory.window(app.window) else {
            return ToolOutput::error("unavailable", "A janela ativa não existe mais.");
        };
        if let Gate::Stop(out) = self
            .gate(
                SCREEN_TEXT,
                &ctx,
                Source::Screen,
                Target::Window { window: info },
                "ler o texto da janela",
            )
            .await
        {
            return out;
        }
        let platform = self.platform.clone();
        let lang = self.ocr_language.clone();
        let res = tokio::task::spawn_blocking(move || {
            read_screen_text(
                platform.screen_text.as_ref(),
                platform.frames.as_ref(),
                app.window,
                &mode,
                max,
                &lang,
            )
        })
        .await;
        match res {
            Ok(Ok(r)) if r.text.trim().is_empty() => {
                ToolOutput::error("unavailable", "Nenhum texto encontrado na janela.")
            }
            Ok(Ok(r)) => ToolOutput::text(format!("[{}] {}\n\n{}", r.origin, app.title, r.text)),
            Ok(Err(e)) => ToolOutput::error("unavailable", &e.to_string()),
            Err(e) => ToolOutput::error("unavailable", &e.to_string()),
        }
    }

    async fn screen_recent(&self, args: Value, ctx: CallContext) -> ToolOutput {
        let minutes = args["minutes"].as_f64().unwrap_or(1.0).clamp(0.25, 30.0);
        let max = args["max_frames"].as_u64().unwrap_or(6).clamp(1, 8) as usize;
        let reason = args["reason"].as_str().unwrap_or("").to_string();
        if let Gate::Stop(out) = self
            .gate(SCREEN_RECENT, &ctx, Source::Screen, Target::Range, &reason)
            .await
        {
            return out;
        }
        match self.recent.screen_frames(minutes, max).await {
            Ok(frames) if frames.is_empty() => {
                ToolOutput::error("unavailable", "Nada gravado nesse período.")
            }
            Ok(frames) => {
                let mut content = Vec::new();
                for (label, png) in frames {
                    content.push(Content::Text(label));
                    content.push(Content::Image {
                        png_or_jpeg: png,
                        mime: "image/png".into(),
                    });
                }
                ToolOutput {
                    content,
                    is_error: false,
                }
            }
            Err(e) => ToolOutput::error("unavailable", &e),
        }
    }

    async fn audio_recent(&self, args: Value, ctx: CallContext) -> ToolOutput {
        let minutes = args["minutes"].as_f64().unwrap_or(1.0).clamp(0.25, 30.0);
        let which = args["source"].as_str().unwrap_or("both").to_string();
        let reason = args["reason"].as_str().unwrap_or("").to_string();
        let sources: &[Source] = match which.as_str() {
            "mic" => &[Source::Mic],
            "system" => &[Source::SystemAudio],
            _ => &[Source::Mic, Source::SystemAudio],
        };
        for s in sources {
            if let Gate::Stop(out) = self
                .gate(AUDIO_RECENT, &ctx, *s, Target::Range, &reason)
                .await
            {
                return out;
            }
        }
        match self.recent.audio_transcript(minutes, &which).await {
            Ok(t) if t.trim().is_empty() => {
                ToolOutput::error("unavailable", "Nenhuma fala nesse período.")
            }
            Ok(t) => ToolOutput::text(t),
            Err(e) => ToolOutput::error("unavailable", &e),
        }
    }

    async fn attachment_read(&self, args: Value, ctx: CallContext) -> ToolOutput {
        let Some(id) = args["attachment_id"].as_str().map(str::to_string) else {
            return ToolOutput::error("invalid", "attachment_id é obrigatório");
        };
        let selector: aura_ingest::Selector =
            serde_json::from_value(args["selector"].clone()).unwrap_or_default();
        let atts = self.attachments.clone();
        let conv = ctx.conversation.clone();
        match tokio::task::spawn_blocking(move || atts.read(&conv, &id, &selector)).await {
            Ok(Ok(blocks)) if blocks.is_empty() => {
                ToolOutput::error("unavailable", "A seleção não tem conteúdo.")
            }
            Ok(Ok(blocks)) => ToolOutput::text(blocks_to_text(&blocks)),
            Ok(Err(aura_ingest::IngestError::NotFound(m))) => ToolOutput::error("not_found", &m),
            Ok(Err(e)) => ToolOutput::error("unavailable", &e.to_string()),
            Err(e) => ToolOutput::error("unavailable", &e.to_string()),
        }
    }
}

impl HostTools {
    fn extensions(&self) -> Result<Arc<dyn ExtensionsAccess>, ToolOutput> {
        self.extensions
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| ToolOutput::error("unavailable", "Extensões indisponíveis agora."))
    }

    async fn extensions_tool(&self, tool: &str, args: Value) -> ToolOutput {
        let ext = match self.extensions() {
            Ok(e) => e,
            Err(out) => return out,
        };
        let text = |k: &str| args[k].as_str().unwrap_or_default().to_string();
        let replace = args["replace"].as_bool().unwrap_or(false);
        let invalid = |m: String| ToolOutput::error("invalid", &m);
        match tool {
            EXTENSIONS_LIST => match ext.list().await {
                Ok(v) => ToolOutput::text(v.to_string()),
                Err(e) => ToolOutput::error("unavailable", &e),
            },
            SKILL_SAVE => {
                let name = text("name");
                match ext.save_skill(&name, &text("description"), &text("instructions"), replace) {
                    Ok(()) => ToolOutput::text(format!(
                        "Skill \"{name}\" salva. Vale para as próximas conversas; o usuário pode revisá-la em Configurações › Extensões."
                    )),
                    Err(e) => invalid(e),
                }
            }
            QUICK_COMMAND_SAVE => {
                let name = text("name");
                match ext.save_quick_command(&name, &text("template"), replace) {
                    Ok(()) => ToolOutput::text(format!(
                        "Comando rápido /{name} salvo. O usuário já pode digitar /{name} na barra do Aura."
                    )),
                    Err(e) => invalid(e),
                }
            }
            MCP_SERVER_SAVE => {
                let spec = match mcp_spec_from_args(&args) {
                    Ok(s) => s,
                    Err(e) => return invalid(e),
                };
                let (name, enabled) = (spec.name.clone(), spec.enabled);
                let needs: Vec<String> =
                    spec.secret_vars().into_iter().map(|(var, _)| var).collect();
                match ext.save_mcp_server(spec) {
                    Ok(()) if enabled => ToolOutput::text(format!(
                        "Servidor MCP \"{name}\" salvo e ligado. Ele vale nas próximas conversas (não nesta)."
                    )),
                    Ok(()) => ToolOutput::text(format!(
                        "Servidor MCP \"{name}\" salvo DESLIGADO: falta o segredo ({}). Peça ao usuário para abrir Configurações › Extensões, editar \"{name}\", informar o segredo e ligar o servidor. Nunca peça o segredo na conversa.",
                        needs.join(", ")
                    )),
                    Err(e) => invalid(e),
                }
            }
            MEETING_STATS => match ext.meeting_stats(&text("meeting_id")) {
                Ok(v) => ToolOutput::text(v.to_string()),
                Err(e) => invalid(e),
            },
            PROJECT_LIST => ToolOutput::text(ext.project_list().to_string()),
            PROJECT_SAVE => match ext.project_save(
                args["id"].as_str(),
                &text("name"),
                args["instructions"].as_str().unwrap_or_default(),
            ) {
                Ok(v) => ToolOutput::text(format!("Projeto salvo: {v}")),
                Err(e) => invalid(e),
            },
            MEETING_SET_PROJECT => {
                match ext.meeting_set_project(&text("meeting_id"), args["project_id"].as_str()) {
                    Ok(()) => ToolOutput::text("Reunião atualizada."),
                    Err(e) => invalid(e),
                }
            }
            MEETING_SEARCH => match ext.meeting_search(
                &text("query"),
                args["meeting_id"].as_str(),
                args["project_id"].as_str(),
            ) {
                Ok(v) => ToolOutput::text(v.to_string()),
                Err(e) => invalid(e),
            },
            OPEN_WINDOWS | EXCLUSION_LIST | EXCLUSION_ADD | PROFILE_LIST | PROFILE_SAVE => {
                match ext.config_tool(tool, &args) {
                    Ok(v) => ToolOutput::text(v.to_string()),
                    Err(e) => invalid(e),
                }
            }
            ACTION_SAVE => match ext.action_save(
                args["meeting_id"].as_str(),
                &text("text"),
                args["owner"].as_str().unwrap_or("you"),
                args["due"].as_str(),
                args["minute"].as_str(),
            ) {
                Ok(v) => ToolOutput::text(format!("Compromisso salvo: {v}")),
                Err(e) => invalid(e),
            },
            ACTION_LIST => ToolOutput::text(
                ext.action_list(args["status"].as_str(), args["owner"].as_str())
                    .to_string(),
            ),
            ACTION_DONE => {
                match ext.action_done(&text("id"), args["done"].as_bool().unwrap_or(true)) {
                    Ok(()) => ToolOutput::text("Compromisso atualizado."),
                    Err(e) => invalid(e),
                }
            }
            RECIPE_LIST => ToolOutput::text(ext.recipe_list().to_string()),
            RECIPE_SAVE => match ext.recipe_save(
                &text("id"),
                &text("name"),
                &text("description"),
                &text("notes_template"),
                args["help_level"].as_str().unwrap_or("onDemand"),
                replace,
            ) {
                Ok(()) => ToolOutput::text(format!(
                    "Receita \"{}\" salva. O usuário a escolhe ao preparar uma reunião.",
                    text("name")
                )),
                Err(e) => invalid(e),
            },
            MEETING_BRIEF_SAVE => match ext.meeting_brief_save(&text("briefing")) {
                Ok(()) => ToolOutput::text(
                    "Briefing guardado. O usuário revisa e aperta Começar no painel Reunião; você não inicia a reunião.",
                ),
                Err(e) => invalid(e),
            },
            MEETING_GET => match ext.meeting_get(args["meeting_id"].as_str()) {
                Ok(v) => ToolOutput::text(v.to_string()),
                Err(e) => invalid(e),
            },
            CLOCK_NOW => ToolOutput::text(ext.clock_now()),
            REMINDER_CREATE => match ext.reminder_create(
                &text("text"),
                args["at"].as_str(),
                args["delay_minutes"].as_i64(),
                args["repeat"].as_str().unwrap_or("none"),
                args["agent_prompt"].as_str(),
            ) {
                Ok(v) => ToolOutput::text(format!("Lembrete criado: {v}")),
                Err(e) => invalid(e),
            },
            REMINDER_LIST => ToolOutput::text(ext.reminder_list().to_string()),
            REMINDER_DELETE => match ext.reminder_delete(&text("id")) {
                Ok(()) => ToolOutput::text("Lembrete apagado."),
                Err(e) => invalid(e),
            },
            NOTE_SAVE => {
                match ext.note_save(args["kind"].as_str().unwrap_or("note"), &text("text")) {
                    Ok(v) => ToolOutput::text(format!("Salvo: {v}")),
                    Err(e) => invalid(e),
                }
            }
            NOTE_SEARCH => {
                match ext.note_search(args["kind"].as_str().unwrap_or("note"), &text("query")) {
                    Ok(v) => ToolOutput::text(v.to_string()),
                    Err(e) => invalid(e),
                }
            }
            SETTINGS_DESCRIBE => ToolOutput::text(ext.settings_describe().to_string()),
            SETTINGS_PROPOSE => {
                ToolOutput::text(ext.settings_propose(&args["changes"]).to_string())
            }
            SETTINGS_APPLY => match ext.settings_apply(&args["changes"]) {
                Ok(v) => ToolOutput::text(format!(
                    "Configurações aplicadas. O usuário pode desfazer com settings_undo ou em Configurações. {v}"
                )),
                Err(e) => invalid(e),
            },
            SETTINGS_UNDO => match ext.settings_undo() {
                Ok(v) => ToolOutput::text(format!("Última mudança desfeita: {v}")),
                Err(e) => invalid(e),
            },
            other => {
                ToolOutput::error("unknown_tool", &format!("ferramenta desconhecida: {other}"))
            }
        }
    }
}

impl ToolHandler for HostTools {
    fn call<'a>(&'a self, tool: &'a str, args: Value, ctx: CallContext) -> BoxFut<'a, ToolOutput> {
        Box::pin(async move {
            match tool {
                SCREEN_CAPTURE => self.screen_capture(args, ctx).await,
                ACTIVE_WINDOW_INFO => self.active_window_info(),
                SCREEN_TEXT => self.screen_text(args, ctx).await,
                SCREEN_RECENT => self.screen_recent(args, ctx).await,
                AUDIO_RECENT => self.audio_recent(args, ctx).await,
                ATTACHMENT_READ => self.attachment_read(args, ctx).await,
                EXTENSIONS_LIST | SKILL_SAVE | QUICK_COMMAND_SAVE | MCP_SERVER_SAVE
                | SETTINGS_DESCRIBE | SETTINGS_PROPOSE | SETTINGS_APPLY | SETTINGS_UNDO
                | CLOCK_NOW | MEETING_SEARCH | MEETING_GET | MEETING_BRIEF_SAVE | RECIPE_LIST
                | RECIPE_SAVE | ACTION_SAVE | ACTION_LIST | ACTION_DONE | OPEN_WINDOWS
                | EXCLUSION_LIST | EXCLUSION_ADD | PROFILE_LIST | PROFILE_SAVE
                | REMINDER_CREATE | REMINDER_LIST | REMINDER_DELETE | NOTE_SAVE | NOTE_SEARCH => {
                    self.extensions_tool(tool, args).await
                }
                PROJECT_LIST | PROJECT_SAVE | MEETING_SET_PROJECT | MEETING_STATS => {
                    self.extensions_tool(tool, args).await
                }
                other => {
                    ToolOutput::error("unknown_tool", &format!("ferramenta desconhecida: {other}"))
                }
            }
        })
    }
}
