//! The login/refresh/revoke service used by the host and the gateway.

use crate::accounts::{AccountsRepo, ChatGptAccount, host_id};
use crate::config::SiwcConfig;
use crate::idtoken;
use crate::pkce::{Pkce, random_urlsafe};
use crate::tokens::{self, TokenSet};
use aura_core::Secret;
use aura_core::credentials::CredentialStore;
use aura_store::{Store, now_secs};
use axum::Router;
use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{Mutex, broadcast, oneshot};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("sign-in was declined")]
    Denied,
    #[error("sign-in cancelled")]
    Cancelled,
    #[error("state mismatch")]
    StateMismatch,
    #[error("client id mismatch")]
    ClientIdMismatch,
    #[error("registration incomplete (no client id)")]
    RegistrationIncomplete,
    #[error("invalid ID token: {0}")]
    InvalidIdToken(String),
    #[error("please sign in again")]
    ReloginRequired,
    #[error("invalid client configuration")]
    InvalidClient,
    #[error("ChatGPT plan usage is not enabled for this account")]
    PlanUsageDisabled,
    #[error("no ChatGPT account signed in")]
    NotSignedIn,
    #[error("network: {0}")]
    Network(String),
    #[error("storage: {0}")]
    Storage(String),
}

impl From<aura_store::StoreError> for AuthError {
    fn from(e: aura_store::StoreError) -> Self {
        AuthError::Storage(e.to_string())
    }
}
impl From<aura_core::credentials::CredentialError> for AuthError {
    fn from(e: aura_core::credentials::CredentialError) -> Self {
        AuthError::Storage(e.to_string())
    }
}
impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::Network(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "state",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LoginProgress {
    WaitingBrowser {
        authorize_url: String,
    },
    Completed {
        account: ChatGptAccount,
        first_time: bool,
    },
    Failed {
        reason: String,
    },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttempt {
    pub authorize_url: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginOutcome {
    pub account: ChatGptAccount,
    /// True when this is the first successful sign-in with plan usage
    /// (show the "Você está usando seu plano ChatGPT" modal once).
    pub show_welcome: bool,
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    client_id: Option<String>,
    error: Option<String>,
}

struct Pending {
    cancel: oneshot::Sender<()>,
}

pub struct AuthService {
    cfg: SiwcConfig,
    http: reqwest::Client,
    store: Store,
    creds: Arc<dyn CredentialStore>,
    pending: Mutex<Option<Pending>>,
    refresh_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    jwks_cache: Mutex<Option<Value>>,
    progress: broadcast::Sender<LoginProgress>,
}

const CALLBACK_HTML: &str = r#"<!doctype html><html lang="pt-BR"><meta charset="utf-8"><title>Aura</title>
<body style="font-family:Segoe UI,system-ui,sans-serif;display:grid;place-items:center;height:100vh;margin:0;background:#f5f5f7;color:#1b1b1f">
<div style="text-align:center"><h1 style="font-weight:600">Pronto!</h1><p>Você já pode voltar ao Aura e fechar esta aba.</p></div></body></html>"#;

impl AuthService {
    pub fn new(cfg: SiwcConfig, store: Store, creds: Arc<dyn CredentialStore>) -> Arc<Self> {
        let (progress, _) = broadcast::channel(32);
        Arc::new(Self {
            cfg,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("http client"),
            store,
            creds,
            pending: Mutex::new(None),
            refresh_locks: Mutex::new(HashMap::new()),
            jwks_cache: Mutex::new(None),
            progress,
        })
    }

    pub fn progress(&self) -> broadcast::Receiver<LoginProgress> {
        self.progress.subscribe()
    }

    pub fn accounts(&self) -> Result<Vec<ChatGptAccount>, AuthError> {
        Ok(AccountsRepo::new(&self.store).list()?)
    }

    pub fn active(&self) -> Result<Option<ChatGptAccount>, AuthError> {
        Ok(AccountsRepo::new(&self.store)
            .active()?
            .filter(|a| a.signed_in))
    }

    pub fn switch(&self, client_id: &str) -> Result<(), AuthError> {
        let repo = AccountsRepo::new(&self.store);
        let acc = repo.get(client_id)?.ok_or(AuthError::NotSignedIn)?;
        if !acc.signed_in {
            return Err(AuthError::NotSignedIn);
        }
        repo.set_active(client_id)?;
        Ok(())
    }

    pub fn mark_welcomed(&self, client_id: &str) -> Result<(), AuthError> {
        let repo = AccountsRepo::new(&self.store);
        if let Some(mut a) = repo.get(client_id)? {
            a.welcomed = true;
            repo.upsert(&a)?;
        }
        Ok(())
    }

    /// Starts a sign-in. `existing` reuses an issued client id (reauthorization);
    /// `force_consent` asks again for plan usage after a decline.
    /// The returned future resolves when the browser comes back.
    pub async fn begin_login(
        self: &Arc<Self>,
        existing: Option<String>,
        force_consent: bool,
    ) -> Result<
        (
            LoginAttempt,
            tokio::task::JoinHandle<Result<LoginOutcome, AuthError>>,
        ),
        AuthError,
    > {
        self.cancel_login().await;
        let host_id = host_id(&self.store)?;
        let listener =
            match tokio::net::TcpListener::bind(("127.0.0.1", self.cfg.preferred_port)).await {
                Ok(l) => l,
                Err(_) => tokio::net::TcpListener::bind(("127.0.0.1", 0))
                    .await
                    .map_err(|e| AuthError::Network(e.to_string()))?,
            };
        let port = listener
            .local_addr()
            .map_err(|e| AuthError::Network(e.to_string()))?
            .port();
        let redirect_uri = format!("http://127.0.0.1:{port}/auth/callback");
        let pkce = Pkce::new();
        let state = random_urlsafe(24);
        let nonce = random_urlsafe(24);

        let known = match &existing {
            Some(id) => AccountsRepo::new(&self.store).get(id)?,
            None => None,
        };
        let client_param = existing
            .clone()
            .unwrap_or_else(|| "dynamic_agent_client".into());
        let mut url = url::Url::parse(&self.cfg.authorize_url)
            .map_err(|e| AuthError::Network(e.to_string()))?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("client_id", &client_param);
            if existing.is_none() {
                q.append_pair("agent_name_hint", &self.cfg.agent_name);
            }
            q.append_pair("ext_agent_host_id", &host_id);
            q.append_pair("response_type", "code");
            q.append_pair("redirect_uri", &redirect_uri);
            q.append_pair("scope", &self.cfg.scopes.join(" "));
            q.append_pair("resource", &self.cfg.resource);
            q.append_pair("state", &state);
            q.append_pair("nonce", &nonce);
            q.append_pair("code_challenge_method", "S256");
            q.append_pair("code_challenge", &pkce.challenge);
            if let Some(acc) = &known {
                if let Some(email) = &acc.email {
                    q.append_pair("login_hint", email);
                }
                if let Ok(Some(t)) = tokens::load(self.creds.as_ref(), &acc.client_id)
                    && let Some(id) = t.id_token
                {
                    q.append_pair("id_token_hint", &id);
                }
            }
            if force_consent {
                q.append_pair("prompt", "consent");
            }
        }
        let authorize_url = url.to_string();

        let (cb_tx, cb_rx) = oneshot::channel::<CallbackQuery>();
        let cb_tx = Arc::new(Mutex::new(Some(cb_tx)));
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let app = Router::new()
            .route(
                "/auth/callback",
                get(
                    |State(tx): State<Arc<Mutex<Option<oneshot::Sender<CallbackQuery>>>>>,
                     Query(q): Query<CallbackQuery>| async move {
                        if let Some(tx) = tx.lock().await.take() {
                            let _ = tx.send(q);
                        }
                        Html(CALLBACK_HTML)
                    },
                ),
            )
            .with_state(cb_tx);
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
        *self.pending.lock().await = Some(Pending { cancel: cancel_tx });
        let _ = self.progress.send(LoginProgress::WaitingBrowser {
            authorize_url: aura_core::logging::redact(&authorize_url),
        });

        let me = self.clone();
        let redirect = redirect_uri.clone();
        let handle = tokio::spawn(async move {
            let result = tokio::select! {
                q = cb_rx => match q {
                    Ok(q) => me.complete(q, &state, &nonce, &pkce.verifier, &redirect, existing).await,
                    Err(_) => Err(AuthError::Cancelled),
                },
                _ = cancel_rx => Err(AuthError::Cancelled),
                _ = tokio::time::sleep(Duration::from_secs(15 * 60)) => Err(AuthError::Cancelled),
            };
            let _ = shutdown_tx.send(());
            me.pending.lock().await.take();
            let event = match &result {
                Ok(o) => LoginProgress::Completed {
                    account: o.account.clone(),
                    first_time: o.show_welcome,
                },
                Err(AuthError::Cancelled) => LoginProgress::Cancelled,
                Err(e) => LoginProgress::Failed {
                    reason: e.to_string(),
                },
            };
            let _ = me.progress.send(event);
            result
        });
        Ok((
            LoginAttempt {
                authorize_url,
                redirect_uri,
            },
            handle,
        ))
    }

    pub async fn cancel_login(&self) {
        if let Some(p) = self.pending.lock().await.take() {
            let _ = p.cancel.send(());
        }
    }

    async fn complete(
        &self,
        q: CallbackQuery,
        state: &str,
        nonce: &str,
        verifier: &str,
        redirect_uri: &str,
        existing: Option<String>,
    ) -> Result<LoginOutcome, AuthError> {
        if q.state.as_deref() != Some(state) {
            return Err(AuthError::StateMismatch);
        }
        if let Some(err) = q.error {
            return Err(if err == "access_denied" {
                AuthError::Denied
            } else {
                AuthError::Network(err)
            });
        }
        let client_id = match (&existing, q.client_id) {
            (Some(saved), Some(returned)) if *saved != returned => {
                return Err(AuthError::ClientIdMismatch);
            }
            (Some(saved), _) => saved.clone(),
            (None, Some(issued)) if issued != "dynamic_agent_client" => issued,
            (None, _) => return Err(AuthError::RegistrationIncomplete),
        };
        let code = q.code.ok_or(AuthError::RegistrationIncomplete)?;
        let body = self
            .token_request(&[
                ("grant_type", "authorization_code"),
                ("client_id", &client_id),
                ("code", &code),
                ("code_verifier", verifier),
                ("redirect_uri", redirect_uri),
                ("resource", &self.cfg.resource),
            ])
            .await?;
        let token_set = parse_token_response(&body, None)?;
        let id_token = token_set
            .id_token
            .clone()
            .ok_or_else(|| AuthError::InvalidIdToken("missing".into()))?;
        let jwks = self.jwks().await?;
        let claims = idtoken::validate(&id_token, &jwks, &self.cfg.issuer, &client_id, nonce)
            .map_err(|e| AuthError::InvalidIdToken(e.to_string()))?;

        let repo = AccountsRepo::new(&self.store);
        let previous = repo.get(&client_id)?;
        if let Some(prev) = &previous
            && prev.subject != claims.sub
        {
            return Err(AuthError::ClientIdMismatch);
        }
        let plan_usage_enabled = token_set.scopes.iter().any(|s| s == crate::PLAN_SCOPE);
        tokens::save(self.creds.as_ref(), &client_id, &token_set)?;
        let welcomed = previous.as_ref().is_some_and(|p| p.welcomed);
        let account = ChatGptAccount {
            client_id: client_id.clone(),
            subject: claims.sub,
            email: claims.email,
            scopes: token_set.scopes.clone(),
            plan_usage_enabled,
            expires_at: Some(token_set.expires_at),
            active: true,
            welcomed,
            signed_in: true,
        };
        repo.upsert(&account)?;
        repo.set_active(&client_id)?;
        Ok(LoginOutcome {
            show_welcome: plan_usage_enabled && !welcomed,
            account,
        })
    }

    async fn jwks(&self) -> Result<Value, AuthError> {
        if let Some(v) = self.jwks_cache.lock().await.clone() {
            return Ok(v);
        }
        let v: Value = self
            .http
            .get(&self.cfg.jwks_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        *self.jwks_cache.lock().await = Some(v.clone());
        Ok(v)
    }

    async fn token_request(&self, form: &[(&str, &str)]) -> Result<Value, AuthError> {
        let resp = self
            .http
            .post(&self.cfg.token_url)
            .form(form)
            .send()
            .await?;
        let status = resp.status();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        if status.is_success() {
            return Ok(body);
        }
        let code = body["error"]
            .as_str()
            .or(body["error"]["code"].as_str())
            .unwrap_or_default();
        Err(match code {
            "invalid_grant"
            | "invalid_refresh_token"
            | "token_expired"
            | "refresh_token_expired"
            | "refresh_token_invalidated"
            | "refresh_token_reused" => AuthError::ReloginRequired,
            "invalid_client" => AuthError::InvalidClient,
            _ => AuthError::Network(format!("token endpoint HTTP {status}")),
        })
    }

    async fn lock_for(&self, client_id: &str) -> Arc<Mutex<()>> {
        self.refresh_locks
            .lock()
            .await
            .entry(client_id.to_string())
            .or_default()
            .clone()
    }

    /// Returns a valid access token of the active account, refreshing when it
    /// expires within the configured margin. Used by the gateway per request.
    pub async fn access_token(&self) -> Result<Secret<String>, AuthError> {
        let account = self.active()?.ok_or(AuthError::NotSignedIn)?;
        if !account.plan_usage_enabled {
            return Err(AuthError::PlanUsageDisabled);
        }
        self.token_for(&account.client_id, false).await
    }

    /// Forces one refresh after an upstream 401.
    pub async fn on_unauthorized(&self) -> Result<Secret<String>, AuthError> {
        let account = self.active()?.ok_or(AuthError::NotSignedIn)?;
        self.token_for(&account.client_id, true).await
    }

    async fn token_for(&self, client_id: &str, force: bool) -> Result<Secret<String>, AuthError> {
        let lock = self.lock_for(client_id).await;
        let _guard = lock.lock().await;
        let current =
            tokens::load(self.creds.as_ref(), client_id)?.ok_or(AuthError::ReloginRequired)?;
        if !force && current.expires_at - now_secs() > self.cfg.refresh_margin_secs {
            return Ok(Secret::new(current.access_token));
        }
        let refresh = current
            .refresh_token
            .clone()
            .ok_or(AuthError::ReloginRequired)?;
        let result = self
            .token_request(&[
                ("grant_type", "refresh_token"),
                ("client_id", client_id),
                ("refresh_token", &refresh),
                ("resource", &self.cfg.resource),
            ])
            .await;
        match result {
            Ok(body) => {
                let next = parse_token_response(&body, Some(&current))?;
                tokens::save(self.creds.as_ref(), client_id, &next)?;
                let repo = AccountsRepo::new(&self.store);
                if let Some(mut a) = repo.get(client_id)? {
                    a.expires_at = Some(next.expires_at);
                    a.scopes = next.scopes.clone();
                    repo.upsert(&a)?;
                }
                Ok(Secret::new(next.access_token))
            }
            Err(AuthError::ReloginRequired) => {
                tokens::clear(self.creds.as_ref(), client_id)?;
                self.mark_signed_out(client_id)?;
                Err(AuthError::ReloginRequired)
            }
            Err(e) => Err(e),
        }
    }

    fn mark_signed_out(&self, client_id: &str) -> Result<(), AuthError> {
        let repo = AccountsRepo::new(&self.store);
        if let Some(mut a) = repo.get(client_id)? {
            a.signed_in = false;
            a.expires_at = None;
            repo.upsert(&a)?;
        }
        Ok(())
    }

    /// Revokes the renewable session and clears local tokens (AC-004).
    /// Returns `false` when remote revocation could not be confirmed.
    pub async fn logout(&self, client_id: &str) -> Result<bool, AuthError> {
        let mut confirmed = false;
        if let Some(t) = tokens::load(self.creds.as_ref(), client_id)? {
            if let Some(refresh) = t.refresh_token {
                for attempt in 0..3u32 {
                    let resp = self
                        .http
                        .post(&self.cfg.revocation_url)
                        .form(&[
                            ("token", refresh.as_str()),
                            ("token_type_hint", "refresh_token"),
                            ("client_id", client_id),
                        ])
                        .send()
                        .await;
                    match resp {
                        Ok(r) if r.status().is_success() => {
                            confirmed = true;
                            break;
                        }
                        Ok(r) if r.status().is_client_error() => break,
                        _ => {
                            tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt))).await
                        }
                    }
                }
            } else {
                confirmed = true;
            }
        } else {
            confirmed = true;
        }
        tokens::clear(self.creds.as_ref(), client_id)?;
        self.mark_signed_out(client_id)?;
        Ok(confirmed)
    }
}

fn parse_token_response(body: &Value, previous: Option<&TokenSet>) -> Result<TokenSet, AuthError> {
    let access = body["access_token"]
        .as_str()
        .ok_or_else(|| AuthError::Network("no access_token".into()))?;
    let scopes = body["scope"]
        .as_str()
        .map(|s| s.split_whitespace().map(str::to_string).collect())
        .or_else(|| previous.map(|p| p.scopes.clone()))
        .unwrap_or_default();
    Ok(TokenSet {
        access_token: access.to_string(),
        refresh_token: body["refresh_token"]
            .as_str()
            .map(str::to_string)
            .or_else(|| previous.and_then(|p| p.refresh_token.clone())),
        id_token: body["id_token"]
            .as_str()
            .map(str::to_string)
            .or_else(|| previous.and_then(|p| p.id_token.clone())),
        expires_at: now_secs() + body["expires_in"].as_i64().unwrap_or(3600),
        scopes,
    })
}
