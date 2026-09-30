//! Bridges `aura-auth` (SIWC) to the gateway's `TokenProvider` so the
//! `aura-chatgpt-plan` route always forwards a fresh plan token.

use aura_auth::{AuthError, AuthService};
use aura_core::secret::Secret;
use aura_gateway::errors::UpstreamError;
use aura_gateway::upstream::{BoxFut, TokenProvider};
use std::sync::Arc;

pub struct AuthTokenProvider(pub Arc<AuthService>);

fn map(e: AuthError) -> UpstreamError {
    match e {
        AuthError::Network(m) => UpstreamError::unreachable(m),
        // Codex maps 401 to "session expired" → the UI offers "Entrar de novo".
        other => UpstreamError::new(401, "session_expired", other.to_string()),
    }
}

impl TokenProvider for AuthTokenProvider {
    fn token(&self) -> BoxFut<'_, Result<Secret<String>, UpstreamError>> {
        Box::pin(async move { self.0.access_token().await.map_err(map) })
    }
    fn refresh(&self) -> BoxFut<'_, Result<Secret<String>, UpstreamError>> {
        Box::pin(async move { self.0.on_unauthorized().await.map_err(map) })
    }
}
