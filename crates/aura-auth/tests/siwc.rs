//! SIWC flow against a local mock of auth.openai.com.

use aura_auth::{AuthError, AuthService, LoginOutcome, SiwcConfig};
use aura_core::credentials::{CredentialStore, MemoryCredentialStore};
use aura_store::Store;
use axum::extract::{Form, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MockState {
    /// Form bodies received by the token endpoint.
    token_calls: Vec<HashMap<String, String>>,
    revoke_calls: Vec<HashMap<String, String>>,
    nonce: String,
    challenge: String,
    scope: String,
    expires_in: i64,
    refresh_fails: bool,
    access_counter: u32,
    subject: String,
}

type Shared = Arc<Mutex<MockState>>;

fn sign(claims: Value) -> String {
    let key = EncodingKey::from_rsa_pem(include_bytes!("fixtures/test-signing-key.pem")).unwrap();
    let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("test-key".into());
    encode(&header, &claims, &key).unwrap()
}

async fn token(
    State(s): State<Shared>,
    Form(form): Form<HashMap<String, String>>,
) -> (axum::http::StatusCode, Json<Value>) {
    let mut st = s.lock().unwrap();
    st.token_calls.push(form.clone());
    let client_id = form.get("client_id").cloned().unwrap_or_default();
    match form.get("grant_type").map(String::as_str) {
        Some("authorization_code") => {
            let verifier = form.get("code_verifier").cloned().unwrap_or_default();
            use base64::Engine;
            use sha2::Digest;
            let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(sha2::Sha256::digest(verifier.as_bytes()));
            if challenge != st.challenge {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid_grant"})),
                );
            }
            st.access_counter += 1;
            let now = aura_store::now_secs();
            let id_token = sign(
                json!({"iss": "https://auth.test", "aud": client_id, "sub": st.subject,
                "email": "teste@exemplo.com", "nonce": st.nonce, "iat": now, "exp": now + 3600}),
            );
            (
                axum::http::StatusCode::OK,
                Json(json!({
                "access_token": format!("access-{}", st.access_counter), "refresh_token": "refresh-1",
                "id_token": id_token, "token_type": "Bearer", "expires_in": st.expires_in, "scope": st.scope})),
            )
        }
        Some("refresh_token") => {
            if st.refresh_fails {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid_grant"})),
                );
            }
            st.access_counter += 1;
            (
                axum::http::StatusCode::OK,
                Json(json!({
                "access_token": format!("access-{}", st.access_counter), "refresh_token": "refresh-2",
                "token_type": "Bearer", "expires_in": 3600})),
            )
        }
        _ => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "unsupported_grant_type"})),
        ),
    }
}

async fn revoke(
    State(s): State<Shared>,
    Form(form): Form<HashMap<String, String>>,
) -> &'static str {
    s.lock().unwrap().revoke_calls.push(form);
    ""
}

async fn jwks() -> Json<Value> {
    Json(serde_json::from_str(include_str!("fixtures/test-jwks.json")).unwrap())
}

struct Env {
    svc: Arc<AuthService>,
    creds: Arc<MemoryCredentialStore>,
    state: Shared,
}

async fn env(scope: &str, expires_in: i64) -> Env {
    let state: Shared = Arc::new(Mutex::new(MockState {
        scope: scope.into(),
        expires_in,
        subject: "user-sub-1".into(),
        ..Default::default()
    }));
    let app = Router::new()
        .route("/token", post(token))
        .route("/revoke", post(revoke))
        .route("/jwks", get(jwks))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let cfg = SiwcConfig {
        authorize_url: format!("{base}/authorize"),
        token_url: format!("{base}/token"),
        revocation_url: format!("{base}/revoke"),
        jwks_url: format!("{base}/jwks"),
        issuer: "https://auth.test".into(),
        preferred_port: 0,
        ..Default::default()
    };
    let creds = Arc::new(MemoryCredentialStore::new());
    let svc = AuthService::new(cfg, Store::open_in_memory().unwrap(), creds.clone());
    Env { svc, creds, state }
}

fn params(url: &str) -> HashMap<String, String> {
    url::Url::parse(url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

/// Simulates the browser: records nonce/challenge in the mock and calls the
/// loopback callback like OpenAI's redirect would.
async fn browser(e: &Env, authorize_url: &str, redirect: &str, extra: &[(&str, &str)]) {
    let p = params(authorize_url);
    {
        let mut st = e.state.lock().unwrap();
        st.nonce = p["nonce"].clone();
        st.challenge = p["code_challenge"].clone();
    }
    let mut url = url::Url::parse(redirect).unwrap();
    url.query_pairs_mut().append_pair("state", &p["state"]);
    for (k, v) in extra {
        url.query_pairs_mut().append_pair(k, v);
    }
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    assert!(body.contains("Aura"));
}

const FULL: &str = "chatgpt.tokens.use.direct email offline_access openid profile resource.invoke";

async fn first_login(e: &Env) -> LoginOutcome {
    let (attempt, handle) = e.svc.begin_login(None, false).await.unwrap();
    browser(
        e,
        &attempt.authorize_url,
        &attempt.redirect_uri,
        &[("code", "c1"), ("client_id", "oaiapp_123")],
    )
    .await;
    handle.await.unwrap().unwrap()
}

#[tokio::test]
async fn first_sign_in_registers_the_client_and_enables_plan_usage() {
    let e = env(FULL, 3600).await;
    let (attempt, handle) = e.svc.begin_login(None, false).await.unwrap();
    let p = params(&attempt.authorize_url);
    assert_eq!(p["client_id"], "dynamic_agent_client");
    assert_eq!(p["agent_name_hint"], "Aura");
    assert!(p["ext_agent_host_id"].starts_with("urn:uuid:"));
    assert_eq!(p["code_challenge_method"], "S256");
    assert_eq!(p["resource"], "https://api.openai.com/v1");
    assert_eq!(
        p["scope"],
        "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct"
    );
    assert!(
        p["redirect_uri"].starts_with("http://127.0.0.1:")
            && p["redirect_uri"].ends_with("/auth/callback")
    );

    browser(
        &e,
        &attempt.authorize_url,
        &attempt.redirect_uri,
        &[("code", "c1"), ("client_id", "oaiapp_123")],
    )
    .await;
    let out = handle.await.unwrap().unwrap();
    assert!(out.show_welcome);
    assert!(out.account.plan_usage_enabled);
    assert_eq!(out.account.email.as_deref(), Some("teste@exemplo.com"));
    assert_eq!(e.svc.active().unwrap().unwrap().client_id, "oaiapp_123");
    // The code exchange used the issued client id and no secret.
    let call = e.state.lock().unwrap().token_calls[0].clone();
    assert_eq!(call["client_id"], "oaiapp_123");
    assert!(!call.contains_key("client_secret"));
    assert_eq!(call["redirect_uri"], p["redirect_uri"]);
    // Tokens only in the credential store.
    assert!(e.creds.get("Aura/chatgpt/oaiapp_123").unwrap().is_some());
    assert_eq!(e.svc.access_token().await.unwrap().expose(), "access-1");
}

#[tokio::test]
async fn returning_sign_in_uses_hints_and_skips_welcome() {
    let e = env(FULL, 3600).await;
    first_login(&e).await;
    e.svc.mark_welcomed("oaiapp_123").unwrap();
    let (attempt, handle) = e
        .svc
        .begin_login(Some("oaiapp_123".into()), false)
        .await
        .unwrap();
    let p = params(&attempt.authorize_url);
    assert_eq!(p["client_id"], "oaiapp_123");
    assert!(!p.contains_key("agent_name_hint"));
    assert!(p.contains_key("id_token_hint"));
    assert_eq!(p["login_hint"], "teste@exemplo.com");
    browser(
        &e,
        &attempt.authorize_url,
        &attempt.redirect_uri,
        &[("code", "c2")],
    )
    .await;
    let out = handle.await.unwrap().unwrap();
    assert!(!out.show_welcome);
}

#[tokio::test]
async fn declined_consent_and_missing_plan_scope() {
    let e = env("openid profile email offline_access", 3600).await;
    let (attempt, handle) = e.svc.begin_login(None, false).await.unwrap();
    browser(
        &e,
        &attempt.authorize_url,
        &attempt.redirect_uri,
        &[("error", "access_denied")],
    )
    .await;
    assert_eq!(handle.await.unwrap().unwrap_err(), AuthError::Denied);
    assert!(e.state.lock().unwrap().token_calls.is_empty());

    let out = first_login(&e).await;
    assert!(!out.account.plan_usage_enabled);
    assert!(!out.show_welcome);
    assert_eq!(
        e.svc.access_token().await.unwrap_err(),
        AuthError::PlanUsageDisabled
    );
}

#[tokio::test]
async fn state_mismatch_is_rejected() {
    let e = env(FULL, 3600).await;
    let (attempt, handle) = e.svc.begin_login(None, false).await.unwrap();
    let url = format!(
        "{}?state=forged&code=c&client_id=oaiapp_123",
        attempt.redirect_uri
    );
    reqwest::get(url).await.unwrap();
    assert_eq!(handle.await.unwrap().unwrap_err(), AuthError::StateMismatch);
}

#[tokio::test]
async fn access_token_refreshes_near_expiry() {
    let e = env(FULL, 100).await; // below the 300 s margin
    first_login(&e).await;
    assert_eq!(e.svc.access_token().await.unwrap().expose(), "access-2");
    let calls = e.state.lock().unwrap().token_calls.clone();
    let refresh = calls
        .iter()
        .find(|c| c["grant_type"] == "refresh_token")
        .unwrap();
    assert_eq!(refresh["client_id"], "oaiapp_123");
    assert_eq!(refresh["resource"], "https://api.openai.com/v1");
    assert!(!refresh.contains_key("scope"));
    // The refreshed token is now valid for an hour: no new refresh.
    assert_eq!(e.svc.access_token().await.unwrap().expose(), "access-2");
}

#[tokio::test]
async fn invalid_grant_on_refresh_requires_sign_in_again() {
    let e = env(FULL, 100).await;
    first_login(&e).await;
    e.state.lock().unwrap().refresh_fails = true;
    assert_eq!(
        e.svc.access_token().await.unwrap_err(),
        AuthError::ReloginRequired
    );
    assert!(e.svc.active().unwrap().is_none());
    assert!(e.creds.get("Aura/chatgpt/oaiapp_123").unwrap().is_none());
}

#[tokio::test]
async fn logout_revokes_and_keeps_the_registration() {
    let e = env(FULL, 3600).await;
    first_login(&e).await;
    assert!(e.svc.logout("oaiapp_123").await.unwrap());
    let revoke = e.state.lock().unwrap().revoke_calls[0].clone();
    assert_eq!(revoke["token"], "refresh-1");
    assert_eq!(revoke["token_type_hint"], "refresh_token");
    assert_eq!(revoke["client_id"], "oaiapp_123");
    assert!(e.creds.get("Aura/chatgpt/oaiapp_123").unwrap().is_none());
    let accounts = e.svc.accounts().unwrap();
    assert_eq!(accounts.len(), 1);
    assert!(!accounts[0].signed_in);
    assert_eq!(
        e.svc.access_token().await.unwrap_err(),
        AuthError::NotSignedIn
    );
}

#[tokio::test]
async fn cancel_frees_the_listener_and_a_new_attempt_works() {
    let e = env(FULL, 3600).await;
    let (_attempt, handle) = e.svc.begin_login(None, false).await.unwrap();
    e.svc.cancel_login().await;
    assert_eq!(handle.await.unwrap().unwrap_err(), AuthError::Cancelled);
    first_login(&e).await;
}

#[tokio::test]
async fn two_accounts_can_be_switched() {
    let e = env(FULL, 3600).await;
    first_login(&e).await;
    e.state.lock().unwrap().subject = "user-sub-2".into();
    let (attempt, handle) = e.svc.begin_login(None, false).await.unwrap();
    browser(
        &e,
        &attempt.authorize_url,
        &attempt.redirect_uri,
        &[("code", "c3"), ("client_id", "oaiapp_456")],
    )
    .await;
    handle.await.unwrap().unwrap();
    assert_eq!(e.svc.active().unwrap().unwrap().client_id, "oaiapp_456");
    e.svc.switch("oaiapp_123").unwrap();
    assert_eq!(e.svc.active().unwrap().unwrap().client_id, "oaiapp_123");
    assert!(e.creds.get("Aura/chatgpt/oaiapp_456").unwrap().is_some());
}
