//! Aura-side conversation events. The UI only ever sees these types, never the
//! raw Codex protocol, so upgrading the app-server does not ripple into the UI.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnStatus {
    Completed,
    Interrupted,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolKind {
    Command,
    FileChange,
    Mcp,
    Dynamic,
    WebSearch,
    ImageView,
    Collab,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemStatus {
    InProgress,
    Completed,
    Failed,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApprovalKind {
    Command,
    FileChange,
    Permissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeSummary {
    pub path: String,
    pub added: u32,
    pub removed: u32,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub step: String,
    /// `pending`, `inProgress` or `completed`.
    pub status: String,
}

/// Why a turn failed, in categories the UI knows how to explain (AC-008, 002).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TurnError {
    /// ChatGPT plan usage limit (`subscription_sharing_usage_limit_exceeded`).
    PlanUsageLimit,
    /// Plan/workspace not eligible (`subscription_sharing_user_not_eligible`).
    PlanNotEligible,
    /// Request used something the plan route does not support.
    UnsupportedCapability {
        param: Option<String>,
    },
    /// Provider usage limit / rate limit, with an optional wait hint.
    UsageLimit {
        retry_after_secs: Option<u64>,
    },
    NoConnection,
    SessionExpired,
    ContextTooLong,
    AppServerCrashed,
    Other {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ConversationEvent {
    TurnStarted {
        thread_id: String,
        turn_id: String,
    },
    MessageDelta {
        thread_id: String,
        item_id: String,
        delta: String,
    },
    MessageCompleted {
        thread_id: String,
        item_id: String,
        text: String,
    },
    ReasoningDelta {
        thread_id: String,
        item_id: String,
        delta: String,
    },
    ToolCall {
        thread_id: String,
        item_id: String,
        kind: ToolKind,
        title: String,
        status: ItemStatus,
        detail: Option<String>,
    },
    FileChanges {
        thread_id: String,
        item_id: String,
        changes: Vec<FileChangeSummary>,
        status: ItemStatus,
    },
    ApprovalRequested {
        thread_id: String,
        request_id: String,
        kind: ApprovalKind,
        command: Option<String>,
        cwd: Option<String>,
        reason: Option<String>,
        changes: Vec<FileChangeSummary>,
        options: Vec<String>,
    },
    UserInputRequested {
        thread_id: Option<String>,
        request_id: String,
        /// Raw questions/schema, rendered generically by the UI.
        prompt: Value,
        auto_resolve_ms: Option<u64>,
        source: String,
    },
    RequestResolved {
        request_id: String,
    },
    PlanUpdated {
        thread_id: String,
        turn_id: String,
        explanation: Option<String>,
        steps: Vec<PlanStep>,
    },
    PlanProposed {
        thread_id: String,
        item_id: String,
        text: String,
    },
    DiffUpdated {
        thread_id: String,
        turn_id: String,
        diff: String,
    },
    TokenUsage {
        thread_id: String,
        used: i64,
        window: Option<i64>,
    },
    Compacted {
        thread_id: String,
    },
    TurnCompleted {
        thread_id: String,
        turn_id: String,
        status: TurnStatus,
        error: Option<TurnError>,
    },
    AppServerState {
        state: AppServerState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "state",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AppServerState {
    Stopped,
    Downloading { bytes: u64, total: Option<u64> },
    Starting,
    Ready { version: String },
    Restarting { attempt: u32 },
    Failed { reason: String },
}
