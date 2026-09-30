//! "Continue with ChatGPT" — Sign in with ChatGPT with ChatGPT plan usage
//! (ADR 0007, `specs/002-conversa-agente-codex` TK-002).
//!
//! Flow (docs: developers.openai.com/siwc/token-sharing-open-source):
//! 1. [`AuthService::begin_login`] starts a loopback listener on
//!    `http://127.0.0.1:<port>/auth/callback`, builds the authorize URL with
//!    PKCE (S256), `state`, `nonce`, `ext_agent_host_id` and — for a new
//!    account — `client_id=dynamic_agent_client` + `agent_name_hint=Aura`.
//! 2. The browser returns `code`, `state` and the issued `client_id`.
//! 3. The code is exchanged (no client secret), the ID token is validated
//!    against the JWKS (iss, aud = issued client id, exp, nonce) and the
//!    granted scopes are checked for `chatgpt.tokens.use.direct`.
//! 4. Tokens go to the credential store only; metadata to `chatgpt_accounts`.
//! 5. [`AuthService::access_token`] refreshes ~5 min before expiry, one
//!    refresh at a time per account.

mod accounts;
mod config;
mod idtoken;
mod pkce;
mod service;
mod tokens;

pub use accounts::ChatGptAccount;
pub use config::SiwcConfig;
pub use service::{AuthError, AuthService, LoginAttempt, LoginOutcome, LoginProgress};
pub use tokens::TokenSet;

/// Scope that authorizes ChatGPT plan usage.
pub const PLAN_SCOPE: &str = "chatgpt.tokens.use.direct";
/// Link shown next to the plan indicator and in usage-limit dialogs.
pub const MANAGE_USAGE_URL: &str = "https://chatgpt.com/settings/usage";
