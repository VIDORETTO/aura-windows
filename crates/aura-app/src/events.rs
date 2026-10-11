//! Everything the host pushes to the UI, on one typed stream
//! (`aura://event` in the Tauri shell).

use aura_auth::LoginProgress;
use aura_codex::events::ConversationEvent;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "channel",
    content = "event",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum HostEvent {
    /// Provider metadata changed; every window must reload its own catalog.
    /// The payload deliberately contains no credentials or provider details.
    ProvidersChanged {},
    Conversation(ConversationEvent),
    /// Verified provenance only; full pages remain in the memory cache.
    WebSource {
        thread_id: String,
        turn_id: String,
        source: aura_web::WebSource,
    },
    Login(LoginProgress),
    /// The agent asked for a source with permission "Ask" (004 AC-008).
    Consent(ConsentRequest),
    ConsentResolved {
        id: String,
    },
    Privacy(PrivacyState),
    /// Background download (app-server, ASR model) progress.
    Download {
        id: String,
        bytes: u64,
        total: Option<u64>,
        done: bool,
        error: Option<String>,
    },
    /// Live input level of a device test in Settings (005 AC-001), in dBFS
    /// (floor −100), at about 30 Hz.
    AudioLevel {
        source: aura_audio::AudioSourceKind,
        dbfs: f32,
    },
    /// Push-to-talk state (listening, transcribing, done…).
    Voice(aura_asr::ptt::PttState),
    /// Overlay-facing notice (toast): `info`, `warning`, `error`.
    Notice {
        level: String,
        message: String,
    },
    /// Settings asked the Overlay to show a conversation (access-log link).
    OpenConversation {
        thread_id: String,
    },
    /// Settings asked the agent to do something ("Create with AI", 017): the
    /// Overlay starts a new conversation in `mode` and sends `text`.
    AgentTask {
        text: String,
        mode: String,
    },
    /// Skills, quick commands or MCP servers changed (e.g. by the agent).
    ExtensionsChanged {},
    /// A Meeting started, got new speech (`updated`) or ended (023).
    Meeting {
        id: String,
        status: String,
    },
    /// A reminder is due (020): the Overlay shows a Windows notification.
    Reminder {
        id: String,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentRequest {
    pub id: String,
    pub conversation: String,
    pub tool: String,
    /// `screen`, `mic`, `systemAudio`.
    pub source: String,
    pub reason: String,
    pub app: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyState {
    pub paused: bool,
    /// `off`, `onDemand`, `recentBuffer`, `manual`, `continuous` per source.
    pub screen: String,
    pub mic: String,
    pub system_audio: String,
    /// Sources recording right now (`screen`, `mic`, `system`).
    pub recording: Vec<String>,
}
