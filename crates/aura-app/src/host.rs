//! The composition root: builds every service once, owns the shared state and
//! exposes one method per UI command. The Tauri shell only forwards calls and
//! events; everything here is testable without a window system.

use crate::attachments::{AttachmentInfo, AttachmentService};
use crate::consent::{ConsentAnswer, ConsentBroker};
use crate::error::{HostError, HostResult};
use crate::events::{HostEvent, PrivacyState};
use crate::launcher::{AuraLauncher, LaunchState, SpawnHook};
pub use crate::memories::MemoryView;
use crate::paths::AppPaths;
use crate::platform::Platform;
use crate::privacy::{AccessLogEntry, PrivacyRepo};
use crate::tokens::AuthTokenProvider;
use crate::tools::{HostTools, RecentMedia};
use crate::voice::{AsrBackend, ModelView, Voice};
use aura_asr::catalog::Hardware;
use aura_asr::download::DiskSpace;
use aura_asr::ptt::PttState;
use aura_asr::transcriber::AsrOptions;
use aura_audio::DeviceSel;
use aura_auth::AuthService;
use aura_auth::ChatGptAccount;
use aura_auth::SiwcConfig;
use aura_capture::source::Target as CapTarget;
use aura_capture::{CaptureOutcome, capture_with_policy};
use aura_codex::approvals::Decision as ApprovalDecision;
use aura_codex::events::{AppServerState, ConversationEvent};
use aura_codex::home::BaseConfig;
use aura_codex::launcher::{InMemoryLauncher, Launcher};
use aura_codex::models::ModelInfo;
use aura_codex::modes::ConversationMode;
use aura_codex::service::{
    CodexService, HistoryPage, HistoryQuery, StartOptions, StartedConversation, TranscriptMessage,
    TurnOptions,
};
use aura_codex::supervisor::{AppServerSupervisor, SupervisorConfig};
use aura_core::context::{ChipKind, ChipPayload, ContextChip, ContextTray, TurnInput};
use aura_core::secret::Secret;
use aura_core::settings::{Settings, SettingsPatch};
use aura_extensions::import::{DetectedServer, Roots};
use aura_extensions::mcp_config::{McpServerSpec, McpServersRepo};
use aura_extensions::quick::{Expansion, QuickCommand, QuickCommandsRepo, QuickContext};
use aura_extensions::skills::{self, SkillOrigin, SkillReview, SkillSource};
use aura_gateway::registry::{
    Preset, Provider, ProviderDraft, ProviderRegistry, ProviderStatus, presets,
};
use aura_gateway::server::{GatewayHandle, Routes};
use aura_gateway::upstream::chatgpt_plan::ChatGptPlanUpstream;
use aura_policy::{
    AgentPermission, CaptureMode, ExclusionRule, Grants, Policy, Requester, Source, SourcePolicy,
};
use aura_store::Store;
use aura_store::settings_repo::SettingsRepo;
use aura_store::vault::Vault;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::{broadcast, mpsc};

pub use aura_audio::DeviceInfo as AudioDevice;
pub use aura_core::placement::{OverlayMode, SavedPlacement};

pub const CHATGPT_PLAN_ROUTE: &str = "chatgpt-plan";

/// Model of a new conversation: the picker's choice, else the app profile's
/// default, else the Settings default — each only for its own provider. The
/// Settings default lists ChatGPT-plan models; a profile's BYOK model is
/// stored as `aura-<provider>::<model>` (a bare id is a plan model).
pub fn start_model(
    provider: &str,
    requested: Option<&str>,
    settings_default: Option<&str>,
    profile_default: Option<&str>,
) -> Option<String> {
    let plan = format!("aura-{CHATGPT_PLAN_ROUTE}");
    let from_profile = profile_default.and_then(|v| match v.split_once("::") {
        Some((p, m)) if p.starts_with("aura-") => (p == provider).then_some(m),
        _ => (provider == plan).then_some(v),
    });
    if provider == plan {
        // The plan offers only Aura's catalog (017): stale choices fall back
        // to its first model, and the server never picks one on its own.
        let offered = |m: &&str| {
            aura_core::model_catalog::known_model(m).is_some_and(|k| k.in_plan && k.id == *m)
        };
        return requested
            .filter(offered)
            .or(from_profile.filter(offered))
            .or(settings_default.filter(offered))
            .or_else(|| {
                aura_core::model_catalog::KNOWN
                    .iter()
                    .find(|k| k.in_plan)
                    .map(|k| k.id)
            })
            .map(str::to_string);
    }
    requested.or(from_profile).map(str::to_string)
}

/// New skills cannot take the names of the agent's core skills (017).
fn reserved_skill_name(name: &str) -> HostResult<()> {
    if name.starts_with(crate::core_skills::RESERVED_PREFIX) {
        return Err(HostError::new(
            "skill",
            format!(
                "nomes começando com \"{}\" são reservados ao Aura",
                crate::core_skills::RESERVED_PREFIX
            ),
        ));
    }
    Ok(())
}

impl crate::tools::ExtensionsAccess for Host {
    fn list(&self) -> aura_mcp::BoxFut<'_, Result<serde_json::Value, String>> {
        Box::pin(async move {
            let skills = self.skills_catalog().await.map_err(|e| e.message)?;
            let quick = self.quick_commands().map_err(|e| e.message)?;
            let servers = self.mcp_servers().map_err(|e| e.message)?;
            Ok(serde_json::json!({
                "skills": skills.iter().map(|s| serde_json::json!({
                    "name": s.name, "description": s.description, "origin": s.origin, "enabled": s.enabled,
                })).collect::<Vec<_>>(),
                "quick_commands": quick,
                "mcp_servers": servers.iter().map(|s| {
                    let (transport, target) = match &s.transport {
                        aura_extensions::mcp_config::Transport::Stdio { command, args, .. } => {
                            ("stdio", format!("{command} {}", args.join(" ")).trim().to_string())
                        }
                        aura_extensions::mcp_config::Transport::Http { url, .. } => ("http", url.clone()),
                    };
                    serde_json::json!({
                        "name": s.name, "transport": transport, "target": target, "enabled": s.enabled,
                        "secret_env": s.secret_vars().into_iter().map(|(v, _)| v).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
            }))
        })
    }

    fn save_skill(
        &self,
        name: &str,
        description: &str,
        instructions: &str,
        replace: bool,
    ) -> Result<(), String> {
        let exists = self.paths.skills().join(name).join("SKILL.md").is_file();
        let r = if exists && replace {
            self.update_skill(name, description, instructions)
        } else {
            self.create_skill(name, description, instructions)
                .map(|_| ())
        };
        r.map_err(|e| e.message)?;
        self.extensions_changed();
        Ok(())
    }

    fn save_quick_command(&self, name: &str, template: &str, replace: bool) -> Result<(), String> {
        self.save_quick_command(name, template, replace)
            .map_err(|e| e.message)?;
        self.extensions_changed();
        Ok(())
    }

    fn config_tool(
        &self,
        tool: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        use aura_mcp::tools::{
            EXCLUSION_ADD, EXCLUSION_LIST, OPEN_WINDOWS, PROFILE_LIST, PROFILE_SAVE,
        };
        let text = |k: &str| {
            args[k]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
        };
        match tool {
            OPEN_WINDOWS => {
                let app = self.platform.foreground.current().ok_or("nenhuma janela ativa")?;
                let area = self
                    .platform
                    .foreground
                    .monitor_area(&app.monitor_id)
                    .ok_or("monitor não encontrado")?;
                let own = std::process::id();
                let windows: Vec<_> = self
                    .platform
                    .inventory
                    .visible_windows(area)
                    .into_iter()
                    .filter(|w| w.pid != own)
                    .take(30)
                    .map(|w| serde_json::json!({
                        "process": w.process,
                        "title": w.title.chars().take(60).collect::<String>(),
                    }))
                    .collect();
                Ok(serde_json::json!(windows))
            }
            EXCLUSION_LIST => Ok(serde_json::json!(
                self.policy.read().unwrap().exclusions.iter().map(|r| serde_json::json!({
                    "id": r.id, "process": r.process, "title_glob": r.title_glob,
                    "enabled": r.enabled, "builtin": r.builtin,
                })).collect::<Vec<_>>()
            )),
            EXCLUSION_ADD => {
                let (process, title_glob) = (text("process"), text("title_glob"));
                if process.is_none() && title_glob.is_none() {
                    return Err("informe process e/ou title_glob".into());
                }
                let rule = aura_policy::ExclusionRule {
                    id: String::new(),
                    process,
                    title_glob,
                    class: None,
                    enabled: true,
                    builtin: false,
                };
                self.upsert_exclusion(rule.clone()).map_err(|e| e.message)?;
                Ok(serde_json::json!({"process": rule.process, "title_glob": rule.title_glob}))
            }
            PROFILE_LIST => Ok(serde_json::json!(
                self.profiles().map_err(|e| e.message)?.iter().map(|p| serde_json::json!({
                    "id": p.id, "name": p.name, "process": p.process_pattern, "title_glob": p.title_glob,
                    "instructions": p.instructions, "attach_screen": p.attach_screen,
                    "default_mode": p.default_mode,
                })).collect::<Vec<_>>()
            )),
            PROFILE_SAVE => {
                let profile = crate::profiles::AppProfile {
                    id: text("id").unwrap_or_default(),
                    name: text("name").unwrap_or_default(),
                    process_pattern: text("process").unwrap_or_default(),
                    title_glob: text("title_glob"),
                    instructions: args["instructions"].as_str().unwrap_or_default().to_string(),
                    attach_screen: args["attach_screen"].as_bool().unwrap_or(false),
                    default_mode: text("default_mode"),
                    default_model: None,
                };
                let saved = self.save_profile(profile).map_err(|e| e.message)?;
                Ok(serde_json::json!({"id": saved.id, "name": saved.name}))
            }
            other => Err(format!("ferramenta desconhecida: {other}")),
        }
    }

    fn recipe_list(&self) -> serde_json::Value {
        serde_json::json!(self.recipes_list().unwrap_or_default().iter().map(|r| serde_json::json!({
            "id": r.id, "name": r.name, "description": r.description, "notes_template": r.notes_template,
            "help_level": r.help_level, "builtin": r.builtin,
        })).collect::<Vec<_>>())
    }

    fn recipe_save(
        &self,
        id: &str,
        name: &str,
        description: &str,
        notes_template: &str,
        help_level: &str,
        replace: bool,
    ) -> Result<(), String> {
        let recipe = crate::recipes::Recipe {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            notes_template: notes_template.into(),
            help_level: help_level.into(),
            builtin: false,
        };
        self.recipes
            .save(&recipe, replace)
            .map_err(|e| e.to_string())
    }

    fn meeting_brief_save(&self, briefing: &str) -> Result<(), String> {
        self.meeting_set_brief(briefing).map_err(|e| e.message)
    }

    fn meeting_search(
        &self,
        query: &str,
        meeting: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let hits = Host::meeting_search(self, query, meeting).map_err(|e| e.message)?;
        Ok(serde_json::json!(
            hits.iter()
                .map(|h| serde_json::json!({
                    "meeting_id": h.meeting_id, "title": h.title,
                    "minute": format!("{:02}:{:02}", h.t0 / 60_000, (h.t0 / 1000) % 60),
                    "speaker": h.speaker, "text": self.for_model(&h.text),
                }))
                .collect::<Vec<_>>()
        ))
    }

    fn meeting_get(&self, id: Option<&str>) -> Result<serde_json::Value, String> {
        let Some(id) = id else {
            let list = self.meetings_list().map_err(|e| e.message)?;
            return Ok(serde_json::json!(
                list.iter()
                    .take(20)
                    .map(|m| serde_json::json!({
                        "meeting_id": m.id, "title": m.title, "kind": m.kind, "status": m.status,
                    }))
                    .collect::<Vec<_>>()
            ));
        };
        let m = self.meetings.repo().get(id).map_err(|e| e.to_string())?;
        let lines = self.meeting_utterances(id).map_err(|e| e.message)?;
        let recipe = self
            .recipes
            .get(&m.kind)
            .ok()
            .map(|r| serde_json::json!({"name": r.name, "notes_template": r.notes_template}));
        Ok(serde_json::json!({
            "meeting_id": m.id, "title": m.title, "briefing": m.briefing, "status": m.status, "recipe": recipe,
            "transcript": lines.iter().map(|u| format!(
                "[{:02}:{:02}] {}: {}", u.t0 / 60_000, (u.t0 / 1000) % 60,
                match u.speaker.as_str() { "you" => "Você", "note" => "Nota do usuário", _ => "Eles" }, self.for_model(&u.text))).collect::<Vec<_>>(),
        }))
    }

    fn clock_now(&self) -> String {
        crate::reminders::format_local(self.clock.now(), self.clock.utc_offset_secs())
    }

    fn reminder_create(
        &self,
        text: &str,
        at: Option<&str>,
        delay_minutes: Option<i64>,
        repeat: &str,
    ) -> Result<serde_json::Value, String> {
        use crate::reminders::{Repeat, parse_time};
        let (now, offset) = (self.clock.now(), self.clock.utc_offset_secs());
        let due = match (at, delay_minutes) {
            (Some(at), _) => parse_time(at, offset).map_err(|e| e.to_string())?,
            (None, Some(m)) if m >= 1 => now + m * 60,
            _ => return Err("informe at ou delay_minutes".into()),
        };
        let r = self
            .reminders
            .create(
                text,
                due,
                Repeat::parse(repeat).map_err(|e| e.to_string())?,
                now,
            )
            .map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "id": r.id, "text": r.text, "due": crate::reminders::format_local(r.due_at, offset), "repeat": r.repeat,
        }))
    }

    fn reminder_list(&self) -> serde_json::Value {
        let offset = self.clock.utc_offset_secs();
        let items = self.reminders.list().unwrap_or_default();
        serde_json::json!(items.iter().map(|r| serde_json::json!({
            "id": r.id, "text": r.text, "due": crate::reminders::format_local(r.due_at, offset), "repeat": r.repeat,
        })).collect::<Vec<_>>())
    }

    fn reminder_delete(&self, id: &str) -> Result<(), String> {
        self.reminders.delete(id).map_err(|e| e.to_string())
    }

    fn note_save(&self, kind: &str, text: &str) -> Result<serde_json::Value, String> {
        let n = self
            .notes
            .add(kind, text, self.clock.now())
            .map_err(|e| e.to_string())?;
        Ok(serde_json::json!({"id": n.id, "kind": n.kind}))
    }

    fn note_search(&self, kind: &str, query: &str) -> Result<serde_json::Value, String> {
        let hits = self
            .notes
            .search(kind, query, 20)
            .map_err(|e| e.to_string())?;
        let offset = self.clock.utc_offset_secs();
        Ok(serde_json::json!(hits.iter().map(|n| serde_json::json!({
            "id": n.id, "text": n.text, "at": crate::reminders::format_local(n.created_at, offset),
        })).collect::<Vec<_>>()))
    }

    fn settings_describe(&self) -> serde_json::Value {
        crate::settings_assistant::describe(&self.settings())
    }

    fn settings_propose(&self, changes: &serde_json::Value) -> serde_json::Value {
        crate::settings_assistant::propose(&self.settings(), changes)
            .0
            .to_json()
    }

    fn settings_apply(&self, changes: &serde_json::Value) -> Result<serde_json::Value, String> {
        let (proposal, patch) = crate::settings_assistant::propose(&self.settings(), changes);
        let Some(patch) = patch else {
            return Err(proposal.errors.join("; "));
        };
        if proposal.changes.is_empty() {
            return Ok(proposal.to_json());
        }
        self.update_settings(patch).map_err(|e| e.message)?;
        self.settings_undo
            .lock()
            .unwrap()
            .push(crate::settings_assistant::inverse(&proposal));
        Ok(proposal.to_json())
    }

    fn settings_undo(&self) -> Result<serde_json::Value, String> {
        let Some(inverse) = self.settings_undo.lock().unwrap().pop() else {
            return Err("não há mudança da IA para desfazer".into());
        };
        let patch: aura_core::settings::SettingsPatch =
            serde_json::from_value(inverse.clone()).map_err(|e| e.to_string())?;
        self.update_settings(patch).map_err(|e| e.message)?;
        Ok(inverse)
    }

    fn save_mcp_server(&self, spec: McpServerSpec) -> Result<(), String> {
        // Keeps the secrets already in the vault when the server exists.
        self.save_mcp_server(spec, vec![], None)
            .map_err(|e| e.message)?;
        self.extensions_changed();
        Ok(())
    }
}

/// How the host runs the Codex app-server.
#[derive(Clone)]
pub enum CodexRuntime {
    /// The pinned, verified binary (production). `program` is `None` until
    /// [`Host::ensure_codex`] has downloaded and verified it.
    Binary {
        program: Option<PathBuf>,
        on_spawn: Option<SpawnHook>,
    },
    /// In-process fake app-server (tests, UI demo without an account).
    Fake,
}

pub struct HostConfig {
    pub paths: AppPaths,
    pub platform: Platform,
    pub codex: CodexRuntime,
    pub siwc: SiwcConfig,
    /// `:memory:` store (tests).
    pub in_memory_store: bool,
    pub recent: Option<Arc<dyn RecentMedia>>,
    pub asr: AsrBackend,
    pub hardware: Hardware,
    pub disk: Arc<dyn DiskSpace>,
    /// Background screen capture interval (1 fps in production).
    pub capture_interval: std::time::Duration,
}

impl HostConfig {
    /// Everything fake except the real services: tests and the Linux demo.
    pub fn demo(paths: AppPaths, platform: Platform) -> Self {
        Self {
            paths,
            platform,
            codex: CodexRuntime::Fake,
            siwc: SiwcConfig::default(),
            in_memory_store: false,
            recent: None,
            asr: AsrBackend::Fake {
                text: "texto ditado de exemplo".into(),
            },
            hardware: Hardware::default(),
            disk: Arc::new(aura_asr::download::UnknownSpace),
            capture_interval: std::time::Duration::from_secs(1),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyView {
    pub screen: SourcePolicy,
    pub mic: SourcePolicy,
    pub system_audio: SourcePolicy,
    pub paused: bool,
    pub exclusions: Vec<ExclusionRule>,
    pub retention: aura_policy::Retention,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub accounts: Vec<ChatGptAccount>,
    pub active: Option<ChatGptAccount>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub version: String,
    pub os: String,
    pub data_dir: PathBuf,
    pub gateway_port: u16,
    pub app_server: AppServerState,
    pub app_server_launches: u64,
    pub providers: usize,
    pub mcp_servers: usize,
    pub pending_consents: usize,
    pub gateway: GatewayDiag,
    /// MCP servers as the running app-server sees them (empty when stopped).
    pub mcp: Vec<McpDiag>,
    pub worker: crate::voice::WorkerDiag,
    pub capture: CaptureDiag,
    /// Active ChatGPT account, e-mail masked, never tokens.
    pub account: Option<AccountDiag>,
    pub disk: DiskDiag,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayDiag {
    pub port: u16,
    /// Accepts connections on 127.0.0.1.
    pub reachable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDiag {
    pub name: String,
    pub tools: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDiag {
    pub paused: bool,
    /// Sources recording now (`screen`, `mic`, `system`).
    pub active: Vec<String>,
    /// Manual recording in progress.
    pub recording: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDiag {
    pub email: Option<String>,
    pub signed_in: bool,
    pub plan_usage_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskDiag {
    /// Free space on the drive of the data folder (unknown off Windows).
    pub free_bytes: Option<u64>,
    /// Space used by Aura's data folder.
    pub aura_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRequest {
    pub thread_id: String,
    pub text: String,
    /// Tray whose chips go with this turn (usually the thread id or "draft").
    pub tray: String,
    #[serde(default = "yes")]
    pub accepts_images: bool,
    #[serde(default)]
    pub options: TurnOptions,
}

fn yes() -> bool {
    true
}

fn random_token() -> String {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).expect("random");
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn meeting_err(e: crate::meeting::MeetingError) -> HostError {
    use crate::meeting::MeetingError as E;
    let code = match e {
        E::AlreadyActive => "busy",
        E::NotActive | E::NotFound => "not_found",
        E::EmptyNote => "invalid",
        E::NothingSaid => "empty",
        E::Source(_) => "asr",
        E::Store(_) => "store",
    };
    HostError::new(code, e.to_string())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub struct Host {
    pub paths: AppPaths,
    platform: Platform,
    store: Store,
    vault: Arc<Vault>,
    settings: RwLock<Settings>,
    /// Inverse patches of the settings the agent changed (022), newest last.
    settings_undo: Mutex<Vec<serde_json::Value>>,
    reminders: crate::reminders::RemindersRepo,
    recipes: crate::recipes::RecipesRepo,
    notes: crate::notes::NotesRepo,
    clock: Arc<dyn crate::reminders::Clock>,
    policy: Arc<RwLock<Policy>>,
    grants: Arc<RwLock<Grants>>,
    privacy: PrivacyRepo,
    consent: Arc<ConsentBroker>,
    events: broadcast::Sender<HostEvent>,
    auth: Arc<AuthService>,
    routes: Arc<Routes>,
    gateway: GatewayHandle,
    http: reqwest::Client,
    codex: Arc<CodexService>,
    launch_state: Arc<RwLock<LaunchState>>,
    attachments: Arc<AttachmentService>,
    trays: Mutex<HashMap<String, ContextTray>>,
    workspaces: Mutex<HashMap<String, (String, PathBuf)>>,
    quick: QuickCommandsRepo,
    mcp_servers: McpServersRepo,
    voice: Arc<Voice>,
    codex_program: Arc<RwLock<Option<PathBuf>>>,
    capture: Arc<crate::recorder::CaptureService>,
    meetings: Arc<crate::meeting::MeetingService>,
    /// The "Entendi assim" briefing the agent prepared, waiting for the user
    /// to press Start (the agent never starts a Meeting).
    pending_briefing: Mutex<Option<String>>,
    /// Free space of the data drive (diagnostics, model downloads).
    disk: Arc<dyn DiskSpace>,
    /// Device tests in Settings (005 AC-001): one live hub per source.
    audio_tests: Mutex<HashMap<aura_audio::AudioSourceKind, AudioTest>>,
    profiles: crate::profiles::ProfilesRepo,
    /// Text of the last selection chip (012): a chip removed by the user is
    /// not re-added automatically while the selection stays the same.
    last_selection: Mutex<Option<String>>,
    /// Text a quick command was applied to; "Substituir seleção" replaces it (019).
    replace_target: Mutex<Option<String>>,
    /// A replacement that can still be undone.
    replaced: Mutex<bool>,
    /// Frozen screens waiting for a region selection (token → frame).
    frozen: Mutex<HashMap<String, aura_capture::frame::Frame>>,
    /// Runtime captured at start: sync methods are called from threads without
    /// a Tokio context (Tauri sync commands, tray, hotkeys).
    rt: tokio::runtime::Handle,
}

/// Unique, sortable name part for pasted files (milliseconds + sequence).
fn chrono_like_stamp() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{ms}-{n}")
}

/// A playable file of a recording (decrypted copy in the session cache).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackMedia {
    /// `audio` or `video`.
    pub kind: String,
    /// `mic`, `system` or `screen`.
    pub source: String,
    pub mime: String,
    pub path: PathBuf,
}

impl PlaybackMedia {
    fn of(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().into_owned();
        let (kind, source, mime) = if let Some(src) = name.strip_suffix(".wav") {
            ("audio", src.to_string(), "audio/wav")
        } else if name.starts_with("screen-") && name.ends_with(".mp4") {
            ("video", "screen".to_string(), "video/mp4")
        } else {
            return None;
        };
        Some(Self {
            kind: kind.into(),
            source,
            mime: mime.into(),
            path,
        })
    }
}

/// Voices offered in Settings › Voice (009 AC-005/006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechOptions {
    pub voices: Vec<crate::speech::SpeechVoice>,
    pub cloud: Vec<SpeechProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechProvider {
    pub id: String,
    pub name: String,
}

/// User request to attach the recent buffer (004 AC-014, 005 AC-007/009).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentClip {
    /// Last N minutes (0 < N <= 30).
    pub minutes: f64,
    /// Keyframes of the screen buffer.
    pub screen: bool,
    /// Audio transcript: `mic`, `system` or `both`.
    pub audio: Option<String>,
}

/// Result of attaching a recording: chips added and files that failed.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingAttach {
    pub chips: Vec<ContextChip>,
    pub failed: Vec<String>,
}

/// A skill in Settings: origin and whether new conversations get it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub origin: SkillOrigin,
    pub enabled: bool,
}

struct AudioTest {
    hub: aura_audio::hub::AudioHub,
    task: tokio::task::JoinHandle<()>,
}

/// One MCP server as the running app-server sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStatus {
    pub name: String,
    pub tools: Vec<String>,
    /// `unsupported`, `notLoggedIn`, `bearerToken`, `oAuth`…
    pub auth: Option<String>,
    /// Why the server did not start (`toolsError`); `None` when connected.
    pub error: Option<String>,
}

/// Parses `mcpServerStatus/list` leniently (tools as map or list).
pub fn parse_mcp_status(v: &serde_json::Value) -> Vec<McpStatus> {
    v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|s| {
            let mut tools: Vec<String> = match &s["tools"] {
                serde_json::Value::Object(m) => m.keys().cloned().collect(),
                serde_json::Value::Array(a) => a
                    .iter()
                    .filter_map(|t| t["name"].as_str().map(str::to_string))
                    .collect(),
                _ => vec![],
            };
            tools.sort();
            McpStatus {
                name: s["name"].as_str().unwrap_or_default().to_string(),
                tools,
                auth: s["authStatus"].as_str().map(str::to_string),
                error: s["toolsError"]
                    .as_str()
                    .filter(|e| !e.is_empty())
                    .map(str::to_string),
            }
        })
        .collect()
}

/// Text preview of a PDF in the Files panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPreview {
    pub total_pages: u32,
    /// Pages with a text layer among the first ones, in order.
    pub pages: Vec<PdfPage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPage {
    pub number: u32,
    pub text: String,
}

/// A file the agent produced in the conversation workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    /// Relative to the workspace, `/`-separated.
    pub path: String,
    pub absolute: PathBuf,
    pub bytes: u64,
    pub modified: i64,
}

/// A frozen screen shown by the region selector.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrozenScreen {
    pub token: String,
    /// PNG of the (already redacted) monitor.
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    /// Monitor area in physical pixels (where the selector window goes).
    pub area: aura_core::placement::Rect,
}

const ASR_MODEL_KEY: &str = "asr.model";

impl Host {
    pub async fn start(cfg: HostConfig) -> HostResult<Arc<Host>> {
        cfg.paths.ensure()?;
        // Decrypted playback copies never outlive a session.
        let _ = std::fs::remove_dir_all(cfg.paths.captures_tmp().join("playback"));
        let store = if cfg.in_memory_store {
            Store::open_in_memory()?
        } else {
            Store::open(&cfg.paths.db())?
        };
        let vault = Arc::new(Vault::open(&store, cfg.platform.protector.as_ref())?);
        let settings = SettingsRepo::new(&store).load()?;
        let privacy = PrivacyRepo::new(store.clone());
        let policy = Arc::new(RwLock::new(privacy.load()?));
        let grants = Arc::new(RwLock::new(privacy.grants()?));
        let (events, _) = broadcast::channel(2048);
        let http = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| HostError::new("network", e.to_string()))?;

        let auth = AuthService::new(
            cfg.siwc.clone(),
            store.clone(),
            cfg.platform.credentials.clone(),
        );
        let routes = Routes::new();
        routes.set(
            CHATGPT_PLAN_ROUTE,
            Arc::new(ChatGptPlanUpstream::new(
                http.clone(),
                Arc::new(AuthTokenProvider(auth.clone())),
            )),
        );
        let providers = ProviderRegistry::new(&store, cfg.platform.credentials.as_ref()).list()?;
        for p in &providers {
            let reg = ProviderRegistry::new(&store, cfg.platform.credentials.as_ref());
            match aura_gateway::build_upstream(&reg, p, http.clone()) {
                Ok(up) => routes.set(&p.id, up),
                Err(e) => tracing::warn!(provider = %p.id, "provider not routable: {e}"),
            }
        }

        let selected_model = SettingsRepo::new(&store)
            .get_raw(ASR_MODEL_KEY)?
            .and_then(|v| v.as_str().map(str::to_string));
        let voice = Arc::new(Voice::new(
            cfg.paths.asr_models(),
            cfg.asr.clone(),
            cfg.hardware.clone(),
            cfg.disk.clone(),
            cfg.platform.audio.clone(),
            events.clone(),
            selected_model,
        ));
        // WAV transcription in-process; other media via the platform worker.
        let media = Arc::new(crate::media::MediaIngestor {
            voice: voice.clone(),
            media: cfg.platform.media.clone(),
            fallback: cfg.platform.heavy_ingest.clone(),
            handle: tokio::runtime::Handle::current(),
        });
        let attachments = Arc::new(AttachmentService::new(media, cfg.paths.ingest_cache()));
        let consent = Arc::new(ConsentBroker::default());
        let capture = crate::recorder::CaptureService::new(
            cfg.platform.clone(),
            cfg.platform.codec.clone(),
            voice.clone(),
            store.clone(),
            (*vault).clone(),
            cfg.paths.screen_segments(),
            cfg.paths.audio_segments(),
            policy.read().unwrap().clone(),
            cfg.capture_interval,
        );
        let extensions_slot: crate::tools::ExtensionsSlot = Arc::default();
        let tools = Arc::new(HostTools {
            extensions: extensions_slot.clone(),
            vault: vault.clone(),
            platform: cfg.platform.clone(),
            policy: policy.clone(),
            grants: grants.clone(),
            privacy: privacy.clone(),
            consent: consent.clone(),
            events: events.clone(),
            attachments: attachments.clone(),
            recent: cfg
                .recent
                .clone()
                .unwrap_or_else(|| capture.clone() as Arc<dyn RecentMedia>),
            ocr_language: match settings.language {
                aura_core::settings::Language::PtBr => "pt-BR".into(),
                aura_core::settings::Language::En => "en-US".into(),
            },
        });
        let mcp_token = random_token();
        let mcp_router = aura_mcp::router(aura_mcp::tools::all(), tools, mcp_token.clone());
        let gateway = aura_gateway::server::start(routes.clone(), Some(mcp_router)).await?;

        let mcp_servers = McpServersRepo::new(store.clone());
        let launch_state = Arc::new(RwLock::new(LaunchState {
            base: BaseConfig {
                gateway_port: gateway.port,
                mcp_port: Some(gateway.port),
                memories: false,
                aura_prompt_tools: aura_mcp::tools::WRITE_TOOLS
                    .iter()
                    .map(|t| t.to_string())
                    .collect(),
                ..BaseConfig::default()
            },
            providers,
            mcp_servers: mcp_servers.list().unwrap_or_default(),
        }));
        let codex_program = Arc::new(RwLock::new(match &cfg.codex {
            CodexRuntime::Binary { program, .. } => program.clone(),
            CodexRuntime::Fake => None,
        }));
        let launcher: Arc<dyn Launcher> = match &cfg.codex {
            CodexRuntime::Binary { on_spawn, .. } => Arc::new(AuraLauncher {
                program: codex_program.clone(),
                codex_home: cfg.paths.codex_home(),
                state: launch_state.clone(),
                vault: cfg.platform.credentials.clone(),
                gateway_token: gateway.token.clone(),
                mcp_token: Secret::new(mcp_token),
                on_spawn: on_spawn.clone(),
            }),
            CodexRuntime::Fake => {
                let fake = aura_codex::fake::FakeConfig::default();
                Arc::new(InMemoryLauncher::new(move |stream| {
                    let fake = fake.clone();
                    tokio::spawn(async move {
                        let (r, w) = tokio::io::split(stream);
                        aura_codex::fake::serve(r, w, fake).await;
                    });
                }))
            }
        };
        let sup_cfg = SupervisorConfig {
            idle: std::time::Duration::from_secs(
                settings.app_server_idle_minutes.max(1) as u64 * 60,
            ),
            ..SupervisorConfig::default()
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let supervisor = AppServerSupervisor::new(launcher, sup_cfg, tx);
        let codex = CodexService::new(supervisor, rx, store.clone(), cfg.paths.workspaces());
        if let Err(e) = crate::core_skills::install(&cfg.paths.core_skills()) {
            tracing::warn!("core skills: {e}");
        }
        codex.set_skill_roots(vec![cfg.paths.skills(), cfg.paths.core_skills()]);
        if let Err(e) = codex.cleanup_on_start() {
            tracing::warn!("workspace cleanup failed: {e}");
        }
        let quick = QuickCommandsRepo::new(store.clone())?;

        let store_for_profiles = store.clone();
        let store_for_notes = store.clone();
        let meetings = Arc::new(crate::meeting::MeetingService::new(
            crate::meeting::MeetingRepo::new(store.clone()),
            capture.clone(),
        ));
        let _ = meetings.repo().recover(now_ms());
        let host = Arc::new(Host {
            paths: cfg.paths,
            platform: cfg.platform,
            store,
            vault,
            settings: RwLock::new(settings),
            settings_undo: Mutex::new(Vec::new()),
            reminders: crate::reminders::RemindersRepo::new(store_for_notes.clone()),
            recipes: crate::recipes::RecipesRepo::new(store_for_notes.clone()),
            notes: crate::notes::NotesRepo::new(store_for_notes.clone()),
            clock: Arc::new(crate::reminders::SystemClock),
            policy,
            grants,
            privacy,
            consent,
            events,
            auth,
            routes,
            gateway,
            http,
            codex,
            launch_state,
            attachments,
            trays: Mutex::new(HashMap::new()),
            workspaces: Mutex::new(HashMap::new()),
            quick,
            mcp_servers,
            voice,
            codex_program,
            capture,
            meetings,
            pending_briefing: Mutex::new(None),
            disk: cfg.disk.clone(),
            audio_tests: Mutex::new(HashMap::new()),
            last_selection: Mutex::new(None),
            replace_target: Mutex::new(None),
            replaced: Mutex::new(false),
            frozen: Mutex::new(HashMap::new()),
            profiles: crate::profiles::ProfilesRepo::new(store_for_profiles),
            rt: tokio::runtime::Handle::current(),
        });
        let weak: std::sync::Weak<dyn crate::tools::ExtensionsAccess> =
            Arc::downgrade(&host) as std::sync::Weak<dyn crate::tools::ExtensionsAccess>;
        let _ = extensions_slot.set(weak);
        host.apply_runtime_settings(&host.settings());
        host.spawn_forwarders();
        host.reconcile_capture();
        // Reminders (020): check every 15 s; stops when the host is dropped.
        let weak_host = Arc::downgrade(&host);
        host.rt.spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                let Some(h) = weak_host.upgrade() else { break };
                h.tick_reminders();
                h.meeting_poll().await;
            }
        });
        Ok(host)
    }

    /// Conversation and login events → the single host stream.
    fn spawn_forwarders(self: &Arc<Self>) {
        let mut conv = self.codex.events();
        let tx = self.events.clone();
        tokio::spawn(async move {
            loop {
                match conv.recv().await {
                    Ok(e) => {
                        let _ = tx.send(HostEvent::Conversation(e));
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("ui lagged {n} conversation events")
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        let mut login = self.auth.progress();
        let tx = self.events.clone();
        tokio::spawn(async move {
            while let Ok(p) = login.recv().await {
                let _ = tx.send(HostEvent::Login(p));
            }
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<HostEvent> {
        self.events.subscribe()
    }

    fn notice(&self, level: &str, message: impl Into<String>) {
        let _ = self.events.send(HostEvent::Notice {
            level: level.into(),
            message: message.into(),
        });
    }

    // ---------------------------------------------------------------- settings

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    // ------------------------------------------------------------ meetings

    pub fn recipes_list(&self) -> HostResult<Vec<crate::recipes::Recipe>> {
        self.recipes
            .list()
            .map_err(|e| HostError::new("recipe", e.to_string()))
    }

    /// Briefing prepared by the agent (`meeting_brief_save`); starting a
    /// Meeting without text uses it.
    pub fn meeting_brief(&self) -> Option<String> {
        self.pending_briefing.lock().unwrap().clone()
    }

    pub fn meeting_set_brief(&self, text: &str) -> HostResult<()> {
        let text = text.trim();
        if text.chars().count() > 8_000 {
            return Err(HostError::new(
                "invalid",
                "briefing longo demais (máx. 8000 caracteres)",
            ));
        }
        *self.pending_briefing.lock().unwrap() = (!text.is_empty()).then(|| text.to_string());
        let _ = self.events.send(HostEvent::Meeting {
            id: String::new(),
            status: "briefing".into(),
        });
        Ok(())
    }

    /// The meeting in progress (023).
    pub fn meeting_active(&self) -> Option<crate::meeting::Meeting> {
        self.meetings.active()
    }

    /// Starts a Meeting. Only an explicit call from the user starts audio
    /// capture; the privacy pause blocks it.
    pub async fn meeting_start(
        &self,
        title: &str,
        kind: &str,
        briefing: &str,
    ) -> HostResult<crate::meeting::Meeting> {
        if self.policy.read().unwrap().paused {
            return Err(HostError::new("paused", "a privacidade está pausada"));
        }
        let pending = self.pending_briefing.lock().unwrap().take();
        let briefing = if briefing.trim().is_empty() {
            pending.as_deref().unwrap_or("")
        } else {
            briefing
        };
        let m = self
            .meetings
            .start(title, kind, briefing, now_ms())
            .map_err(meeting_err)?;
        self.capture.set_meeting(true).await;
        let _ = self.events.send(HostEvent::Meeting {
            id: m.id.clone(),
            status: "active".into(),
        });
        Ok(m)
    }

    /// A note or ★ marker on the meeting timeline.
    pub fn meeting_note(&self, text: &str) -> HostResult<crate::meeting::Utterance> {
        let u = self
            .meetings
            .add_note(text, now_ms())
            .map_err(meeting_err)?;
        let _ = self.events.send(HostEvent::Meeting {
            id: u.meeting_id.clone(),
            status: "updated".into(),
        });
        Ok(u)
    }

    pub fn meeting_set_paused(&self, paused: bool) -> HostResult<()> {
        self.meetings
            .set_paused(paused, now_ms())
            .map_err(meeting_err)
    }

    /// Ends the Meeting after a last read of the audio.
    pub async fn meeting_stop(&self) -> HostResult<crate::meeting::Meeting> {
        let m = self.meetings.stop(now_ms()).await.map_err(meeting_err)?;
        self.capture.set_meeting(false).await;
        if !self.settings().meeting_keep_audio {
            self.capture
                .erase_audio(m.started_at, m.ended_at.unwrap_or_else(now_ms) + 1_000)
                .await;
        }
        let _ = self.events.send(HostEvent::Meeting {
            id: m.id.clone(),
            status: "ended".into(),
        });
        Ok(m)
    }

    /// "Esqueci de iniciar": a Meeting from the last `minutes` of audio buffer.
    pub async fn meeting_from_buffer(
        &self,
        title: &str,
        minutes: u32,
    ) -> HostResult<crate::meeting::Meeting> {
        let m = self
            .meetings
            .from_buffer(title, minutes, now_ms())
            .await
            .map_err(meeting_err)?;
        let _ = self.events.send(HostEvent::Meeting {
            id: m.id.clone(),
            status: "ended".into(),
        });
        Ok(m)
    }

    /// Reads new speech into the active Meeting (the timer calls it).
    pub async fn meeting_poll(&self) {
        match self.meetings.poll(now_ms()).await {
            Ok(new) if !new.is_empty() => {
                if let Some(m) = self.meetings.active() {
                    let _ = self.events.send(HostEvent::Meeting {
                        id: m.id,
                        status: "updated".into(),
                    });
                }
            }
            Ok(_) => {}
            Err(e) => tracing::warn!("meeting poll: {e}"),
        }
    }

    /// Saved meetings store (the Meeting UI and tests read through it).
    pub fn meetings_repo(&self) -> &crate::meeting::MeetingRepo {
        self.meetings.repo()
    }

    pub fn meetings_list(&self) -> HostResult<Vec<crate::meeting::Meeting>> {
        self.meetings.repo().list(100).map_err(meeting_err)
    }

    pub fn meeting_utterances(&self, id: &str) -> HostResult<Vec<crate::meeting::Utterance>> {
        self.meetings.repo().utterances(id).map_err(meeting_err)
    }

    /// How many audio segments of a Meeting are still on disk (0 once erased).
    pub fn meeting_audio_kept(&self, id: &str) -> HostResult<usize> {
        let m = self.meetings.repo().get(id).map_err(meeting_err)?;
        Ok(self
            .capture
            .audio_segments_between(m.started_at, m.ended_at.unwrap_or_else(now_ms) + 1_000))
    }

    /// What the model may read: personal data masked when the user asked.
    fn for_model(&self, text: &str) -> String {
        if self.settings().meeting_redact_pii {
            crate::pii::redact(text)
        } else {
            text.to_string()
        }
    }

    pub fn meeting_delete(&self, id: &str) -> HostResult<()> {
        if self.meetings.active().is_some_and(|m| m.id == id) {
            return Err(HostError::new(
                "busy",
                "encerre a reunião antes de apagá-la",
            ));
        }
        self.meetings.repo().delete(id).map_err(meeting_err)
    }

    pub fn meeting_search(
        &self,
        query: &str,
        meeting: Option<&str>,
    ) -> HostResult<Vec<crate::meeting::Hit>> {
        self.meetings
            .repo()
            .search(query, meeting, 30)
            .map_err(meeting_err)
    }

    /// Fires the reminders due at `now` (the timer calls it every few seconds).
    pub fn tick_reminders_at(&self, now: i64, offset: i32) -> usize {
        let due = self.reminders.fire_due(now, offset).unwrap_or_default();
        for r in &due {
            let _ = self.events.send(HostEvent::Reminder {
                id: r.id.clone(),
                text: r.text.clone(),
            });
        }
        due.len()
    }

    pub fn tick_reminders(&self) -> usize {
        self.tick_reminders_at(self.clock.now(), self.clock.utc_offset_secs())
    }

    pub fn update_settings(&self, patch: SettingsPatch) -> HostResult<Settings> {
        let next = self.settings.read().unwrap().apply(&patch)?;
        SettingsRepo::new(&self.store).save(&next)?;
        *self.settings.write().unwrap() = next.clone();
        self.apply_runtime_settings(&next);
        Ok(next)
    }

    /// Persists settings changed by a dedicated command (not a patch).
    fn replace_settings(&self, next: Settings) -> HostResult<Settings> {
        SettingsRepo::new(&self.store).save(&next)?;
        *self.settings.write().unwrap() = next.clone();
        self.apply_runtime_settings(&next);
        Ok(next)
    }

    /// Settings that configure running services (memories, cloud ASR).
    fn apply_runtime_settings(&self, s: &Settings) {
        // Texts Aura adds to turns (mode announcements) follow the UI language.
        self.codex.set_language(match s.language {
            aura_core::settings::Language::PtBr => aura_codex::modes::UiLanguage::PtBr,
            aura_core::settings::Language::En => aura_codex::modes::UiLanguage::En,
        });
        self.codex.set_yolo(s.yolo);
        self.capture.set_devices(
            s.microphone_device_id.clone(),
            s.system_audio_device_id.clone(),
        );
        {
            let mut launch = self.launch_state.write().unwrap();
            launch.base.memories = s.memories;
            launch.base.disabled_skills = s.disabled_skills.iter().map(PathBuf::from).collect();
        }
        let cloud = s.cloud_asr_provider.as_deref().and_then(|id| {
            let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
            let provider = reg.get(id).ok()??;
            let target = reg.target(&provider).ok()?;
            let model = presets()
                .into_iter()
                .find(|p| p.id == provider.preset)
                .and_then(|p| p.transcription_models.first().cloned())
                .unwrap_or_else(|| "whisper-1".into());
            Some(Arc::new(aura_asr::cloud::CloudTranscriber {
                base_url: target.base_url,
                model,
                credential: target.credential,
                http: self.http.clone(),
                verbose_json: true,
            })
                as Arc<dyn aura_asr::transcriber::Transcriber>)
        });
        self.voice.set_cloud(cloud);
    }

    // ----------------------------------------------------------------- privacy

    pub fn privacy(&self) -> PrivacyView {
        let p = self.policy.read().unwrap().clone();
        PrivacyView {
            screen: p.screen,
            mic: p.mic,
            system_audio: p.system_audio,
            paused: p.paused,
            exclusions: p.exclusions,
            retention: p.retention,
        }
    }

    /// Starts/stops background recorders for the current policy and tells
    /// the UI which sources are recording (indicators, 004/005 TK-003).
    fn reconcile_capture(&self) {
        let capture = self.capture.clone();
        let events = self.events.clone();
        let p = self.policy.read().unwrap().clone();
        self.rt.spawn(async move {
            let recording = capture.apply(&p).await;
            let mut state = Self::privacy_state(&p);
            state.recording = recording;
            let _ = events.send(HostEvent::Privacy(state));
        });
    }

    fn privacy_state(p: &Policy) -> PrivacyState {
        let m = |s: SourcePolicy| {
            match s.mode {
                CaptureMode::Off => "off",
                CaptureMode::OnDemand => "onDemand",
                CaptureMode::RecentBuffer { .. } => "recentBuffer",
                CaptureMode::Manual => "manual",
                CaptureMode::Continuous => "continuous",
            }
            .to_string()
        };
        PrivacyState {
            paused: p.paused,
            screen: m(p.screen),
            mic: m(p.mic),
            system_audio: m(p.system_audio),
            recording: vec![],
        }
    }

    fn save_policy(&self, p: Policy) -> HostResult<PrivacyView> {
        self.privacy.save(&p)?;
        *self.policy.write().unwrap() = p;
        self.reconcile_capture();
        Ok(self.privacy())
    }

    pub fn set_source_policy(
        &self,
        source: Source,
        mode: CaptureMode,
        agent: AgentPermission,
    ) -> HostResult<PrivacyView> {
        // Recent buffer: 1 to 30 minutes (004 AC-013).
        if let CaptureMode::RecentBuffer { minutes } = mode
            && !(1..=30).contains(&minutes)
        {
            return Err(HostError::new(
                "out_of_range",
                "o buffer recente vai de 1 a 30 minutos",
            ));
        }
        let mut p = self.policy.read().unwrap().clone();
        let sp = SourcePolicy { mode, agent };
        match source {
            Source::Screen | Source::Selection => p.screen = sp,
            Source::Mic => p.mic = sp,
            Source::SystemAudio => p.system_audio = sp,
        }
        self.save_policy(p)
    }

    /// Retention limits (004 AC-016): 1–365 days, 1–1000 GB.
    pub fn set_retention(
        &self,
        days: u32,
        max_gb: u32,
        apply_to_manual: bool,
    ) -> HostResult<PrivacyView> {
        if !(1..=365).contains(&days) || !(1..=1000).contains(&max_gb) {
            return Err(HostError::new(
                "out_of_range",
                "retenção: de 1 a 365 dias e de 1 a 1000 GB",
            ));
        }
        let mut p = self.policy.read().unwrap().clone();
        p.retention = aura_policy::Retention {
            days,
            max_gb,
            apply_to_manual,
        };
        self.save_policy(p)
    }

    pub fn set_paused(&self, paused: bool) -> HostResult<PrivacyView> {
        let mut p = self.policy.read().unwrap().clone();
        p.paused = paused;
        self.save_policy(p)
    }

    pub fn toggle_pause(&self) -> HostResult<PrivacyView> {
        let paused = self.policy.read().unwrap().paused;
        self.set_paused(!paused)
    }

    pub fn upsert_exclusion(&self, rule: ExclusionRule) -> HostResult<PrivacyView> {
        let mut rule = rule;
        if rule.id.trim().is_empty() {
            rule.id = format!("user-{}", uuid::Uuid::new_v4().simple());
        }
        let builtin = self
            .policy
            .read()
            .unwrap()
            .exclusions
            .iter()
            .any(|r| r.id == rule.id && r.builtin);
        rule.builtin = builtin;
        self.privacy.upsert_exclusion(&rule)?;
        let mut p = self.policy.read().unwrap().clone();
        p.exclusions = self.privacy.exclusions()?;
        self.save_policy(p)
    }

    pub fn remove_exclusion(&self, id: &str) -> HostResult<PrivacyView> {
        if !self.privacy.remove_exclusion(id)? {
            return Err(HostError::new(
                "builtin",
                "regras embutidas só podem ser desativadas",
            ));
        }
        let mut p = self.policy.read().unwrap().clone();
        p.exclusions = self.privacy.exclusions()?;
        self.save_policy(p)
    }

    /// Access log, newest first, with the thread of each agent conversation
    /// so Settings can link to it (QA-031).
    pub fn access_log(&self, limit: u32) -> HostResult<Vec<AccessLogEntry>> {
        let mut log = self.privacy.access_log(limit.clamp(1, 1000))?;
        let repo = aura_store::conversations::ConversationsRepo::new(&self.store);
        for e in &mut log {
            if let Some(uuid) = e.conversation.as_deref().filter(|c| !c.is_empty()) {
                e.thread_id = repo.get_by_uuid(uuid).ok().flatten().map(|m| m.thread_id);
            }
        }
        Ok(log)
    }

    /// Thumbnail of what the agent received, as a `data:image/png` URL.
    pub fn access_thumbnail(&self, id: i64) -> HostResult<Option<String>> {
        use base64::Engine as _;
        let Some(sealed) = self.privacy.thumbnail(id)? else {
            return Ok(None);
        };
        let png = self
            .vault
            .open_bytes("access-thumb", &sealed)
            .map_err(|e| HostError::new("vault", e.to_string()))?;
        Ok(Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png)
        )))
    }

    /// "Create with AI" (017): the Overlay starts a new conversation in
    /// `mode` (`chat`, `task`, `plan`) and sends `text`.
    pub fn agent_task(&self, text: &str, mode: &str) -> HostResult<()> {
        if text.trim().is_empty() {
            return Err(HostError::new(
                "invalid",
                "descreva o que o agente deve fazer",
            ));
        }
        if !matches!(mode, "chat" | "task" | "plan") {
            return Err(HostError::new(
                "invalid",
                format!("modo desconhecido: {mode}"),
            ));
        }
        let _ = self.events.send(HostEvent::AgentTask {
            text: text.trim().to_string(),
            mode: mode.to_string(),
        });
        Ok(())
    }

    fn extensions_changed(&self) {
        let _ = self.events.send(HostEvent::ExtensionsChanged {});
    }

    /// Asks the Overlay to show a conversation of the access log.
    pub fn reveal_conversation(&self, thread_id: &str) -> HostResult<()> {
        aura_store::conversations::ConversationsRepo::new(&self.store)
            .get(thread_id)?
            .ok_or_else(|| HostError::new("not_found", "conversa não encontrada"))?;
        let _ = self.events.send(HostEvent::OpenConversation {
            thread_id: thread_id.to_string(),
        });
        Ok(())
    }

    pub fn clear_access_log(&self) -> HostResult<()> {
        Ok(self.privacy.clear_access_log()?)
    }

    pub fn answer_consent(&self, id: &str, answer: ConsentAnswer) -> HostResult<()> {
        if self.consent.answer(id, answer) {
            Ok(())
        } else {
            Err(HostError::new("not_found", "o pedido já expirou"))
        }
    }

    // -------------------------------------------------------------------- auth

    pub fn auth_status(&self) -> HostResult<AuthStatus> {
        Ok(AuthStatus {
            accounts: self.auth.accounts()?,
            active: self.auth.active()?,
        })
    }

    /// Opens the browser (the shell does it with the returned URL) and
    /// completes in the background; progress arrives as `Login` events.
    pub async fn login(
        self: &Arc<Self>,
        reauthorize: Option<String>,
        force_consent: bool,
    ) -> HostResult<String> {
        // AuthService owns the task and emits every terminal outcome. Its
        // progress is already forwarded by forward_events; emitting again here
        // duplicates completion and incorrectly turns cancellation into failure.
        let (attempt, _done) = self.auth.begin_login(reauthorize, force_consent).await?;
        Ok(attempt.authorize_url)
    }

    pub async fn cancel_login(&self) {
        self.auth.cancel_login().await;
    }

    pub async fn logout(&self, client_id: &str) -> HostResult<bool> {
        Ok(self.auth.logout(client_id).await?)
    }

    pub fn switch_account(&self, client_id: &str) -> HostResult<()> {
        Ok(self.auth.switch(client_id)?)
    }

    pub fn mark_welcomed(&self, client_id: &str) -> HostResult<()> {
        Ok(self.auth.mark_welcomed(client_id)?)
    }

    // --------------------------------------------------------------- providers

    pub fn provider_presets(&self) -> Vec<Preset> {
        presets()
    }

    pub fn providers(&self) -> HostResult<Vec<Provider>> {
        Ok(ProviderRegistry::new(&self.store, self.platform.credentials.as_ref()).list()?)
    }

    fn refresh_provider_routes(&self) -> HostResult<()> {
        let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
        let list = reg.list()?;
        for id in self.routes.ids() {
            if id != CHATGPT_PLAN_ROUTE && !list.iter().any(|p| p.id == id) {
                self.routes.remove(&id);
            }
        }
        for p in &list {
            if let Ok(up) = aura_gateway::build_upstream(&reg, p, self.http.clone()) {
                self.routes.set(&p.id, up);
            }
        }
        self.launch_state.write().unwrap().providers = list;
        Ok(())
    }

    /// Saves a provider; `credential` goes straight to the vault.
    pub fn save_provider(
        &self,
        mut draft: ProviderDraft,
        credential: Option<String>,
    ) -> HostResult<Provider> {
        draft.credential = credential
            .filter(|c| !c.trim().is_empty())
            .map(|c| Secret::new(c.trim().to_string()));
        let p =
            ProviderRegistry::new(&self.store, self.platform.credentials.as_ref()).upsert(draft)?;
        self.refresh_provider_routes()?;
        let _ = self.events.send(HostEvent::ProvidersChanged {});
        Ok(p)
    }

    pub fn remove_provider(&self, id: &str) -> HostResult<()> {
        ProviderRegistry::new(&self.store, self.platform.credentials.as_ref()).remove(id)?;
        self.refresh_provider_routes()?;
        // Settings must not keep pointing at (or consenting to) it.
        let mut s = self.settings();
        let before = s.clone();
        s.tts_cloud_consent.retain(|c| c != id);
        for field in [&mut s.tts_provider, &mut s.cloud_asr_provider] {
            if field.as_deref() == Some(id) {
                *field = None;
            }
        }
        if s != before {
            self.replace_settings(s)?;
        }
        let _ = self.events.send(HostEvent::ProvidersChanged {});
        Ok(())
    }

    /// Adds or corrects a model by hand (003 AC-013); discovery keeps it.
    pub fn save_provider_model(
        &self,
        id: &str,
        spec: aura_gateway::registry::ModelSpec,
    ) -> HostResult<Provider> {
        let p = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref())
            .save_model(id, spec)?;
        self.refresh_provider_routes()?;
        let _ = self.events.send(HostEvent::ProvidersChanged {});
        Ok(p)
    }

    pub fn remove_provider_model(&self, id: &str, model_id: &str) -> HostResult<Provider> {
        let p = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref())
            .remove_model(id, model_id)?;
        self.refresh_provider_routes()?;
        let _ = self.events.send(HostEvent::ProvidersChanged {});
        Ok(p)
    }

    /// Tests the connection, stores status and discovered models.
    pub async fn test_provider(&self, id: &str) -> HostResult<Provider> {
        let (provider, target) = {
            let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
            let p = reg
                .get(id)?
                .ok_or_else(|| HostError::new("not_found", "provedor não encontrado"))?;
            let t = reg.target(&p)?;
            (p, t)
        };
        let result = aura_gateway::discovery::test_connection(&provider, &target).await;
        let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
        if result.category == aura_gateway::discovery::ConnectionCategory::Ok {
            reg.set_status(id, ProviderStatus::Verified, None)?;
            if !result.models.is_empty() {
                reg.set_models(id, result.models)?;
            }
        } else if result.category == aura_gateway::discovery::ConnectionCategory::NotFound
            && provider.models.iter().any(|m| m.manual)
        {
            // No model listing, but the user entered models (003 AC-013):
            // reachable endpoint, nothing to report as an error.
            reg.set_status(id, ProviderStatus::Unverified, None)?;
        } else {
            reg.set_status(id, ProviderStatus::Error, Some(result.detail))?;
        }
        self.refresh_provider_routes()?;
        let _ = self.events.send(HostEvent::ProvidersChanged {});
        reg.get(id)?
            .ok_or_else(|| HostError::new("not_found", "provedor não encontrado"))
    }

    // ------------------------------------------------------------ conversations

    pub async fn start_conversation(
        &self,
        mut opts: StartOptions,
    ) -> HostResult<StartedConversation> {
        let s = self.settings();
        if opts.personal_instructions.is_empty() {
            opts.personal_instructions = s.personal_instructions.clone();
        }
        // App profile of the app the Overlay was opened over (009 TK-004).
        let mut profile_model = None;
        if let Some(app) = self.platform.foreground.current()
            && let Some(p) = self.profiles.for_app(&app)
        {
            if opts.profile.is_none() && !p.instructions.trim().is_empty() {
                opts.profile = Some((p.name.clone(), p.instructions.clone()));
            }
            profile_model = p.default_model.clone();
        }
        opts.model = start_model(
            &opts.provider,
            opts.model.as_deref(),
            s.default_model.as_deref(),
            profile_model.as_deref(),
        );
        opts.language = match s.language {
            aura_core::settings::Language::PtBr => aura_codex::modes::UiLanguage::PtBr,
            aura_core::settings::Language::En => aura_codex::modes::UiLanguage::En,
        };
        // BYOK: tell Codex the model limits it has no metadata for.
        if let Some(id) = opts
            .provider
            .strip_prefix("aura-")
            .filter(|id| *id != CHATGPT_PLAN_ROUTE)
            && let Ok(Some(p)) =
                ProviderRegistry::new(&self.store, self.platform.credentials.as_ref()).get(id)
        {
            opts.config_overrides
                .extend(aura_gateway::codex_config::provider_override(
                    &p,
                    self.gateway.port,
                ));
            if let Some(model) = &opts.model {
                opts.config_overrides
                    .extend(aura_gateway::codex_config::thread_overrides(&p, model));
            }
        }
        let started = self.codex.start(opts).await?;
        self.workspaces.lock().unwrap().insert(
            started.thread_id.clone(),
            (started.conversation_uuid.clone(), started.workspace.clone()),
        );
        Ok(started)
    }

    fn conversation_of(&self, thread_id: &str) -> Option<(String, PathBuf)> {
        if let Some(c) = self.workspaces.lock().unwrap().get(thread_id).cloned() {
            return Some(c);
        }
        let meta = aura_store::conversations::ConversationsRepo::new(&self.store)
            .get(thread_id)
            .ok()??;
        let ws = meta.workspace_path.clone();
        self.attachments.load_workspace(&ws);
        self.workspaces.lock().unwrap().insert(
            thread_id.to_string(),
            (meta.conversation_uuid.clone(), ws.clone()),
        );
        Some((meta.conversation_uuid, ws))
    }

    /// Sends text + the chips of `tray`. Returns the turn id.
    pub async fn send(&self, req: SendRequest) -> HostResult<String> {
        self.adopt_chip_files(&req.tray, &req.thread_id);
        let inputs = self.drain_tray(&req.tray, &req.text, req.accepts_images)?;
        let mut opts = req.options;
        if opts.effort.is_none() {
            opts.effort = self.settings().default_effort;
        }
        Ok(self.codex.send(&req.thread_id, &inputs, opts).await?)
    }

    /// Adds instructions to the running turn (Ctrl+Enter while streaming).
    pub async fn steer(&self, thread_id: &str, text: &str, tray: &str) -> HostResult<String> {
        let inputs = self.drain_tray(tray, text, true)?;
        Ok(self.codex.steer(thread_id, &inputs).await?)
    }

    pub async fn interrupt(&self, thread_id: &str) -> HostResult<()> {
        Ok(self.codex.interrupt(thread_id).await?)
    }

    pub async fn compact(&self, thread_id: &str) -> HostResult<()> {
        Ok(self.codex.compact(thread_id).await?)
    }

    pub async fn set_mode(&self, thread_id: &str, mode: ConversationMode) -> HostResult<()> {
        Ok(self.codex.set_mode(thread_id, mode).await?)
    }

    pub async fn history(&self, q: HistoryQuery) -> HostResult<HistoryPage> {
        Ok(self.codex.list(q).await?)
    }

    pub async fn open_conversation(&self, thread_id: &str) -> HostResult<Vec<TranscriptMessage>> {
        let t = self.codex.open(thread_id).await?;
        let _ = self.conversation_of(thread_id);
        Ok(t)
    }

    pub async fn rename(&self, thread_id: &str, name: &str) -> HostResult<()> {
        Ok(self.codex.rename(thread_id, name).await?)
    }

    pub async fn pin(&self, thread_id: &str, pinned: bool) -> HostResult<()> {
        Ok(self.codex.pin(thread_id, pinned).await?)
    }

    pub async fn archive(&self, thread_id: &str) -> HostResult<()> {
        Ok(self.codex.archive(thread_id).await?)
    }

    pub async fn unarchive(&self, thread_id: &str) -> HostResult<()> {
        Ok(self.codex.unarchive(thread_id).await?)
    }

    pub async fn delete_conversation(&self, thread_id: &str) -> HostResult<()> {
        if let Some((uuid, _)) = self.conversation_of(thread_id) {
            self.attachments.forget_conversation(&uuid);
            let _ = self.privacy.revoke_conversation(&uuid);
            self.grants.write().unwrap().revoke_conversation(&uuid);
        }
        self.trays.lock().unwrap().remove(thread_id);
        self.workspaces.lock().unwrap().remove(thread_id);
        Ok(self.codex.delete(thread_id).await?)
    }

    pub async fn close_ephemeral(&self, thread_id: &str) -> HostResult<()> {
        if let Some((uuid, _)) = self.conversation_of(thread_id) {
            self.attachments.forget_conversation(&uuid);
            self.grants.write().unwrap().revoke_conversation(&uuid);
        }
        self.trays.lock().unwrap().remove(thread_id);
        self.workspaces.lock().unwrap().remove(thread_id);
        Ok(self.codex.close_ephemeral(thread_id).await?)
    }

    pub async fn respond(&self, request_id: &str, decision: ApprovalDecision) -> HostResult<()> {
        Ok(self.codex.respond(request_id, decision).await?)
    }

    /// Models of the ChatGPT plan route: Aura's plan catalog, described by
    /// Codex `model/list` when it knows them (017).
    pub async fn models(&self) -> HostResult<Vec<ModelInfo>> {
        Ok(aura_codex::models::plan_catalog(
            &self.codex.codex_models().await?,
        ))
    }

    // ----------------------------------------------------------------- context

    /// Moves files kept with chips (Clips attached before the conversation
    /// existed) into `<workspace>/clips` and points the chips at them.
    fn adopt_chip_files(&self, tray: &str, thread_id: &str) {
        let Some((_, ws)) = self.conversation_of(thread_id) else {
            return;
        };
        let target_root = ws.join("clips");
        let mut trays = self.trays.lock().unwrap();
        let Some(t) = trays.get_mut(tray) else { return };
        for mut chip in t.list().to_vec() {
            let Some(dir) = chip.files_dir.clone() else {
                continue;
            };
            if dir.starts_with(&ws) || !dir.exists() {
                continue;
            }
            let Some(name) = dir.file_name() else {
                continue;
            };
            let dest = target_root.join(name);
            if std::fs::create_dir_all(&target_root).is_err() {
                continue;
            }
            if std::fs::rename(&dir, &dest).is_err() {
                if copy_dir(&dir, &dest).is_err() {
                    continue;
                }
                let _ = std::fs::remove_dir_all(&dir);
            }
            let moved = |p: &PathBuf| match p.strip_prefix(&dir) {
                Ok(rest) => dest.join(rest),
                Err(_) => p.clone(),
            };
            if let ChipPayload::Images { paths, .. } = &mut chip.payload {
                for p in paths.iter_mut() {
                    *p = moved(p);
                }
            }
            chip.preview_path = chip.preview_path.as_ref().map(moved);
            chip.files_dir = Some(dest);
            let _ = t.replace(chip);
        }
    }

    fn drain_tray(
        &self,
        tray: &str,
        text: &str,
        accepts_images: bool,
    ) -> HostResult<Vec<TurnInput>> {
        let mut trays = self.trays.lock().unwrap();
        let t = trays.entry(tray.to_string()).or_default();
        Ok(t.drain_for_turn(text, accepts_images)?)
    }

    pub fn tray(&self, tray: &str) -> Vec<ContextChip> {
        self.trays
            .lock()
            .unwrap()
            .get(tray)
            .map(|t| t.list().to_vec())
            .unwrap_or_default()
    }

    pub fn remove_chip(&self, tray: &str, chip_id: &str) -> HostResult<Vec<ContextChip>> {
        let mut trays = self.trays.lock().unwrap();
        let t = trays.entry(tray.to_string()).or_default();
        t.remove(chip_id)?;
        Ok(t.list().to_vec())
    }

    pub fn clear_tray(&self, tray: &str) {
        self.trays.lock().unwrap().remove(tray);
    }

    /// Moves chips gathered before the conversation existed ("draft").
    pub fn move_tray(&self, from: &str, to: &str) {
        let mut trays = self.trays.lock().unwrap();
        if let Some(t) = trays.remove(from) {
            trays.insert(to.to_string(), t);
        }
    }

    fn add_chip(&self, tray: &str, chip: ContextChip) -> HostResult<ContextChip> {
        let mut trays = self.trays.lock().unwrap();
        let t = trays.entry(tray.to_string()).or_default();
        Ok(t.add(chip)?.clone())
    }

    /// Skill chosen in the `/` menu (015): a Chip the turn turns into `$name`.
    pub async fn attach_skill(&self, tray: &str, name: &str) -> HostResult<ContextChip> {
        let skill = self
            .skills_catalog()
            .await?
            .into_iter()
            .find(|s| s.name == name && s.enabled)
            .ok_or_else(|| HostError::new("skill", format!("skill indisponível: {name}")))?;
        let existing = self
            .tray(tray)
            .into_iter()
            .find(|c| matches!(&c.payload, ChipPayload::Skill { name: n, .. } if n == name));
        if let Some(chip) = existing {
            return Ok(chip);
        }
        let chip = ContextChip::new(
            ChipKind::Skill,
            skill.name.clone(),
            ChipPayload::Skill {
                name: skill.name,
                path: skill.path,
            },
        );
        self.add_chip(tray, chip)
    }

    /// User-initiated screen capture into a chip (Ctrl+Shift+S, `@tela`).
    pub async fn capture_screen(&self, tray: &str, window_only: bool) -> HostResult<ContextChip> {
        let app = self
            .platform
            .foreground
            .current()
            .ok_or_else(|| HostError::new("capture", "nenhuma janela ativa"))?;
        let target = if window_only {
            CapTarget::Window { window: app.window }
        } else {
            let area = self
                .platform
                .foreground
                .monitor_area(&app.monitor_id)
                .ok_or_else(|| HostError::new("capture", "monitor não encontrado"))?;
            CapTarget::Monitor {
                id: app.monitor_id.clone(),
                area,
            }
        };
        let policy = self.policy.read().unwrap().clone();
        let grants = self.grants.read().unwrap().clone();
        let platform = self.platform.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            capture_with_policy(
                &policy,
                &grants,
                Requester::User,
                &target,
                platform.frames.as_ref(),
                platform.inventory.as_ref(),
            )
        })
        .await
        .map_err(|e| HostError::new("capture", e.to_string()))??;
        let label = if window_only {
            format!("Janela · {}", app.process_name)
        } else {
            format!("Tela · {}", app.title)
        };
        let chip = match outcome {
            CaptureOutcome::Captured { frame, redacted } => {
                let frame = frame.downscale(crate::tools::MAX_IMAGE_SIDE);
                let path = self
                    .paths
                    .captures_tmp()
                    .join(format!("{}.png", uuid::Uuid::new_v4()));
                std::fs::write(&path, frame.to_png())?;
                let mut chip = ContextChip::new(
                    ChipKind::Screen,
                    label,
                    ChipPayload::Image { path: path.clone() },
                );
                chip.preview_path = Some(path);
                if redacted {
                    chip.label.push_str(" · áreas cobertas");
                }
                chip
            }
            CaptureOutcome::Denied { reason } => {
                ContextChip::blocked(ChipKind::Screen, label, reason.code())
            }
            CaptureOutcome::NeedsPermission => {
                ContextChip::blocked(ChipKind::Screen, label, "permission")
            }
        };
        self.add_chip(tray, chip)
    }

    /// Attaches the last minutes of the recent buffer as a Clip chip: up to 8
    /// keyframes and/or the transcript of the same interval; frames and audio
    /// are kept in the conversation workspace (draft cache before the
    /// conversation exists).
    pub async fn attach_recent(
        &self,
        tray: &str,
        thread_id: Option<&str>,
        req: RecentClip,
    ) -> HostResult<ContextChip> {
        let audio = req.audio.as_deref();
        if !(req.minutes > 0.0 && req.minutes <= 30.0)
            || (!req.screen && audio.is_none())
            || audio.is_some_and(|a| !matches!(a, "mic" | "system" | "both"))
        {
            return Err(HostError::new(
                "invalid",
                "escolha de 1 a 30 minutos e ao menos uma fonte",
            ));
        }
        let policy = self.policy.read().unwrap().clone();
        if policy.paused {
            return Err(HostError::new("paused", "a privacidade está pausada"));
        }
        let buffered = |m: CaptureMode| {
            matches!(
                m,
                CaptureMode::RecentBuffer { .. } | CaptureMode::Continuous
            )
        };
        let audio_sources: Vec<Source> = match audio {
            Some("mic") => vec![Source::Mic],
            Some("system") => vec![Source::SystemAudio],
            Some(_) => vec![Source::Mic, Source::SystemAudio],
            None => vec![],
        };
        let mode = |s: Source| match s {
            Source::Screen | Source::Selection => policy.screen.mode,
            Source::Mic => policy.mic.mode,
            Source::SystemAudio => policy.system_audio.mode,
        };
        // "Both" needs at least one audio buffer; a single source needs its own.
        let audio_ok = audio_sources.is_empty() || audio_sources.iter().any(|s| buffered(mode(*s)));
        if (req.screen && !buffered(policy.screen.mode)) || !audio_ok {
            return Err(HostError::new(
                "not_recording",
                "a fonte escolhida não está com o buffer recente ligado",
            ));
        }
        let mut sources = audio_sources;
        if req.screen {
            sources.insert(0, Source::Screen);
        }
        for s in sources.into_iter().filter(|s| buffered(mode(*s))) {
            let access = aura_policy::AccessRequest {
                source: s,
                requester: Requester::User,
                target: aura_policy::Target::Range,
                visible_windows: vec![],
                background: false,
            };
            let _ = self.privacy.log(&access, &aura_policy::Decision::Allow);
        }

        let root = match thread_id.and_then(|t| self.conversation_of(t)) {
            Some((_, ws)) => ws,
            None => self.paths.captures_tmp().join("draft"),
        };
        let dir = root.join("clips").join(chrono_like_stamp());
        std::fs::create_dir_all(&dir).map_err(|e| HostError::new("clip", e.to_string()))?;
        let mut frames = Vec::new();
        let mut labels = Vec::new();
        if req.screen {
            let shots = self
                .capture
                .screen_frames(req.minutes, 8)
                .await
                .map_err(|e| HostError::new("not_recording", e))?;
            for (i, (label, png)) in shots.into_iter().enumerate() {
                let path = dir.join(format!("frame-{:02}.png", i + 1));
                std::fs::write(&path, png).map_err(|e| HostError::new("clip", e.to_string()))?;
                labels.push(format!("Quadro {} ({label})", i + 1));
                frames.push(path);
            }
        }
        let transcript = match audio {
            Some(which) => {
                let text = self
                    .capture
                    .audio_transcript(req.minutes, which)
                    .await
                    .map_err(|e| HostError::new("not_recording", e))?;
                let capture = self.capture.clone();
                let (minutes, which, d) = (req.minutes, which.to_string(), dir.clone());
                let _ = tokio::task::spawn_blocking(move || {
                    capture.export_recent_audio(minutes, &which, &d)
                })
                .await;
                let _ = std::fs::write(dir.join("transcript.txt"), &text);
                Some(text)
            }
            None => None,
        };
        let span = if req.minutes.fract() == 0.0 {
            format!("{} min", req.minutes as u32)
        } else {
            format!("{} s", (req.minutes * 60.0).round() as u32)
        };
        let what = match (req.screen, audio) {
            (true, Some(a)) => format!("tela + {}", audio_label(a)),
            (true, None) => "tela".to_string(),
            (false, Some(a)) => audio_label(a).to_string(),
            (false, None) => String::new(),
        };
        let label = format!("Últimos {span} · {what}");
        let mut chip = if req.screen {
            let mut caption = format!(
                "Recorte dos últimos {span} ({what}).\n{}",
                labels.join("\n")
            );
            if let Some(t) = &transcript {
                caption.push_str("\n\n");
                caption.push_str(t);
            }
            let preview = frames.first().cloned();
            let mut chip = ContextChip::new(
                ChipKind::Clip,
                label,
                ChipPayload::Images {
                    paths: frames,
                    caption: Some(caption),
                },
            );
            chip.preview_path = preview;
            chip
        } else {
            ContextChip::new(
                ChipKind::Audio,
                label,
                ChipPayload::Text {
                    text: transcript.unwrap_or_default(),
                },
            )
        };
        chip.files_dir = Some(dir);
        self.add_chip(tray, chip)
    }

    /// Freezes the monitor of the previous app for region selection (004
    /// TK-002). The frame is policy-checked and redacted like any capture.
    pub async fn region_begin(&self) -> HostResult<FrozenScreen> {
        let app = self
            .platform
            .foreground
            .current()
            .ok_or_else(|| HostError::new("capture", "nenhuma janela ativa"))?;
        let area = self
            .platform
            .foreground
            .monitor_area(&app.monitor_id)
            .ok_or_else(|| HostError::new("capture", "monitor não encontrado"))?;
        let target = CapTarget::Monitor {
            id: app.monitor_id.clone(),
            area,
        };
        let policy = self.policy.read().unwrap().clone();
        let grants = self.grants.read().unwrap().clone();
        let platform = self.platform.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            capture_with_policy(
                &policy,
                &grants,
                Requester::User,
                &target,
                platform.frames.as_ref(),
                platform.inventory.as_ref(),
            )
        })
        .await
        .map_err(|e| HostError::new("capture", e.to_string()))??;
        let frame = match outcome {
            CaptureOutcome::Captured { frame, .. } => frame,
            CaptureOutcome::Denied { reason } => {
                return Err(HostError::new(
                    reason.code(),
                    "captura bloqueada pela privacidade",
                ));
            }
            CaptureOutcome::NeedsPermission => {
                return Err(HostError::new("permission", "permissão necessária"));
            }
        };
        let token = uuid::Uuid::new_v4().simple().to_string();
        let path = self
            .paths
            .captures_tmp()
            .join(format!("frozen-{token}.png"));
        std::fs::write(&path, frame.to_png())?;
        let fs = FrozenScreen {
            token: token.clone(),
            path,
            width: frame.width,
            height: frame.height,
            area,
        };
        self.frozen.lock().unwrap().insert(token, frame);
        Ok(fs)
    }

    /// Crops the frozen screen to `rect` (image pixels) and adds a Region chip.
    pub fn region_commit(
        &self,
        token: &str,
        rect: aura_core::placement::Rect,
        tray: &str,
    ) -> HostResult<ContextChip> {
        let frame = self
            .frozen
            .lock()
            .unwrap()
            .remove(token)
            .ok_or_else(|| HostError::new("not_found", "seleção expirada"))?;
        let _ = std::fs::remove_file(
            self.paths
                .captures_tmp()
                .join(format!("frozen-{token}.png")),
        );
        if rect.w < 4 || rect.h < 4 {
            return Err(HostError::new("invalid", "região pequena demais"));
        }
        let crop = frame
            .crop(rect)
            .ok_or_else(|| HostError::new("invalid", "região fora da tela"))?;
        let crop = crop.downscale(crate::tools::MAX_IMAGE_SIDE);
        let path = self
            .paths
            .captures_tmp()
            .join(format!("{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&path, crop.to_png())?;
        let mut chip = ContextChip::new(
            ChipKind::Region,
            format!("Região {}×{}", rect.w, rect.h),
            ChipPayload::Image { path: path.clone() },
        );
        chip.preview_path = Some(path);
        self.add_chip(tray, chip)
    }

    /// Reads the text inside `rect` of the frozen screen (OCR) and copies it
    /// to the clipboard; nothing is attached to the conversation (020).
    pub fn region_copy_text(
        &self,
        token: &str,
        rect: aura_core::placement::Rect,
    ) -> HostResult<String> {
        let frame = self
            .frozen
            .lock()
            .unwrap()
            .remove(token)
            .ok_or_else(|| HostError::new("not_found", "seleção expirada"))?;
        let _ = std::fs::remove_file(
            self.paths
                .captures_tmp()
                .join(format!("frozen-{token}.png")),
        );
        if rect.w < 4 || rect.h < 4 {
            return Err(HostError::new("invalid", "região pequena demais"));
        }
        let crop = frame
            .crop(rect)
            .ok_or_else(|| HostError::new("invalid", "região fora da tela"))?;
        let language = match self.settings().language {
            aura_core::settings::Language::PtBr => "pt-BR",
            aura_core::settings::Language::En => "en-US",
        };
        let text = self
            .platform
            .screen_text
            .ocr(&crop, language)
            .map_err(|e| HostError::new("ocr", e.to_string()))?;
        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(HostError::new("empty", "não encontrei texto nessa região"));
        }
        if !self.platform.foreground.set_clipboard(&text) {
            return Err(HostError::new(
                "clipboard",
                "não consegui copiar para a área de transferência",
            ));
        }
        Ok(text)
    }

    pub fn region_cancel(&self, token: &str) {
        self.frozen.lock().unwrap().remove(token);
        let _ = std::fs::remove_file(
            self.paths
                .captures_tmp()
                .join(format!("frozen-{token}.png")),
        );
    }

    /// Selected text of the previous app into a chip (respects exclusions).
    /// Selection of the previous app as the tray's single selection chip
    /// (012 AC-001). A new text replaces the chip; the same text is not
    /// duplicated; a chip the user removed only comes back when `explicit`
    /// (`@seleção`) or after the selection changes.
    pub fn capture_selection(&self, tray: &str, explicit: bool) -> HostResult<Option<ContextChip>> {
        let policy = self.policy.read().unwrap().clone();
        if policy.paused {
            return Ok(None);
        }
        let Some(app) = self.platform.foreground.current() else {
            return Ok(None);
        };
        if let Some(info) = self.platform.inventory.window(app.window)
            && policy
                .exclusions
                .iter()
                .any(|r| r.enabled && r.matches(&info))
        {
            return Ok(Some(self.add_chip(
                tray,
                ContextChip::blocked(ChipKind::Selection, "Seleção", "excluded"),
            )?));
        }
        let Some(text) = self.platform.foreground.selection(50_000) else {
            return Ok(None);
        };
        let mut trays = self.trays.lock().unwrap();
        let t = trays.entry(tray.to_string()).or_default();
        let current: Vec<ContextChip> = t
            .list()
            .iter()
            .filter(|c| c.kind == ChipKind::Selection)
            .cloned()
            .collect();
        if let Some(same) = current
            .iter()
            .find(|c| matches!(&c.payload, ChipPayload::Text { text: t } if *t == text))
        {
            return Ok(Some(same.clone()));
        }
        let mut last = self.last_selection.lock().unwrap();
        if !explicit && current.is_empty() && last.as_deref() == Some(text.as_str()) {
            // Removed by the user and still the same selection.
            return Ok(None);
        }
        for c in &current {
            let _ = t.remove(&c.id);
        }
        *last = Some(text.clone());
        *self.replace_target.lock().unwrap() = Some(text.clone());
        let preview: String = text.chars().take(40).collect();
        let chip = ContextChip::new(
            ChipKind::Selection,
            format!("❝ {preview}"),
            ChipPayload::Text { text },
        );
        Ok(Some(t.add(chip)?.clone()))
    }

    /// Attaches a file (drag-and-drop, file picker, paste).
    pub async fn attach(
        &self,
        tray: &str,
        thread_id: Option<&str>,
        path: &Path,
    ) -> HostResult<(AttachmentInfo, ContextChip)> {
        let (conv, ws) = match thread_id.and_then(|t| self.conversation_of(t)) {
            Some(c) => c,
            None => ("draft".to_string(), self.paths.captures_tmp().join("draft")),
        };
        let atts = self.attachments.clone();
        let path = path.to_path_buf();
        let (info, chip) = tokio::task::spawn_blocking(move || atts.add(&conv, &ws, &path))
            .await
            .map_err(|e| HostError::new("attachment", e.to_string()))??;
        let chip = self.add_chip(tray, chip)?;
        Ok((info, chip))
    }

    /// Attaches an image pasted from the clipboard (Ctrl+V; 002 AC-013,
    /// 007 AC-001): the bytes are stored as a file and follow `attach`.
    pub async fn attach_clipboard_image(
        &self,
        tray: &str,
        thread_id: Option<&str>,
        mime: &str,
        bytes: &[u8],
    ) -> HostResult<(AttachmentInfo, ContextChip)> {
        const MAX_BYTES: usize = 20 * 1024 * 1024;
        let ext = match mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => {
                return Err(HostError::new(
                    "unsupported",
                    format!("tipo de imagem não suportado: {mime}"),
                ));
            }
        };
        if bytes.len() > MAX_BYTES {
            return Err(HostError::new(
                "too_large",
                "a imagem colada passa de 20 MB",
            ));
        }
        let dir = self.paths.captures_tmp().join("pasted");
        std::fs::create_dir_all(&dir).map_err(|e| HostError::new("attachment", e.to_string()))?;
        let stamp = chrono_like_stamp();
        let path = dir.join(format!("clipboard-{stamp}.{ext}"));
        std::fs::write(&path, bytes).map_err(|e| HostError::new("attachment", e.to_string()))?;
        self.attach(tray, thread_id, &path).await
    }

    /// Selective read of an attachment (same path as the `attachment_read` tool).
    pub async fn read_attachment(
        &self,
        thread_id: &str,
        id: &str,
        selector: aura_ingest::Selector,
    ) -> HostResult<String> {
        let (uuid, _) = self
            .conversation_of(thread_id)
            .ok_or_else(|| HostError::new("not_found", "conversa não encontrada"))?;
        let atts = self.attachments.clone();
        let id = id.to_string();
        let blocks = tokio::task::spawn_blocking(move || atts.read(&uuid, &id, &selector))
            .await
            .map_err(|e| HostError::new("attachment", e.to_string()))??;
        Ok(crate::attachments::blocks_to_text(&blocks))
    }

    /// Files generated in the conversation workspace (008 TK-005), newest first.
    pub fn workspace_files(&self, thread_id: &str) -> Vec<WorkspaceFile> {
        fn walk(root: &Path, dir: &Path, depth: usize, out: &mut Vec<WorkspaceFile>) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for e in rd.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().into_owned();
                if name.starts_with('.')
                    || (depth == 0 && name == "attachments")
                    || out.len() >= 500
                {
                    continue;
                }
                let Ok(meta) = e.metadata() else { continue };
                if meta.is_dir() {
                    if depth < 4 {
                        walk(root, &p, depth + 1, out);
                    }
                } else {
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    out.push(WorkspaceFile {
                        path: p
                            .strip_prefix(root)
                            .unwrap_or(&p)
                            .to_string_lossy()
                            .replace('\\', "/"),
                        absolute: p.clone(),
                        bytes: meta.len(),
                        modified,
                    });
                }
            }
        }
        let Some((_, ws)) = self.conversation_of(thread_id) else {
            return vec![];
        };
        let mut out = Vec::new();
        walk(&ws, &ws, 0, &mut out);
        out.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.path.cmp(&b.path)));
        out
    }

    /// Text of a workspace file for preview (≤ 512 KB); refuses paths
    /// outside the workspace.
    /// A file inside the conversation workspace (never outside it).
    fn workspace_path(&self, thread_id: &str, rel: &str) -> HostResult<PathBuf> {
        let (_, ws) = self
            .conversation_of(thread_id)
            .ok_or_else(|| HostError::new("not_found", "conversa não encontrada"))?;
        let root = ws.canonicalize()?;
        let path = root.join(rel).canonicalize()?;
        if !path.starts_with(&root) {
            return Err(HostError::new("invalid", "caminho fora do workspace"));
        }
        Ok(path)
    }

    /// Text of the first pages of a workspace PDF, for the Files preview
    /// (008 AC-014). Scanned PDFs have no text layer: `pages` is empty.
    pub fn workspace_pdf_preview(&self, thread_id: &str, rel: &str) -> HostResult<PdfPreview> {
        const PAGES: u32 = 5;
        let path = self.workspace_path(thread_id, rel)?;
        if std::fs::metadata(&path)?.len() > 50 * 1024 * 1024 {
            return Err(HostError::new(
                "too_large",
                "arquivo grande demais para prévia",
            ));
        }
        let text = aura_ingest::pdf::extract(&std::fs::read(&path)?, Some((1, PAGES)))
            .map_err(|e| HostError::new("pdf", e.to_string()))?;
        Ok(PdfPreview {
            total_pages: text.total_pages,
            pages: text
                .pages
                .into_iter()
                .filter(|(_, t)| !t.trim().is_empty())
                .map(|(number, text)| PdfPage { number, text })
                .collect(),
        })
    }

    pub fn read_workspace_file(&self, thread_id: &str, rel: &str) -> HostResult<String> {
        let path = self.workspace_path(thread_id, rel)?;
        let meta = std::fs::metadata(&path)?;
        if meta.len() > 512 * 1024 {
            return Err(HostError::new(
                "too_large",
                "arquivo grande demais para prévia",
            ));
        }
        Ok(aura_ingest::text::decode(&std::fs::read(&path)?))
    }

    pub fn attachments(&self, thread_id: &str) -> Vec<AttachmentInfo> {
        self.conversation_of(thread_id)
            .map(|(uuid, _)| self.attachments.list(&uuid))
            .unwrap_or_default()
    }

    /// Saves a note or an answer the user starred (020).
    pub fn note_add(&self, kind: &str, text: &str) -> HostResult<()> {
        self.notes
            .add(kind, text, self.clock.now())
            .map(|_| ())
            .map_err(|e| HostError::new("invalid", e.to_string()))
    }

    /// The text "Substituir seleção" would replace (shown as the "before").
    pub fn replace_target(&self) -> Option<String> {
        self.replace_target.lock().unwrap().clone()
    }

    /// Replaces the selection of the previous app with `text` (019). Returns
    /// the original text. Fails without a remembered selection.
    pub fn replace_selection(&self, text: &str) -> HostResult<String> {
        let original = self.replace_target().ok_or_else(|| {
            HostError::new("no_selection", "não há texto selecionado para substituir")
        })?;
        if text.trim().is_empty() {
            return Err(HostError::new("invalid", "texto vazio"));
        }
        if let Some(app) = self.platform.foreground.current() {
            self.platform.foreground.restore(&app);
        }
        if !self.platform.foreground.paste(text) {
            return Err(HostError::new("paste", "não consegui colar no app"));
        }
        *self.replaced.lock().unwrap() = true;
        Ok(original)
    }

    /// Takes back the last replacement (Ctrl+Z in the app).
    pub fn undo_replace(&self) -> bool {
        if !std::mem::take(&mut *self.replaced.lock().unwrap()) {
            return false;
        }
        if let Some(app) = self.platform.foreground.current() {
            self.platform.foreground.restore(&app);
        }
        self.platform.foreground.undo()
    }

    /// Dictated or typed text into the previous app (006 "inserir no app").
    pub fn insert_into_app(&self, text: &str) -> bool {
        if let Some(app) = self.platform.foreground.current() {
            self.platform.foreground.restore(&app);
        }
        self.platform.foreground.paste(text)
    }

    // ---------------------------------------------------------- quick commands

    pub fn quick_commands(&self) -> HostResult<Vec<QuickCommand>> {
        let mut commands = self.quick.list()?;
        let language = self.settings().language;
        for command in commands.iter_mut().filter(|c| c.builtin) {
            let key = match command.name.as_str() {
                "tldr" => "quick.template.tldr",
                "traduzir" => "quick.template.translate",
                "reescrever" => "quick.template.rewrite",
                "explicar" => "quick.template.explain",
                "corrigir" => "quick.template.correct",
                "resumir-tela" => "quick.template.screen",
                "formal" => "quick.template.formal",
                "curto" => "quick.template.short",
                "amigavel" => "quick.template.friendly",
                "golpe" => "quick.template.scam",
                "responder" => "quick.template.reply",
                "parei" => "quick.template.resume",
                "lembrar" => "quick.template.remind",
                "anota" => "quick.template.note",
                "notas" => "quick.template.notes",
                "colar" => "quick.template.pasteas",
                "salvos" => "quick.template.saved",
                "configurar" => "quick.template.configure",
                "preparo" => "quick.template.prepare",
                _ => continue,
            };
            command.template = crate::localization::text(language, key).into();
        }
        Ok(commands)
    }

    pub fn save_quick_command(
        &self,
        name: &str,
        template: &str,
        replace: bool,
    ) -> HostResult<Vec<QuickCommand>> {
        self.quick.save(name, template, replace)?;
        self.quick_commands()
    }

    pub fn delete_quick_command(&self, name: &str) -> HostResult<Vec<QuickCommand>> {
        self.quick.delete(name)?;
        self.quick_commands()
    }

    pub fn toggle_quick_command(&self, name: &str, enabled: bool) -> HostResult<Vec<QuickCommand>> {
        self.quick.set_enabled(name, enabled)?;
        self.quick_commands()
    }

    /// Expands `/cmd args` with the tray selection; attaches the screen if the
    /// command asks for it.
    pub async fn expand_quick_command(
        &self,
        tray: &str,
        input: &str,
        typed: &str,
    ) -> HostResult<Expansion> {
        let chips = self.tray(tray);
        let selection = chips.iter().find_map(|c| match (&c.kind, &c.payload) {
            (ChipKind::Selection, ChipPayload::Text { text }) if c.blocked_reason.is_none() => {
                Some(text.clone())
            }
            _ => None,
        });
        let ctx = QuickContext {
            selection,
            typed: typed.to_string(),
            has_chips: chips.iter().any(|c| c.kind != ChipKind::Selection),
            clipboard: None,
        };
        let (name, tail) = aura_extensions::quick::parse_invocation(input)
            .ok_or_else(|| aura_extensions::quick::QuickError::Unknown(input.into()))?;
        let command = self
            .quick_commands()?
            .into_iter()
            .find(|c| c.name == name)
            .ok_or_else(|| aura_extensions::quick::QuickError::Unknown(name.into()))?;
        // The clipboard is read only for commands that ask for it (`/colar`).
        let ctx = if command.template.contains(aura_extensions::quick::AREA_MARK) {
            QuickContext {
                clipboard: self.platform.foreground.clipboard_text(),
                ..ctx
            }
        } else {
            ctx
        };
        let exp = aura_extensions::quick::expand(&command, tail, &ctx)?;
        if exp.needs_screen && !chips.iter().any(|c| c.kind == ChipKind::Screen) {
            self.capture_screen(tray, false).await?;
        }
        // The selection is inlined in the prompt; don't send it twice. It may
        // come back on the next return to the Overlay.
        if let Some(selected) = &ctx.selection {
            *self.replace_target.lock().unwrap() = Some(selected.clone());
            *self.last_selection.lock().unwrap() = None;
            let mut trays = self.trays.lock().unwrap();
            if let Some(t) = trays.get_mut(tray) {
                let ids: Vec<String> = t
                    .list()
                    .iter()
                    .filter(|c| c.kind == ChipKind::Selection)
                    .map(|c| c.id.clone())
                    .collect();
                for id in ids {
                    let _ = t.remove(&id);
                }
            }
        }
        Ok(exp)
    }

    // ------------------------------------------------------------- MCP servers

    pub fn mcp_servers(&self) -> HostResult<Vec<McpServerSpec>> {
        Ok(self.mcp_servers.list()?)
    }

    fn refresh_mcp(&self) -> HostResult<()> {
        self.launch_state.write().unwrap().mcp_servers = self.mcp_servers.list()?;
        Ok(())
    }

    /// Saves a server; secrets (`VAR → value`, bearer) go to the vault.
    /// Takes effect on the next app-server start ([`Self::restart_agent`]).
    pub fn save_mcp_server(
        &self,
        spec: McpServerSpec,
        secrets: Vec<(String, String)>,
        bearer: Option<String>,
    ) -> HostResult<Vec<McpServerSpec>> {
        let secrets: Vec<(String, Secret<String>)> = secrets
            .into_iter()
            .map(|(k, v)| (k, Secret::new(v)))
            .collect();
        let bearer = bearer.map(Secret::new);
        self.mcp_servers.save(
            &spec,
            &secrets,
            bearer.as_ref(),
            self.platform.credentials.as_ref(),
        )?;
        self.refresh_mcp()?;
        self.mcp_servers()
    }

    pub fn delete_mcp_server(&self, name: &str) -> HostResult<Vec<McpServerSpec>> {
        self.mcp_servers
            .delete(name, self.platform.credentials.as_ref())?;
        self.refresh_mcp()?;
        self.mcp_servers()
    }

    pub fn detect_mcp_imports(&self) -> Vec<DetectedServer> {
        aura_extensions::import::default_roots()
            .map(|r| aura_extensions::import::detect(&r))
            .unwrap_or_default()
    }

    pub fn detect_mcp_imports_in(&self, roots: &Roots) -> Vec<DetectedServer> {
        aura_extensions::import::detect(roots)
    }

    /// Imports the chosen servers (by name) from a fresh detection.
    pub fn import_mcp_servers(
        &self,
        detected: Vec<DetectedServer>,
        names: &[String],
    ) -> HostResult<Vec<McpServerSpec>> {
        for d in detected
            .into_iter()
            .filter(|d| names.contains(&d.spec.name))
        {
            self.mcp_servers.save(
                &d.spec,
                &d.secrets,
                d.bearer.as_ref(),
                self.platform.credentials.as_ref(),
            )?;
        }
        self.refresh_mcp()?;
        self.mcp_servers()
    }

    /// Live status of MCP servers (tools, auth) from the app-server.
    pub async fn mcp_status(&self) -> HostResult<Vec<McpStatus>> {
        Ok(parse_mcp_status(&self.codex.mcp_status().await?))
    }

    /// Starts a stdio server the way the app-server would and returns whether
    /// it answered `initialize` plus its last stderr lines (008 AC-005).
    pub async fn mcp_diagnose(&self, name: &str) -> HostResult<crate::mcp_diag::McpDiagnosis> {
        let spec = self
            .mcp_servers()?
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| HostError::new("not_found", "servidor MCP não encontrado"))?;
        let aura_extensions::mcp_config::Transport::Stdio {
            command,
            args,
            env,
            cwd,
        } = &spec.transport
        else {
            return Err(HostError::new(
                "unsupported",
                "diagnóstico de log disponível para servidores stdio",
            ));
        };
        let mut vars: Vec<(String, String)> = env
            .iter()
            .filter_map(|(k, v)| match v {
                aura_extensions::mcp_config::EnvValue::Plain { value } => {
                    Some((k.clone(), value.clone()))
                }
                aura_extensions::mcp_config::EnvValue::Secret => None,
            })
            .collect();
        if let Ok(secrets) = aura_extensions::mcp_config::secret_env(
            std::slice::from_ref(&spec),
            self.platform.credentials.as_ref(),
        ) {
            vars.extend(secrets.into_iter().map(|(k, v)| (k, v.expose().clone())));
        }
        crate::mcp_diag::diagnose(
            crate::mcp_diag::StdioLaunch {
                command: command.clone(),
                args: args.clone(),
                env: vars,
                cwd: cwd.as_ref().map(PathBuf::from),
            },
            std::time::Duration::from_secs(10),
        )
        .await
        .map_err(|e| HostError::new("mcp", format!("{command}: {e}")))
    }

    /// OAuth for an HTTP MCP server: returns the URL to open in the browser.
    pub async fn mcp_login(&self, name: &str) -> HostResult<Option<String>> {
        Ok(self.codex.mcp_oauth_login(name).await?)
    }

    /// Stops the app-server when idle so config changes apply on next use.
    pub async fn restart_agent(&self) -> bool {
        if !self.codex.pending_request_ids().is_empty() {
            return false;
        }
        self.codex.supervisor().stop().await;
        true
    }

    // ---------------------------------------------------------------- memories

    pub fn memories(&self) -> HostResult<MemoryView> {
        Ok(crate::memories::read(
            &self.paths.codex_home().join("memories"),
        )?)
    }

    pub fn forget_memory_fact(&self, fact: &str) -> HostResult<MemoryView> {
        let dir = self.paths.codex_home().join("memories");
        crate::memories::forget_fact(&dir, fact)?;
        Ok(crate::memories::read(&dir)?)
    }

    pub fn save_memories(&self, summary: &str, registry: &str) -> HostResult<MemoryView> {
        let dir = self.paths.codex_home().join("memories");
        crate::memories::save(&dir, summary, registry)?;
        Ok(crate::memories::read(&dir)?)
    }

    pub fn forget_all_memories(&self) -> HostResult<()> {
        Ok(crate::memories::forget_all(
            &self.paths.codex_home().join("memories"),
        )?)
    }

    // ------------------------------------------------------------------ skills

    pub fn review_skill(&self, path: &Path) -> HostResult<SkillReview> {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
        {
            Ok(skills::review_zip(std::fs::read(path)?)?)
        } else {
            Ok(skills::review_dir(path)?)
        }
    }

    pub fn install_skill(&self, path: &Path, overwrite: bool) -> HostResult<PathBuf> {
        let source = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
        {
            SkillSource::Zip(std::fs::read(path)?)
        } else {
            SkillSource::Dir(path.to_path_buf())
        };
        Ok(skills::install(&source, &self.paths.skills(), overwrite)?)
    }

    pub fn create_skill(&self, name: &str, description: &str, body: &str) -> HostResult<PathBuf> {
        reserved_skill_name(name)?;
        Ok(skills::create(
            &self.paths.skills(),
            name,
            description,
            body,
        )?)
    }

    /// Aura-managed skills (Codex lists all roots itself via `skills/list`).
    pub fn skills(&self) -> Vec<SkillReview> {
        let Ok(rd) = std::fs::read_dir(self.paths.skills()) else {
            return vec![];
        };
        let mut out: Vec<SkillReview> = rd
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| skills::review_dir(&e.path()).ok())
            .collect();
        out.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        out
    }

    /// Every skill new conversations can use, with origin and on/off state
    /// (008 AC-001); Codex owns the enabled flag.
    pub async fn skills_catalog(&self) -> HostResult<Vec<SkillEntry>> {
        let aura_root = self.paths.skills();
        let core_root = self.paths.core_skills();
        let _ = std::fs::create_dir_all(&aura_root);
        Ok(self
            .codex
            .skills()
            .await?
            .into_iter()
            // Core skills belong to the agent, not to the user (017).
            .filter(|s| !s.path.starts_with(&core_root))
            .map(|s| {
                let origin = if s.path.starts_with(&aura_root) {
                    SkillOrigin::Aura
                } else if s.scope == "user" {
                    SkillOrigin::User
                } else {
                    SkillOrigin::System
                };
                SkillEntry {
                    name: s.name,
                    description: s.description,
                    path: s.path,
                    origin,
                    enabled: s.enabled,
                }
            })
            .collect())
    }

    pub async fn set_skill_enabled(&self, path: &Path, enabled: bool) -> HostResult<()> {
        if path.starts_with(self.paths.core_skills()) {
            return Err(HostError::new("skill", "esta Skill é interna do Aura"));
        }
        self.codex.set_skill_enabled(path, enabled).await?;
        // Codex wrote its config.toml, which Aura regenerates on every start:
        // keep the choice in Aura's settings too.
        let key = path.to_string_lossy().into_owned();
        let mut next = self.settings();
        next.disabled_skills.retain(|p| *p != key);
        if !enabled {
            next.disabled_skills.push(key);
        }
        SettingsRepo::new(&self.store).save(&next)?;
        *self.settings.write().unwrap() = next.clone();
        self.apply_runtime_settings(&next);
        Ok(())
    }

    /// Description and instructions of an Aura skill, for the editor.
    pub fn skill_source(&self, name: &str) -> HostResult<(String, String)> {
        Ok(skills::read_editable(&self.paths.skills(), name)?)
    }

    pub fn update_skill(&self, name: &str, description: &str, body: &str) -> HostResult<()> {
        Ok(skills::update(
            &self.paths.skills(),
            name,
            description,
            body,
        )?)
    }

    pub fn delete_skill(&self, name: &str) -> HostResult<()> {
        skills::validate_name(name)?;
        let dir = self.paths.skills().join(name);
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------- voice

    fn ui_lang(&self) -> &'static str {
        match self.settings().language {
            aura_core::settings::Language::PtBr => "pt-BR",
            aura_core::settings::Language::En => "en",
        }
    }

    pub fn voice_models(&self) -> Vec<ModelView> {
        self.voice.catalog(self.ui_lang())
    }

    pub fn install_voice_model(&self, id: &str) -> HostResult<()> {
        self.voice.install(id).map_err(|e| HostError::new("asr", e))
    }

    pub fn cancel_voice_model(&self, id: &str) {
        self.voice.cancel_install(id);
    }

    pub fn remove_voice_model(&self, id: &str) -> HostResult<()> {
        self.voice
            .remove(id)
            .map_err(|e| HostError::new("asr", e))?;
        if self.voice.selected().is_none() {
            SettingsRepo::new(&self.store).set_raw(ASR_MODEL_KEY, &serde_json::Value::Null)?;
        }
        Ok(())
    }

    pub fn select_voice_model(&self, id: &str) -> HostResult<()> {
        self.voice
            .select(id)
            .map_err(|e| HostError::new("asr", e))?;
        SettingsRepo::new(&self.store)
            .set_raw(ASR_MODEL_KEY, &serde_json::Value::String(id.into()))?;
        Ok(())
    }

    /// Push-to-talk down. The microphone must not be off or paused.
    pub async fn ptt_press(&self, device: Option<String>) -> HostResult<()> {
        let p = self.policy.read().unwrap().clone();
        if p.paused {
            return Err(HostError::new("paused", "a privacidade está pausada"));
        }
        if p.mic.mode == CaptureMode::Off {
            return Err(HostError::new(
                "mic_off",
                "o microfone está desligado nas configurações de privacidade",
            ));
        }
        let sel = device
            .or_else(|| self.settings().microphone_device_id)
            .map(DeviceSel::Id)
            .unwrap_or(DeviceSel::Default);
        let fallback = self.voice.press(sel).await.map_err(|e| {
            if e == "model_missing" || e == "worker_missing" {
                let _ = self
                    .events
                    .send(HostEvent::Voice(PttState::Failed { error: e.clone() }));
            }
            HostError::new("asr", e)
        })?;
        if let Some(wanted) = fallback {
            let message = crate::localization::text(self.settings().language, "audio.fallback")
                .replace("{name}", &wanted);
            self.notice("warning", message);
        }
        Ok(())
    }

    /// Settings (language, vocabulary) apply when the caller passes none.
    pub async fn ptt_release(&self, language: Option<String>, vocabulary: Vec<String>) -> PttState {
        let s = self.settings();
        let language = s.asr_language.clone().or(language);
        let vocabulary = if vocabulary.is_empty() {
            s.asr_vocabulary.clone()
        } else {
            vocabulary
        };
        self.voice
            .release(AsrOptions {
                language,
                vocabulary,
            })
            .await
    }

    pub async fn ptt_cancel(&self) -> PttState {
        self.voice.cancel().await
    }

    /// Starts a live level meter for source (005 AC-001): AudioLevel
    /// events at about 30 Hz with the peak of each window, until stopped.
    /// Nothing is recorded. A missing chosen device falls back to the OS
    /// default with a warning (005 AC-002).
    pub async fn audio_test_start(
        &self,
        source: aura_audio::AudioSourceKind,
        device: Option<String>,
    ) -> HostResult<()> {
        if self.policy.read().unwrap().paused {
            return Err(HostError::new("paused", "a privacidade está pausada"));
        }
        self.audio_test_stop(source).await;
        let settings = self.settings();
        let saved = match source {
            aura_audio::AudioSourceKind::Mic => settings.microphone_device_id,
            aura_audio::AudioSourceKind::SystemAudio => settings.system_audio_device_id,
        };
        let sel = device
            .or(saved)
            .map(DeviceSel::Id)
            .unwrap_or(DeviceSel::Default);
        let audio = self.platform.audio.clone();
        let hub = tokio::task::spawn_blocking(move || {
            aura_audio::hub::AudioHub::start(audio, source, sel)
        })
        .await
        .map_err(|e| HostError::new("audio", e.to_string()))?
        .map_err(|e| HostError::new("audio", e.to_string()))?;
        if let Some(wanted) = hub.fallback_from() {
            let message = crate::localization::text(settings.language, "audio.fallback")
                .replace("{name}", wanted);
            self.notice("warning", message);
        }
        let mut rx = hub.subscribe();
        let events = self.events.clone();
        let task = tokio::spawn(async move {
            const WINDOW: std::time::Duration = std::time::Duration::from_millis(30);
            let mut peak = f32::NEG_INFINITY;
            let mut since = tokio::time::Instant::now();
            loop {
                // A silent loopback device delivers no chunks: the window
                // still closes, reporting silence instead of freezing.
                match tokio::time::timeout(WINDOW, rx.recv()).await {
                    Ok(Ok(chunk)) => peak = peak.max(chunk.level_dbfs),
                    Ok(Err(broadcast::error::RecvError::Lagged(_))) | Err(_) => {}
                    Ok(Err(broadcast::error::RecvError::Closed)) => break,
                }
                if since.elapsed() >= WINDOW {
                    let _ = events.send(HostEvent::AudioLevel {
                        source,
                        dbfs: peak.max(-100.0),
                    });
                    peak = f32::NEG_INFINITY;
                    since = tokio::time::Instant::now();
                }
            }
        });
        self.audio_tests
            .lock()
            .unwrap()
            .insert(source, AudioTest { hub, task });
        Ok(())
    }

    pub async fn audio_test_stop(&self, source: aura_audio::AudioSourceKind) {
        let test = self.audio_tests.lock().unwrap().remove(&source);
        if let Some(AudioTest { hub, task }) = test {
            task.abort();
            // Stopping joins the device thread.
            let _ = tokio::task::spawn_blocking(move || drop(hub)).await;
        }
    }

    pub fn audio_devices(&self, system: bool) -> Vec<AudioDevice> {
        let kind = if system {
            aura_audio::AudioSourceKind::SystemAudio
        } else {
            aura_audio::AudioSourceKind::Mic
        };
        self.platform.audio.devices(kind)
    }

    // ------------------------------------------------------------ app-server

    /// Downloads (first run) and verifies the pinned app-server; progress
    /// arrives as `Download { id: "codex" }` events.
    pub async fn ensure_codex(&self) -> HostResult<PathBuf> {
        let tx = self.events.clone();
        let progress = move |bytes: u64, total: Option<u64>| {
            let _ = tx.send(HostEvent::Download {
                id: "codex".into(),
                bytes,
                total,
                done: false,
                error: None,
            });
        };
        match aura_codex::binary::ensure(&self.paths.bin(), &aura_codex::binary::pinned(), progress)
            .await
        {
            Ok(path) => {
                *self.codex_program.write().unwrap() = Some(path.clone());
                let _ = self.events.send(HostEvent::Download {
                    id: "codex".into(),
                    bytes: 0,
                    total: None,
                    done: true,
                    error: None,
                });
                Ok(path)
            }
            Err(e) => {
                let _ = self.events.send(HostEvent::Download {
                    id: "codex".into(),
                    bytes: 0,
                    total: None,
                    done: false,
                    error: Some(e.to_string()),
                });
                Err(HostError::new("codex_install", e.to_string()))
            }
        }
    }

    /// Snapshot of the foreground app right before the Overlay shows.
    pub fn prepare_overlay(&self) -> Option<aura_core::events::PreviousApp> {
        self.platform.foreground.snapshot();
        self.platform.foreground.current()
    }

    // -------------------------------------------------------------- recordings

    pub async fn recording_start(&self, title: &str) -> HostResult<String> {
        let policy = self.policy.read().unwrap().clone();
        let id = self
            .capture
            .start_manual(title, &policy)
            .await
            .map_err(|e| HostError::new("recording", e))?;
        self.reconcile_capture();
        Ok(id)
    }

    pub async fn recording_stop(&self) -> Option<String> {
        let id = self.capture.stop_manual().await;
        self.reconcile_capture();
        id
    }

    /// Background capture service (recent buffers, recordings).
    pub fn capture(&self) -> Arc<crate::recorder::CaptureService> {
        self.capture.clone()
    }

    pub fn recordings(&self) -> Vec<crate::recorder::Recording> {
        self.capture.recordings()
    }

    pub fn recording_active(&self) -> Option<String> {
        self.capture.manual_active()
    }

    pub fn recording_delete(&self, id: &str) -> HostResult<()> {
        self.capture
            .delete_recording(id)
            .map_err(|e| HostError::new("recording", e))?;
        let _ = std::fs::remove_dir_all(self.playback_dir(id));
        Ok(())
    }

    fn playback_dir(&self, id: &str) -> PathBuf {
        // Recording ids are UUIDs; anything else must not reach the path.
        let safe: String = id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        self.paths.captures_tmp().join("playback").join(safe)
    }

    /// Decrypted copies of a finished recording for the Aura player (and for
    /// attaching), in the session cache: audio WAV per source, screen MP4
    /// segments (QA-032).
    pub fn recording_playback(&self, id: &str) -> HostResult<Vec<PlaybackMedia>> {
        let rec = self
            .capture
            .recordings()
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| HostError::new("not_found", "gravação não encontrada"))?;
        if rec.ended_at.is_none() {
            return Err(HostError::new(
                "recording",
                "a gravação ainda está em andamento",
            ));
        }
        let dir = self.playback_dir(id);
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .map(|d| d.filter_map(|e| e.ok().map(|e| e.path())).collect())
            .unwrap_or_default();
        if files.is_empty() {
            files = self
                .capture
                .export_recording(id, &dir)
                .map_err(|e| HostError::new("recording", e))?;
        }
        files.sort();
        Ok(files.into_iter().filter_map(PlaybackMedia::of).collect())
    }

    /// Attaches a finished recording to a conversation tray like any file.
    pub async fn recording_attach(
        &self,
        id: &str,
        tray: &str,
        thread_id: Option<&str>,
    ) -> HostResult<RecordingAttach> {
        let mut out = RecordingAttach::default();
        for m in self.recording_playback(id)? {
            let name = m
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            match self.attach(tray, thread_id, &m.path).await {
                Ok((_, chip)) => out.chips.push(chip),
                Err(e) => out.failed.push(format!("{name}: {}", e.message)),
            }
        }
        if out.chips.is_empty()
            && let Some(first) = out.failed.first()
        {
            return Err(HostError::new("attachment", first.clone()));
        }
        Ok(out)
    }

    /// Decrypts a recording into `dir` (defaults to Documents/Aura).
    pub fn recording_export(&self, id: &str, dir: &Path) -> HostResult<Vec<PathBuf>> {
        self.capture
            .export_recording(id, dir)
            .map_err(|e| HostError::new("recording", e))
    }

    // ---------------------------------------------------------------- profiles

    pub fn profiles(&self) -> HostResult<Vec<crate::profiles::AppProfile>> {
        Ok(self.profiles.list()?)
    }

    pub fn save_profile(
        &self,
        p: crate::profiles::AppProfile,
    ) -> HostResult<crate::profiles::AppProfile> {
        Ok(self.profiles.save(p)?)
    }

    pub fn delete_profile(&self, id: &str) -> HostResult<()> {
        Ok(self.profiles.delete(id)?)
    }

    /// Profile for the app in front (badge, default mode, screen chip).
    pub fn active_profile(&self) -> Option<crate::profiles::AppProfile> {
        self.platform
            .foreground
            .current()
            .and_then(|a| self.profiles.for_app(&a))
    }

    // --------------------------------------------------------------- placement

    pub fn saved_placement(&self, monitor_id: &str, mode: OverlayMode) -> Option<SavedPlacement> {
        aura_store::placements::PlacementRepo::new(&self.store)
            .get(monitor_id, mode)
            .ok()
            .flatten()
    }

    pub fn save_placement(&self, placement: &SavedPlacement, mode: OverlayMode) -> HostResult<()> {
        Ok(aura_store::placements::PlacementRepo::new(&self.store).save(placement, mode)?)
    }

    /// The app the Overlay was opened over (tools, placement, insertion).
    pub fn previous_app(&self) -> Option<aura_core::events::PreviousApp> {
        self.platform.foreground.current()
    }

    // ----------------------------------------------------------------- speech

    /// Voices for reading answers: offline Windows voices and the BYOK
    /// providers that can speak (OpenAI preset or a custom compatible API).
    pub fn speech_options(&self) -> HostResult<SpeechOptions> {
        let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
        let tts_presets: Vec<String> = presets()
            .into_iter()
            .filter(|p| p.tts)
            .map(|p| p.id)
            .collect();
        let cloud = reg
            .list()?
            .into_iter()
            .filter(|p| p.preset == "custom" || tts_presets.contains(&p.preset))
            .map(|p| SpeechProvider {
                id: p.id,
                name: p.name,
            })
            .collect();
        Ok(SpeechOptions {
            voices: self.platform.speech.voices(),
            cloud,
        })
    }

    /// The user agreed to send answers to the chosen cloud voice provider.
    /// YOLO (018): turning it on needs the typed confirmation ("ACEITO" or
    /// "ACCEPT"); turning it off needs nothing.
    pub fn set_yolo(&self, enabled: bool, confirmation: &str) -> HostResult<Settings> {
        if enabled && !aura_core::settings::yolo_confirmed(confirmation) {
            return Err(HostError::new(
                "invalid",
                "para ligar o modo YOLO, escreva ACEITO (ou ACCEPT)",
            ));
        }
        let mut s = self.settings();
        s.yolo = enabled;
        tracing::warn!(enabled, "YOLO mode changed");
        self.replace_settings(s)
    }

    pub fn speech_consent(&self) -> HostResult<Settings> {
        let mut s = self.settings();
        let Some(id) = s.tts_provider.clone() else {
            return Err(HostError::new("invalid", "nenhuma voz na nuvem escolhida"));
        };
        if !s.tts_cloud_consent.contains(&id) {
            s.tts_cloud_consent.push(id);
        }
        // Drop consents of providers that no longer exist.
        let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
        s.tts_cloud_consent
            .retain(|c| reg.get(c).ok().flatten().is_some());
        self.replace_settings(s)
    }

    /// Speaks an answer: returns `(base64 audio, mime)` for the UI to play.
    /// Offline Windows voice by default; a cloud voice only after consent.
    pub async fn speak(&self, markdown: &str) -> HostResult<(String, String)> {
        use base64::Engine;
        let text = crate::speech::speakable(markdown);
        let settings = self.settings();
        let lang = match settings.language {
            aura_core::settings::Language::PtBr => "pt-BR",
            aura_core::settings::Language::En => "en-US",
        };
        let (bytes, mime) = match settings.tts_provider.clone() {
            Some(id) => self.speak_cloud(&id, &settings, &text).await?,
            None => {
                let speech = self.platform.speech.clone();
                let voice = settings.tts_voice.clone();
                tokio::task::spawn_blocking(move || {
                    speech.synthesize(&text, lang, voice.as_deref())
                })
                .await
                .map_err(|e| HostError::new("speech", e.to_string()))?
                .map_err(|e| HostError::new("speech", e))?
            }
        };
        Ok((
            base64::engine::general_purpose::STANDARD.encode(bytes),
            mime,
        ))
    }

    async fn speak_cloud(
        &self,
        id: &str,
        settings: &Settings,
        text: &str,
    ) -> HostResult<(Vec<u8>, String)> {
        if text.trim().is_empty() {
            return Err(HostError::new("speech", "nada para ler"));
        }
        let reg = ProviderRegistry::new(&self.store, self.platform.credentials.as_ref());
        let provider = reg
            .get(id)?
            .ok_or_else(|| HostError::new("speech", "provedor de voz não encontrado"))?;
        if !settings.tts_cloud_consent.iter().any(|c| c == id) {
            return Err(HostError::new("consent_required", provider.name.clone()));
        }
        let target = reg.target(&provider)?;
        let model = if provider.preset == "openai" {
            "gpt-4o-mini-tts"
        } else {
            "tts-1"
        };
        let mut req = self
            .http
            .post(format!(
                "{}/audio/speech",
                target.base_url.trim_end_matches('/')
            ))
            .json(&serde_json::json!({
                "model": model,
                "voice": settings.tts_cloud_voice,
                "input": text,
                "response_format": "wav",
            }));
        for (k, v) in &target.headers {
            req = req.header(k, v);
        }
        if let Some(key) = &target.credential {
            req = req.bearer_auth(key.expose());
        }
        let resp = req
            .send()
            .await
            .map_err(|e| HostError::new("speech", e.to_string()))?;
        if !resp.status().is_success() {
            return Err(HostError::new(
                "speech",
                format!("{} respondeu {}", provider.name, resp.status()),
            ));
        }
        let mime = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .filter(|m| m.starts_with("audio/"))
            .unwrap_or("audio/wav")
            .to_string();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| HostError::new("speech", e.to_string()))?;
        Ok((bytes.to_vec(), mime))
    }

    // ------------------------------------------------------------- diagnostics

    pub async fn diagnostics(&self) -> Diagnostics {
        let port = self.gateway.port;
        let reachable = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            tokio::net::TcpStream::connect(("127.0.0.1", port)),
        )
        .await
        .is_ok_and(|r| r.is_ok());
        let ready = matches!(
            self.codex.supervisor().status().await,
            AppServerState::Ready { .. }
        );
        let mcp = if ready {
            tokio::time::timeout(std::time::Duration::from_secs(3), self.mcp_status())
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or_default()
                .into_iter()
                .map(|m| McpDiag {
                    name: m.name,
                    tools: m.tools.len(),
                    error: m.error,
                })
                .collect()
        } else {
            Vec::new()
        };
        let account = self.auth.active().ok().flatten().map(|a| AccountDiag {
            email: a.email.as_deref().map(crate::diagnostics::mask_email),
            signed_in: a.signed_in,
            plan_usage_enabled: a.plan_usage_enabled,
        });
        let paused = self.policy.read().unwrap().paused;
        let root = self.paths.root.clone();
        let disk_space = self.disk.clone();
        let disk = tokio::task::spawn_blocking(move || DiskDiag {
            free_bytes: disk_space.free_bytes(&root),
            aura_bytes: crate::diagnostics::dir_size(&root),
        })
        .await
        .unwrap_or(DiskDiag {
            free_bytes: None,
            aura_bytes: 0,
        });
        Diagnostics {
            gateway: GatewayDiag { port, reachable },
            mcp,
            worker: self.voice.worker_diag().await,
            capture: CaptureDiag {
                paused,
                active: self.capture.active().await,
                recording: self.capture.manual_active().is_some(),
            },
            account,
            disk,
            version: env!("CARGO_PKG_VERSION").into(),
            os: self.platform.os.clone(),
            data_dir: self.paths.root.clone(),
            gateway_port: self.gateway.port,
            app_server: self.codex.supervisor().status().await,
            app_server_launches: self.codex.supervisor().launch_count(),
            providers: self.launch_state.read().unwrap().providers.len(),
            mcp_servers: self.launch_state.read().unwrap().mcp_servers.len(),
            pending_consents: self.consent.pending_ids().len(),
        }
    }

    /// Redacted support package (010 TK-006) at `dest` (a `.zip`).
    pub async fn export_diagnostics(&self, dest: &Path) -> HostResult<PathBuf> {
        let summary = serde_json::to_value(self.diagnostics().await).unwrap_or_default();
        let privacy = serde_json::to_value(self.privacy()).unwrap_or_default();
        let providers: Vec<serde_json::Value> = self
            .providers()
            .unwrap_or_default()
            .into_iter()
            .map(|p| {
                serde_json::json!({"id": p.id, "preset": p.preset, "wire": p.wire, "baseUrl": p.base_url,
                    "status": p.status, "lastError": p.last_error, "models": p.models.len(), "hasCredential": p.credential_hint.is_some()})
            })
            .collect();
        let mcp: Vec<serde_json::Value> = self
            .mcp_servers()
            .unwrap_or_default()
            .into_iter()
            .map(|s| {
                let transport = match &s.transport {
                    aura_extensions::mcp_config::Transport::Stdio { command, .. } => {
                        serde_json::json!({"type": "stdio", "command": command})
                    }
                    aura_extensions::mcp_config::Transport::Http { url, .. } => {
                        serde_json::json!({"type": "http", "url": url})
                    }
                };
                serde_json::json!({"name": s.name, "enabled": s.enabled, "transport": transport})
            })
            .collect();
        let input = crate::diagnostics::DiagnosticsInput {
            summary,
            settings: crate::diagnostics::settings_view(&self.settings()),
            privacy,
            providers: serde_json::Value::Array(providers),
            mcp: serde_json::Value::Array(mcp),
            logs_dir: self.paths.logs(),
        };
        let dest = dest.to_path_buf();
        Ok(
            tokio::task::spawn_blocking(move || crate::diagnostics::export(&input, &dest))
                .await
                .map_err(|e| HostError::new("io", e.to_string()))??,
        )
    }

    /// "Apagar meus dados": vault entries, database and files (010).
    pub async fn erase_all_data(&self) -> HostResult<()> {
        self.codex.shutdown().await;
        for t in self.platform.credentials.list("Aura/")? {
            self.platform.credentials.delete(&t)?;
        }
        for dir in [
            self.paths.workspaces(),
            self.paths.codex_home(),
            self.paths.screen_segments(),
            self.paths.audio_segments(),
            self.paths.skills(),
            self.paths.root.join("cache"),
        ] {
            let _ = std::fs::remove_dir_all(dir);
        }
        self.store.with_conn(|c| {
            c.execute_batch(
                "DELETE FROM settings; DELETE FROM conversations_meta; DELETE FROM providers; DELETE FROM capture_segments;
                 DELETE FROM recordings; DELETE FROM access_log; DELETE FROM agent_grants; DELETE FROM chatgpt_accounts;
                 DELETE FROM mcp_servers_meta; DELETE FROM quick_commands WHERE builtin = 0; DELETE FROM app_profiles;",
            )?;
            Ok(())
        })?;
        self.notice("info", "Dados apagados. O Aura será reiniciado.");
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.capture.shutdown().await;
        self.voice.shutdown().await;
        self.codex.shutdown().await;
    }

    pub fn now_ms(&self) -> i64 {
        now_ms()
    }

    /// Conversation stream for tests and the shell.
    pub fn conversation_events(&self) -> broadcast::Receiver<ConversationEvent> {
        self.codex.events()
    }
}

fn audio_label(which: &str) -> &'static str {
    match which {
        "mic" => "microfone",
        "system" => "áudio do sistema",
        _ => "microfone + sistema",
    }
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let dest = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &dest)?;
        } else {
            std::fs::copy(e.path(), dest)?;
        }
    }
    Ok(())
}
