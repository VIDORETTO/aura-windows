//! `CodexService`: the only Interface the desktop host uses to talk to Codex.

use crate::approvals::{ApprovalError, Decision, PendingRequests};
use crate::events::{AppServerState, ConversationEvent, TurnError, TurnStatus};
use crate::mapping::map_notification;
use crate::modes::{
    ConversationMode, InstructionLayers, UiLanguage, developer_instructions, persona,
    thread_params, turn_overrides,
};
use crate::supervisor::{AppServerSupervisor, SupervisorError, SupervisorSignal};
use aura_core::context::TurnInput;
use aura_core::jsonrpc::{Incoming, Peer, RpcError};
use aura_store::Store;
use aura_store::conversations::{ConversationMeta, ConversationsRepo};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, broadcast, mpsc};

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Supervisor(#[from] SupervisorError),
    #[error(transparent)]
    Rpc(#[from] RpcError),
    #[error(transparent)]
    Approval(#[from] ApprovalError),
    #[error("conversation not found")]
    NotFound,
    #[error("message is empty")]
    EmptyMessage,
    #[error("message is too long ({0} characters; max 100000)")]
    TooLong(usize),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage: {0}")]
    Store(#[from] aura_store::StoreError),
}

pub const MAX_MESSAGE_CHARS: usize = 100_000;
pub const SERVICE_NAME: &str = "aura_desktop";
/// Synchronous, lightweight consumer bookkeeping before event broadcast.
pub type EventObserver = Arc<dyn Fn(&ConversationEvent) + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StartOptions {
    pub mode: ConversationMode,
    /// Codex provider id: `aura-chatgpt-plan` or `aura-<byok id>`.
    pub provider: String,
    pub model: Option<String>,
    pub ephemeral: bool,
    pub language: UiLanguage,
    pub personal_instructions: String,
    pub profile: Option<(String, String)>,
    pub conversation_instructions: String,
    pub previous_summary: Option<String>,
    /// Extra per-thread config overrides (e.g. `model_context_window` for BYOK).
    pub config_overrides: Map<String, Value>,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self {
            mode: ConversationMode::Chat,
            provider: crate::home::CHATGPT_PLAN_PROVIDER.into(),
            model: None,
            ephemeral: false,
            language: UiLanguage::PtBr,
            personal_instructions: String::new(),
            profile: None,
            conversation_instructions: String::new(),
            previous_summary: None,
            config_overrides: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartedConversation {
    pub thread_id: String,
    pub conversation_uuid: String,
    pub workspace: PathBuf,
    pub ephemeral: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HistoryQuery {
    pub search: Option<String>,
    pub archived: bool,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub updated_at: i64,
    pub pinned: bool,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage {
    pub items: Vec<ConversationSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptMessage {
    /// `user` or `assistant`.
    pub role: String,
    pub text: String,
    /// Host correlation only; these protocol identifiers stay out of the UI DTO.
    #[serde(skip)]
    pub item_id: String,
    #[serde(skip)]
    pub turn_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnOptions {
    pub model: Option<String>,
    pub effort: Option<String>,
}

struct ThreadCtx {
    meta: ConversationMeta,
    mode: ConversationMode,
    ephemeral: bool,
    /// Mode the agent was last told about (thread start or a previous
    /// announcement); None after a restart, when it is not known (014).
    announced: Option<&'static str>,
}

struct Shared {
    threads: Mutex<HashMap<String, ThreadCtx>>,
    active_turns: Mutex<HashMap<String, String>>,
    loaded: Mutex<HashSet<String>>,
    /// Interface language, for texts Aura adds to turns (014).
    language: std::sync::RwLock<UiLanguage>,
    /// YOLO (018): Task conversations run without sandbox or approvals.
    yolo: std::sync::atomic::AtomicBool,
}

pub struct CodexService {
    supervisor: AppServerSupervisor,
    pending: Arc<PendingRequests>,
    events: broadcast::Sender<ConversationEvent>,
    shared: Arc<Shared>,
    store: Store,
    workspaces_root: PathBuf,
    /// Skill folders besides Codex's own roots (Aura-managed skills).
    skill_roots: std::sync::Mutex<Vec<PathBuf>>,
    event_observer: std::sync::RwLock<Option<EventObserver>>,
}

/// One skill as the app-server sees it (`skills/list`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSkill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    /// `user`, `repo`, `system`, `admin`.
    pub scope: String,
    pub enabled: bool,
}

fn input_json(i: &TurnInput) -> Value {
    match i {
        TurnInput::Text { text } => json!({"type": "text", "text": text, "text_elements": []}),
        TurnInput::LocalImage { path } => json!({"type": "localImage", "path": path}),
        TurnInput::Skill { name, path } => json!({"type": "skill", "name": name, "path": path}),
    }
}

fn mode_from_key(key: &str, granted: &[PathBuf]) -> ConversationMode {
    match key {
        "task" => ConversationMode::Task {
            granted: granted.to_vec(),
            network: false,
        },
        "plan" => ConversationMode::Plan,
        _ => ConversationMode::Chat,
    }
}

impl CodexService {
    /// `incoming_rx` must be the receiver whose sender was given to the supervisor.
    pub fn new(
        supervisor: AppServerSupervisor,
        incoming_rx: mpsc::UnboundedReceiver<Incoming>,
        store: Store,
        workspaces_root: PathBuf,
    ) -> Arc<Self> {
        let (events, _) = broadcast::channel(1024);
        let svc = Arc::new(Self {
            supervisor,
            pending: Arc::new(PendingRequests::new()),
            events,
            shared: Arc::new(Shared {
                threads: Mutex::new(HashMap::new()),
                active_turns: Mutex::new(HashMap::new()),
                loaded: Mutex::new(HashSet::new()),
                language: std::sync::RwLock::new(UiLanguage::PtBr),
                yolo: std::sync::atomic::AtomicBool::new(false),
            }),
            store,
            workspaces_root,
            skill_roots: std::sync::Mutex::new(Vec::new()),
            event_observer: std::sync::RwLock::new(None),
        });
        svc.clone().spawn_router(incoming_rx);
        svc.clone().spawn_supervisor_listener();
        svc
    }

    pub fn events(&self) -> broadcast::Receiver<ConversationEvent> {
        self.events.subscribe()
    }

    pub fn supervisor(&self) -> &AppServerSupervisor {
        &self.supervisor
    }

    pub fn pending_request_ids(&self) -> Vec<String> {
        self.pending.ids()
    }

    fn emit(&self, e: ConversationEvent) {
        let observer = self.event_observer.read().unwrap().clone();
        if let Some(observer) = observer {
            observer(&e);
        }
        let _ = self.events.send(e);
    }

    pub fn set_event_observer(&self, observer: EventObserver) {
        *self.event_observer.write().unwrap() = Some(observer);
    }

    fn spawn_router(self: Arc<Self>, mut rx: mpsc::UnboundedReceiver<Incoming>) {
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Incoming::Notification { method, params } => {
                        for event in map_notification(&method, &params) {
                            match &event {
                                ConversationEvent::TurnStarted { thread_id, turn_id } => {
                                    self.shared
                                        .active_turns
                                        .lock()
                                        .await
                                        .insert(thread_id.clone(), turn_id.clone());
                                }
                                ConversationEvent::TurnCompleted { thread_id, .. } => {
                                    if self
                                        .shared
                                        .active_turns
                                        .lock()
                                        .await
                                        .remove(thread_id)
                                        .is_some()
                                    {
                                        self.supervisor.turn_finished().await;
                                    }
                                }
                                ConversationEvent::RequestResolved { request_id } => {
                                    self.pending.resolved(request_id);
                                    self.supervisor
                                        .set_pending_requests(self.pending.len())
                                        .await;
                                }
                                _ => {}
                            }
                            self.emit(event);
                        }
                    }
                    Incoming::Request {
                        method,
                        params,
                        responder,
                    } => match self.pending.register(&method, &params, responder) {
                        Ok(event) => {
                            // YOLO (018): permissions in Task are granted without asking.
                            if let Some((id, decision)) = self.yolo_answer(&event).await {
                                let _ = self.pending.respond(&id, decision);
                                tracing::info!(method = %method, "YOLO accepted a request");
                                continue;
                            }
                            self.supervisor
                                .set_pending_requests(self.pending.len())
                                .await;
                            self.emit(event);
                        }
                        Err(responder) => responder.err(
                            RpcError::METHOD_NOT_FOUND,
                            format!("Aura does not handle {method}"),
                        ),
                    },
                }
            }
        });
    }

    fn spawn_supervisor_listener(self: Arc<Self>) {
        let mut signals = self.supervisor.subscribe();
        tokio::spawn(async move {
            loop {
                match signals.recv().await {
                    Ok(SupervisorSignal::Crashed) => {
                        self.shared.loaded.lock().await.clear();
                        let active: Vec<(String, String)> =
                            self.shared.active_turns.lock().await.drain().collect();
                        for (thread_id, turn_id) in active {
                            self.emit(ConversationEvent::TurnCompleted {
                                thread_id,
                                turn_id,
                                status: TurnStatus::Failed,
                                error: Some(TurnError::AppServerCrashed),
                            });
                        }
                    }
                    Ok(SupervisorSignal::State(state)) => {
                        if matches!(state, AppServerState::Stopped | AppServerState::Starting) {
                            self.shared.loaded.lock().await.clear();
                        }
                        self.emit(ConversationEvent::AppServerState { state });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        });
    }

    async fn peer(&self) -> Result<Peer, ServiceError> {
        Ok(self.supervisor.ensure_running().await?)
    }

    /// Skill folders the app-server should load besides its own roots.
    pub fn set_skill_roots(&self, roots: Vec<PathBuf>) {
        *self.skill_roots.lock().unwrap() = roots;
    }

    async fn apply_skill_roots(&self, peer: &Peer) -> Result<(), ServiceError> {
        let roots = self.skill_roots.lock().unwrap().clone();
        if !roots.is_empty() {
            peer.request("skills/extraRoots/set", json!({"extraRoots": roots}))
                .await?;
        }
        Ok(())
    }

    /// Every skill available to new conversations, with origin scope and
    /// enabled state (008 AC-001).
    pub async fn skills(&self) -> Result<Vec<CodexSkill>, ServiceError> {
        let peer = self.peer().await?;
        self.apply_skill_roots(&peer).await?;
        let result = peer
            .request(
                "skills/list",
                json!({"cwds": [self.workspaces_root], "forceReload": true}),
            )
            .await?;
        let mut out: Vec<CodexSkill> = result["data"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|entry| entry["skills"].as_array().cloned().unwrap_or_default())
            .filter_map(|s| serde_json::from_value(s).ok())
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out.dedup_by(|a, b| a.path == b.path);
        Ok(out)
    }

    /// Enables or disables a skill for new conversations (persisted by Codex).
    pub async fn set_skill_enabled(&self, path: &Path, enabled: bool) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request(
            "skills/config/write",
            json!({"path": path, "enabled": enabled}),
        )
        .await?;
        Ok(())
    }

    /// Starts a conversation (AC-007, AC-017, AC-024, AC-025).
    pub async fn start(&self, opts: StartOptions) -> Result<StartedConversation, ServiceError> {
        self.set_language(opts.language);
        let uuid = uuid::Uuid::new_v4().to_string();
        let workspace = if opts.ephemeral {
            self.workspaces_root.join("_ephemeral").join(&uuid)
        } else {
            self.workspaces_root.join(&uuid)
        };
        std::fs::create_dir_all(&workspace)?;

        let layers = InstructionLayers {
            personal: opts.personal_instructions.clone(),
            profile: opts.profile.clone(),
            conversation: opts.conversation_instructions.clone(),
            previous_summary: opts.previous_summary.clone(),
        };
        let mut config = opts.config_overrides.clone();
        // Aura supplies free public web tools over MCP. Hosted search must not
        // be enabled by either a caller override or a restored thread config.
        config.insert("web_search".into(), json!("disabled"));
        // Native code-mode models must call Aura MCP directly so the gateway
        // can protect arguments and restore transient web results by tool scope.
        config.insert(
            "features.code_mode.direct_only_tool_namespaces".into(),
            json!(["mcp__aura"]),
        );
        config.insert(
            "mcp_servers.aura.http_headers".into(),
            json!({ "X-Aura-Conversation": uuid }),
        );
        let mut params = json!({
            "modelProvider": opts.provider,
            "baseInstructions": persona(opts.language),
            "developerInstructions": developer_instructions(&opts.mode, opts.language, &layers),
            "ephemeral": opts.ephemeral,
            "serviceName": SERVICE_NAME,
            "config": config,
        });
        if let Some(model) = &opts.model {
            params["model"] = json!(model);
        }
        for (k, v) in thread_params(&opts.mode, &workspace, self.yolo())
            .as_object()
            .expect("object")
        {
            params[k] = v.clone();
        }

        let peer = self.peer().await?;
        // Aura skills join new conversations (unless disabled in Codex).
        let _ = self.apply_skill_roots(&peer).await;
        let result = peer.request("thread/start", params).await?;
        let thread_id = result["thread"]["id"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let granted = match &opts.mode {
            ConversationMode::Task { granted, .. } => granted.clone(),
            _ => vec![],
        };
        let meta = ConversationMeta {
            thread_id: thread_id.clone(),
            conversation_uuid: uuid.clone(),
            workspace_path: workspace.clone(),
            mode: opts.mode.key().into(),
            provider_id: opts.provider.clone(),
            granted_folders: granted,
            extra_instructions: opts.conversation_instructions.clone(),
        };
        if !opts.ephemeral {
            ConversationsRepo::new(&self.store).upsert(&meta)?;
        }
        self.shared.loaded.lock().await.insert(thread_id.clone());
        self.shared.threads.lock().await.insert(
            thread_id.clone(),
            ThreadCtx {
                meta,
                announced: Some(opts.mode.key()),
                mode: opts.mode,
                ephemeral: opts.ephemeral,
            },
        );
        Ok(StartedConversation {
            thread_id,
            conversation_uuid: uuid,
            workspace,
            ephemeral: opts.ephemeral,
        })
    }

    async fn context(
        &self,
        thread_id: &str,
    ) -> Result<(ConversationMeta, ConversationMode, bool), ServiceError> {
        if let Some(ctx) = self.shared.threads.lock().await.get(thread_id) {
            return Ok((ctx.meta.clone(), ctx.mode.clone(), ctx.ephemeral));
        }
        let meta = ConversationsRepo::new(&self.store)
            .get(thread_id)?
            .ok_or(ServiceError::NotFound)?;
        let mode = mode_from_key(&meta.mode, &meta.granted_folders);
        self.shared.threads.lock().await.insert(
            thread_id.to_string(),
            ThreadCtx {
                meta: meta.clone(),
                mode: mode.clone(),
                ephemeral: false,
                announced: None,
            },
        );
        Ok((meta, mode, false))
    }

    async fn ensure_loaded(&self, peer: &Peer, thread_id: &str) -> Result<(), ServiceError> {
        if self.shared.loaded.lock().await.contains(thread_id) {
            return Ok(());
        }
        let (meta, _, _) = self.context(thread_id).await?;
        peer.request(
            "thread/resume",
            json!({"threadId": thread_id,"config":{
                "web_search":"disabled",
                "features.code_mode.direct_only_tool_namespaces":["mcp__aura"],
                "mcp_servers.aura.http_headers":{"X-Aura-Conversation":meta.conversation_uuid}
            }}),
        )
        .await?;
        self.shared
            .loaded
            .lock()
            .await
            .insert(thread_id.to_string());
        Ok(())
    }

    /// Sends a user turn (AC-005, AC-009, AC-013).
    pub async fn send(
        &self,
        thread_id: &str,
        inputs: &[TurnInput],
        opts: TurnOptions,
    ) -> Result<String, ServiceError> {
        let text_len: usize = inputs
            .iter()
            .map(|i| {
                if let TurnInput::Text { text } = i {
                    text.chars().count()
                } else {
                    0
                }
            })
            .sum();
        if text_len > MAX_MESSAGE_CHARS {
            return Err(ServiceError::TooLong(text_len));
        }
        let has_content = inputs.iter().any(|i| match i {
            TurnInput::Text { text } => !text.trim().is_empty(),
            _ => true,
        });
        if !has_content {
            return Err(ServiceError::EmptyMessage);
        }
        let (meta, mode, _) = self.context(thread_id).await?;
        let peer = self.peer().await?;
        self.ensure_loaded(&peer, thread_id).await?;
        let mut params = turn_overrides(&mode, &meta.workspace_path, self.yolo());
        params["threadId"] = json!(thread_id);
        let mut input: Vec<Value> = inputs.iter().map(input_json).collect();
        // Mode changed since the agent was last told (014): say so after the
        // user's own input.
        let announce = {
            let mut threads = self.shared.threads.lock().await;
            match threads.get_mut(thread_id) {
                Some(ctx) if ctx.announced != Some(mode.key()) => {
                    ctx.announced = Some(mode.key());
                    true
                }
                _ => false,
            }
        };
        if announce {
            let lang = *self.shared.language.read().unwrap();
            input.push(input_json(&TurnInput::Text {
                text: crate::modes::mode_change_note(&mode, lang),
            }));
        }
        params["input"] = Value::Array(input);
        if let Some(m) = opts.model {
            params["model"] = json!(m);
        }
        if let Some(e) = opts.effort {
            params["effort"] = json!(e);
        }
        self.supervisor.turn_started().await;
        match peer.request("turn/start", params).await {
            Ok(result) => {
                let turn_id = result["turn"]["id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                self.shared
                    .active_turns
                    .lock()
                    .await
                    .entry(thread_id.to_string())
                    .or_insert(turn_id.clone());
                Ok(turn_id)
            }
            Err(e) => {
                self.supervisor.turn_finished().await;
                Err(e.into())
            }
        }
    }

    pub async fn active_turn(&self, thread_id: &str) -> Option<String> {
        self.shared
            .active_turns
            .lock()
            .await
            .get(thread_id)
            .cloned()
    }

    /// Appends input to the running turn; falls back to a new turn when it
    /// already ended (AC-022). Returns the turn id used.
    pub async fn steer(
        &self,
        thread_id: &str,
        inputs: &[TurnInput],
    ) -> Result<String, ServiceError> {
        let Some(turn_id) = self.active_turn(thread_id).await else {
            return self.send(thread_id, inputs, TurnOptions::default()).await;
        };
        let peer = self.peer().await?;
        let params = json!({
            "threadId": thread_id,
            "expectedTurnId": turn_id,
            "input": inputs.iter().map(input_json).collect::<Vec<_>>(),
        });
        match peer.request("turn/steer", params).await {
            Ok(_) => Ok(turn_id),
            Err(RpcError::Remote { .. }) => {
                self.send(thread_id, inputs, TurnOptions::default()).await
            }
            Err(e) => Err(e.into()),
        }
    }

    pub async fn interrupt(&self, thread_id: &str) -> Result<(), ServiceError> {
        let turn_id = self.active_turn(thread_id).await;
        let peer = self.peer().await?;
        peer.request(
            "turn/interrupt",
            json!({"threadId": thread_id, "turnId": turn_id}),
        )
        .await?;
        Ok(())
    }

    pub async fn compact(&self, thread_id: &str) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        self.ensure_loaded(&peer, thread_id).await?;
        peer.request("thread/compact/start", json!({"threadId": thread_id}))
            .await?;
        Ok(())
    }

    /// Changes the mode of an existing conversation (applied from the next turn).
    /// Language of the texts Aura adds to turns (mode announcements).
    pub fn set_yolo(&self, on: bool) {
        self.shared
            .yolo
            .store(on, std::sync::atomic::Ordering::SeqCst);
    }

    fn yolo(&self) -> bool {
        self.shared.yolo.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// The automatic answer YOLO gives to a request of a Task conversation.
    async fn yolo_answer(&self, event: &ConversationEvent) -> Option<(String, Decision)> {
        if !self.yolo() {
            return None;
        }
        let (thread_id, request_id) = match event {
            ConversationEvent::ApprovalRequested {
                thread_id,
                request_id,
                ..
            } => (thread_id.clone(), request_id.clone()),
            ConversationEvent::UserInputRequested {
                thread_id: Some(thread_id),
                request_id,
                ..
            } => (thread_id.clone(), request_id.clone()),
            _ => return None,
        };
        let in_task = self
            .shared
            .threads
            .lock()
            .await
            .get(&thread_id)
            .is_some_and(|ctx| matches!(ctx.mode, ConversationMode::Task { .. }));
        if !in_task {
            return None;
        }
        let decision = self.pending.yolo_decision(&request_id)?;
        Some((request_id, decision))
    }

    pub fn set_language(&self, lang: UiLanguage) {
        *self.shared.language.write().unwrap() = lang;
    }

    pub async fn set_mode(
        &self,
        thread_id: &str,
        mode: ConversationMode,
    ) -> Result<(), ServiceError> {
        let (mut meta, _, ephemeral) = self.context(thread_id).await?;
        meta.mode = mode.key().into();
        if let ConversationMode::Task { granted, .. } = &mode {
            meta.granted_folders = granted.clone();
        }
        if !ephemeral {
            ConversationsRepo::new(&self.store).upsert(&meta)?;
        }
        let mut threads = self.shared.threads.lock().await;
        let announced = threads.get(thread_id).and_then(|t| t.announced);
        threads.insert(
            thread_id.to_string(),
            ThreadCtx {
                meta,
                mode,
                ephemeral,
                announced,
            },
        );
        Ok(())
    }

    pub async fn list(&self, q: HistoryQuery) -> Result<HistoryPage, ServiceError> {
        let peer = self.peer().await?;
        // `modelProviders: []` = every provider. Omitted, the app-server lists
        // only its configured default, hiding BYOK conversations (QA-015).
        let mut params =
            json!({"archived": q.archived, "limit": q.limit.unwrap_or(50), "modelProviders": []});
        if let Some(s) = q.search.filter(|s| !s.trim().is_empty()) {
            params["searchTerm"] = json!(s);
        }
        if let Some(c) = q.cursor {
            params["cursor"] = json!(page_cursor(&c));
        }
        let result = peer.request("thread/list", params).await?;
        let mut items: Vec<ConversationSummary> = result["data"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|t| {
                        let preview = t["preview"].as_str().unwrap_or_default().to_string();
                        ConversationSummary {
                            id: t["id"].as_str().unwrap_or_default().to_string(),
                            title: t["name"]
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| preview.clone()),
                            preview,
                            updated_at: t["updatedAt"].as_i64().unwrap_or(0),
                            pinned: t["isPinned"].as_bool().unwrap_or(false),
                            archived: t["archived"].as_bool().unwrap_or(q.archived),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        items.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then(b.updated_at.cmp(&a.updated_at))
        });
        Ok(HistoryPage {
            items,
            next_cursor: result["nextCursor"].as_str().map(str::to_string),
        })
    }

    /// Resumes a conversation and returns its transcript (AC-015).
    pub async fn open(&self, thread_id: &str) -> Result<Vec<TranscriptMessage>, ServiceError> {
        let peer = self.peer().await?;
        self.ensure_loaded(&peer, thread_id).await?;
        let result = peer
            .request(
                "thread/read",
                json!({"threadId": thread_id, "includeTurns": true}),
            )
            .await?;
        let mut out = Vec::new();
        for turn in result["thread"]["turns"].as_array().into_iter().flatten() {
            for item in turn["items"].as_array().into_iter().flatten() {
                match item["type"].as_str() {
                    Some("userMessage") => {
                        let text = item["content"]
                            .as_array()
                            .map(|c| {
                                c.iter()
                                    .filter_map(|p| p["text"].as_str())
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                            .unwrap_or_default();
                        let text = crate::modes::strip_mode_notes(&text);
                        out.push(TranscriptMessage {
                            role: "user".into(),
                            text,
                            item_id: item["id"].as_str().unwrap_or_default().into(),
                            turn_id: turn["id"].as_str().unwrap_or_default().into(),
                        });
                    }
                    Some("agentMessage") => out.push(TranscriptMessage {
                        role: "assistant".into(),
                        text: item["text"].as_str().unwrap_or_default().to_string(),
                        item_id: item["id"].as_str().unwrap_or_default().into(),
                        turn_id: turn["id"].as_str().unwrap_or_default().into(),
                    }),
                    _ => {}
                }
            }
        }
        Ok(out)
    }

    pub async fn rename(&self, thread_id: &str, name: &str) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request(
            "thread/name/set",
            json!({"threadId": thread_id, "name": name}),
        )
        .await?;
        Ok(())
    }

    pub async fn pin(&self, thread_id: &str, pinned: bool) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request(
            "thread/metadata/update",
            json!({"threadId": thread_id, "isPinned": pinned}),
        )
        .await?;
        Ok(())
    }

    pub async fn archive(&self, thread_id: &str) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request("thread/archive", json!({"threadId": thread_id}))
            .await?;
        Ok(())
    }

    pub async fn unarchive(&self, thread_id: &str) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request("thread/unarchive", json!({"threadId": thread_id}))
            .await?;
        Ok(())
    }

    /// Deletes the thread, its workspace and metadata (AC-016). A folder that
    /// cannot be removed now is queued for the next start.
    pub async fn delete(&self, thread_id: &str) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request("thread/delete", json!({"threadId": thread_id}))
            .await?;
        let repo = ConversationsRepo::new(&self.store);
        if let Some(meta) = repo.get(thread_id)? {
            remove_dir_or_queue(&repo, &meta.workspace_path)?;
            repo.remove(thread_id)?;
        }
        self.shared.threads.lock().await.remove(thread_id);
        self.shared.loaded.lock().await.remove(thread_id);
        Ok(())
    }

    /// Forgets an ephemeral conversation and wipes its folder (AC-017).
    pub async fn close_ephemeral(&self, thread_id: &str) -> Result<(), ServiceError> {
        let ctx = self.shared.threads.lock().await.remove(thread_id);
        if let Some(ctx) = ctx.filter(|c| c.ephemeral) {
            let _ = std::fs::remove_dir_all(&ctx.meta.workspace_path);
        }
        Ok(())
    }

    /// Removes leftovers from previous runs (ephemeral folders, queued deletions).
    pub fn cleanup_on_start(&self) -> Result<(), ServiceError> {
        let _ = std::fs::remove_dir_all(self.workspaces_root.join("_ephemeral"));
        let repo = ConversationsRepo::new(&self.store);
        for path in repo.pending_deletions()? {
            if !path.exists() || std::fs::remove_dir_all(&path).is_ok() {
                repo.clear_deletion(&path)?;
            }
        }
        Ok(())
    }

    pub async fn respond(&self, request_id: &str, decision: Decision) -> Result<(), ServiceError> {
        self.pending.respond(request_id, decision)?;
        self.supervisor
            .set_pending_requests(self.pending.len())
            .await;
        self.emit(ConversationEvent::RequestResolved {
            request_id: request_id.to_string(),
        });
        Ok(())
    }

    /// Codex catalog (`model/list`); used for BYOK capability hints and as a
    /// fallback when the ChatGPT plan `/v1/models` is unavailable.
    pub async fn codex_models(&self) -> Result<Vec<crate::models::ModelInfo>, ServiceError> {
        let peer = self.peer().await?;
        let v = peer
            .request("model/list", json!({"includeHidden": false, "limit": 100}))
            .await?;
        Ok(crate::models::parse_codex_model_list(&v))
    }

    /// Status and tools of the configured MCP servers (`mcpServerStatus/list`).
    pub async fn mcp_status(&self) -> Result<Value, ServiceError> {
        let peer = self.peer().await?;
        Ok(peer.request("mcpServerStatus/list", json!({})).await?)
    }

    /// Starts the OAuth flow of an HTTP MCP server; returns the authorize URL.
    pub async fn mcp_oauth_login(&self, name: &str) -> Result<Option<String>, ServiceError> {
        let peer = self.peer().await?;
        let v = peer
            .request("mcpServer/oauth/login", json!({"name": name}))
            .await?;
        Ok(v["authorizationUrl"].as_str().map(str::to_string))
    }

    /// Re-reads MCP server config without restarting the app-server.
    pub async fn mcp_reload(&self) -> Result<(), ServiceError> {
        let peer = self.peer().await?;
        peer.request("config/mcpServer/reload", json!({})).await?;
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.supervisor.stop().await;
    }
}

fn remove_dir_or_queue(repo: &ConversationsRepo<'_>, path: &Path) -> Result<(), ServiceError> {
    if path.exists() && std::fs::remove_dir_all(path).is_err() {
        repo.queue_deletion(path)?;
    }
    Ok(())
}

/// Cursor for the next History page. The pinned app-server pages by
/// `updatedAt` with a whole-second RFC 3339 cursor and a strict "older than",
/// so conversations sharing the last item's second were never listed. Asking
/// from the end of that second re-reads them; the UI drops ids it already has.
pub fn page_cursor(cursor: &str) -> String {
    let b = cursor.as_bytes();
    let whole_second = b.len() == 20
        && b[19] == b'Z'
        && b[10] == b'T'
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            10 => *c == b'T',
            13 | 16 => *c == b':',
            19 => *c == b'Z',
            _ => c.is_ascii_digit(),
        });
    if whole_second {
        format!("{}.999Z", &cursor[..19])
    } else {
        cursor.to_string()
    }
}
