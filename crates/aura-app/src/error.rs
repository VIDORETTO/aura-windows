//! One error type for UI commands, serialized as `{code, message}`.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[error("{message}")]
pub struct HostError {
    /// Stable machine code the UI maps to copy and actions.
    pub code: String,
    /// Human-readable detail (already in the UI language when possible).
    pub message: String,
}

impl HostError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

macro_rules! from_err {
    ($t:ty, $code:expr) => {
        impl From<$t> for HostError {
            fn from(e: $t) -> Self {
                HostError::new($code, e.to_string())
            }
        }
    };
}

from_err!(aura_store::StoreError, "storage");
from_err!(aura_store::vault::VaultError, "vault");
from_err!(aura_codex::service::ServiceError, "conversation");
from_err!(aura_auth::AuthError, "auth");
from_err!(aura_gateway::registry::RegistryError, "provider");
from_err!(aura_core::context::ChipError, "context");
from_err!(aura_core::settings::SettingsError, "settings");
from_err!(aura_core::credentials::CredentialError, "vault");
from_err!(aura_ingest::IngestError, "attachment");
from_err!(aura_extensions::quick::QuickError, "quick_command");
from_err!(aura_extensions::mcp_config::McpConfigError, "mcp");
from_err!(aura_extensions::skills::SkillError, "skill");
from_err!(aura_capture::source::CaptureError, "capture");
from_err!(std::io::Error, "io");
from_err!(crate::profiles::ProfileError, "profile");

pub type HostResult<T> = Result<T, HostError>;
