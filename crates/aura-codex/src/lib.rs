//! Codex app-server integration (ADR 0001, `specs/002-conversa-agente-codex`).
//!
//! The rest of Aura talks to Codex exclusively through [`service::CodexService`]
//! and receives [`events::ConversationEvent`]s; protocol details stay here.

pub mod approvals;
pub mod binary;
pub mod errors;
pub mod events;
pub mod fake;
pub mod home;
pub mod launcher;
pub mod mapping;
pub mod models;
pub mod modes;
pub mod service;
pub mod supervisor;

pub use events::{AppServerState, ConversationEvent, TurnError, TurnStatus};
pub use service::CodexService;
