//! Token set persisted as one JSON secret per account (`Aura/chatgpt/<client_id>`).

use aura_core::Secret;
use aura_core::credentials::{CredentialError, CredentialStore, target};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    /// Unix seconds.
    pub expires_at: i64,
    pub scopes: Vec<String>,
}

impl std::fmt::Debug for TokenSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenSet")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("expires_at", &self.expires_at)
            .field("scopes", &self.scopes)
            .finish()
    }
}

pub fn credential_target(client_id: &str) -> String {
    target("chatgpt", client_id)
}

pub fn save(
    store: &dyn CredentialStore,
    client_id: &str,
    tokens: &TokenSet,
) -> Result<(), CredentialError> {
    let json = serde_json::to_string(tokens).expect("serializable");
    store.put(&credential_target(client_id), &Secret::new(json))
}

pub fn load(
    store: &dyn CredentialStore,
    client_id: &str,
) -> Result<Option<TokenSet>, CredentialError> {
    Ok(store
        .get(&credential_target(client_id))?
        .and_then(|s| serde_json::from_str(s.expose()).ok()))
}

pub fn clear(store: &dyn CredentialStore, client_id: &str) -> Result<(), CredentialError> {
    store.delete(&credential_target(client_id))
}
