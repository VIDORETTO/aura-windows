//! The composition root: builds every service once, owns the shared state and
//! exposes one method per UI command. The Tauri shell only forwards calls and
//! events; everything here is testable without a window system.

use crate::attachments::{AttachmentInfo, AttachmentService};
use crate::consent::{ConsentAnswer, ConsentBroker};
use crate::error::{HostError, HostResult};
use crate::events::{HostEvent, PrivacyState};
use crate::launcher::{AuraLauncher, LaunchState, SpawnHook};
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
use aura_auth::ChatGptAccount;
use aura_auth::SiwcConfig;
use aura_auth::{AuthService, LoginProgress};
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
use aura_extensions::skills::{self, SkillReview, SkillSource};
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
    _vault: Arc<Vault>,
    settings: RwLock<Settings>,
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
    profiles: crate::profiles::ProfilesRepo,
    /// Frozen screens waiting for a region selection (token → frame).
    frozen: Mutex<HashMap<String, aura_capture::frame::Frame>>,
    /// Runtime captured at start: sync methods are called from threads without
    /// a Tokio context (Tauri sync commands, tray, hotkeys).
    rt: tokio::runtime::Handle,
}

/// One MCP server as the running app-server sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStatus {
    pub name: String,
    pub tools: Vec<String>,
    /// `unsupported`, `notLoggedIn`, `bearerToken`, `oAuth`…
    pub auth: Option<String>,
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
            }
        })
        .collect()
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
        let tools = Arc::new(HostTools {
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
        if let Err(e) = codex.cleanup_on_start() {
            tracing::warn!("workspace cleanup failed: {e}");
        }
        let quick = QuickCommandsRepo::new(store.clone())?;

        let store_for_profiles = store.clone();
        let host = Arc::new(Host {
            paths: cfg.paths,
            platform: cfg.platform,
            store,
            _vault: vault,
            settings: RwLock::new(settings),
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
            frozen: Mutex::new(HashMap::new()),
            profiles: crate::profiles::ProfilesRepo::new(store_for_profiles),
            rt: tokio::runtime::Handle::current(),
        });
        host.apply_runtime_settings(&host.settings());
        host.spawn_forwarders();
        host.reconcile_capture();
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

    pub fn update_settings(&self, patch: SettingsPatch) -> HostResult<Settings> {
        let next = self.settings.read().unwrap().apply(&patch)?;
        SettingsRepo::new(&self.store).save(&next)?;
        *self.settings.write().unwrap() = next.clone();
        self.apply_runtime_settings(&next);
        Ok(next)
    }

    /// Settings that configure running services (memories, cloud ASR).
    fn apply_runtime_settings(&self, s: &Settings) {
        self.launch_state.write().unwrap().base.memories = s.memories;
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
        let mut p = self.policy.read().unwrap().clone();
        let sp = SourcePolicy { mode, agent };
        match source {
            Source::Screen | Source::Selection => p.screen = sp,
            Source::Mic => p.mic = sp,
            Source::SystemAudio => p.system_audio = sp,
        }
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

    pub fn access_log(&self, limit: u32) -> HostResult<Vec<AccessLogEntry>> {
        Ok(self.privacy.access_log(limit.clamp(1, 1000))?)
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
        let (attempt, done) = self.auth.begin_login(reauthorize, force_consent).await?;
        let host = self.clone();
        tokio::spawn(async move {
            match done.await {
                Ok(Ok(outcome)) => {
                    let _ = host.events.send(HostEvent::Login(LoginProgress::Completed {
                        account: outcome.account,
                        first_time: outcome.show_welcome,
                    }));
                }
                Ok(Err(e)) => {
                    let _ = host.events.send(HostEvent::Login(LoginProgress::Failed {
                        reason: e.to_string(),
                    }));
                }
                Err(_) => {}
            }
        });
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
        Ok(p)
    }

    pub fn remove_provider(&self, id: &str) -> HostResult<()> {
        ProviderRegistry::new(&self.store, self.platform.credentials.as_ref()).remove(id)?;
        self.refresh_provider_routes()
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
        } else {
            reg.set_status(id, ProviderStatus::Error, Some(result.detail))?;
        }
        self.refresh_provider_routes()?;
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
        if opts.model.is_none() {
            opts.model = s.default_model.clone();
        }
        // App profile of the app the Overlay was opened over (009 TK-004).
        if opts.profile.is_none()
            && let Some(app) = self.platform.foreground.current()
            && let Some(p) = self.profiles.for_app(&app)
        {
            if !p.instructions.trim().is_empty() {
                opts.profile = Some((p.name.clone(), p.instructions.clone()));
            }
            if opts.model.is_none() {
                opts.model = p.default_model.clone();
            }
        }
        opts.language = match s.language {
            aura_core::settings::Language::PtBr => aura_codex::modes::UiLanguage::PtBr,
            aura_core::settings::Language::En => aura_codex::modes::UiLanguage::En,
        };
        // BYOK: tell Codex the model limits it has no metadata for.
        if let Some(id) = opts
            .provider
            .strip_prefix("aura-")
            .filter(|id| *id != CHATGPT_PLAN_ROUTE)
            && let Some(model) = &opts.model
            && let Ok(Some(p)) =
                ProviderRegistry::new(&self.store, self.platform.credentials.as_ref()).get(id)
        {
            opts.config_overrides
                .extend(aura_gateway::codex_config::thread_overrides(&p, model));
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

    /// Models of the ChatGPT plan route (via Codex `model/list`).
    pub async fn models(&self) -> HostResult<Vec<ModelInfo>> {
        Ok(self.codex.codex_models().await?)
    }

    // ----------------------------------------------------------------- context

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

    pub fn region_cancel(&self, token: &str) {
        self.frozen.lock().unwrap().remove(token);
        let _ = std::fs::remove_file(
            self.paths
                .captures_tmp()
                .join(format!("frozen-{token}.png")),
        );
    }

    /// Selected text of the previous app into a chip (respects exclusions).
    pub fn capture_selection(&self, tray: &str) -> HostResult<Option<ContextChip>> {
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
        let preview: String = text.chars().take(40).collect();
        let chip = ContextChip::new(
            ChipKind::Selection,
            format!("❝ {preview}"),
            ChipPayload::Text { text },
        );
        Ok(Some(self.add_chip(tray, chip)?))
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
    pub fn read_workspace_file(&self, thread_id: &str, rel: &str) -> HostResult<String> {
        let (_, ws) = self
            .conversation_of(thread_id)
            .ok_or_else(|| HostError::new("not_found", "conversa não encontrada"))?;
        let root = ws.canonicalize()?;
        let path = root.join(rel).canonicalize()?;
        if !path.starts_with(&root) {
            return Err(HostError::new("invalid", "caminho fora do workspace"));
        }
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

    /// Dictated or typed text into the previous app (006 "inserir no app").
    pub fn insert_into_app(&self, text: &str) -> bool {
        if let Some(app) = self.platform.foreground.current() {
            self.platform.foreground.restore(&app);
        }
        self.platform.foreground.paste(text)
    }

    // ---------------------------------------------------------- quick commands

    pub fn quick_commands(&self) -> HostResult<Vec<QuickCommand>> {
        Ok(self.quick.list()?)
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
        };
        let exp = self.quick.run(input, &ctx)?;
        if exp.needs_screen && !chips.iter().any(|c| c.kind == ChipKind::Screen) {
            self.capture_screen(tray, false).await?;
        }
        // The selection is inlined in the prompt; don't send it twice.
        if ctx.selection.is_some() {
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
        let sel = device.map(DeviceSel::Id).unwrap_or(DeviceSel::Default);
        self.voice
            .press(sel)
            .await
            .map_err(|e| HostError::new("asr", e))
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
            .map_err(|e| HostError::new("recording", e))
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

    /// Speaks an answer: returns `(base64 audio, mime)` for the UI to play.
    pub async fn speak(&self, markdown: &str) -> HostResult<(String, String)> {
        use base64::Engine;
        let text = crate::speech::speakable(markdown);
        let lang = match self.settings().language {
            aura_core::settings::Language::PtBr => "pt-BR",
            aura_core::settings::Language::En => "en-US",
        };
        let speech = self.platform.speech.clone();
        let (bytes, mime) = tokio::task::spawn_blocking(move || speech.synthesize(&text, lang))
            .await
            .map_err(|e| HostError::new("speech", e.to_string()))?
            .map_err(|e| HostError::new("speech", e))?;
        Ok((
            base64::engine::general_purpose::STANDARD.encode(bytes),
            mime,
        ))
    }

    // ------------------------------------------------------------- diagnostics

    pub async fn diagnostics(&self) -> Diagnostics {
        Diagnostics {
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
