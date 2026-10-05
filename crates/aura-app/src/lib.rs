//! Aura host core (`aura-app`).
//!
//! [`host::Host`] is the composition root used by the desktop shell: it owns
//! the store, vault, policy, SIWC auth, the loopback gateway + MCP server, the
//! Codex service and the extension registries, and exposes one method per UI
//! command plus a single event stream ([`events::HostEvent`]).
//!
//! OS specifics arrive through [`platform::Platform`] (built from `aura-win`
//! on Windows, fakes elsewhere), so the whole host runs in Linux CI.

pub mod attachments;
pub mod consent;
pub mod core_skills;
pub mod diagnostics;
pub mod error;
pub mod events;
pub mod host;
pub mod launcher;
pub mod localization;
pub mod mcp_diag;
pub mod media;
pub mod memories;
pub mod notes;
pub mod paths;
pub mod platform;
pub mod privacy;
pub mod profiles;
pub mod recorder;
pub mod reminders;
pub mod settings_assistant;
pub mod speech;
pub mod tokens;
pub mod tools;
pub mod voice;
pub mod webview;

pub use error::{HostError, HostResult};
pub use host::{CodexRuntime, Host, HostConfig};
