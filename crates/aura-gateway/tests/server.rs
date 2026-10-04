//! HTTP-level tests: Codex → gateway → mock upstreams.

use aura_core::Secret;
use aura_core::credentials::MemoryCredentialStore;
use aura_gateway::discovery::{ConnectionCategory, test_connection};
use aura_gateway::errors::UpstreamError;
use aura_gateway::registry::{ProviderDraft, ProviderRegistry, Wire};
use aura_gateway::server::{Routes, start};
use aura_gateway::translate::emitter::{parse_frames, validate_sequence};
use aura_gateway::upstream::chatgpt_plan::ChatGptPlanUpstream;
use aura_gateway::upstream::{BoxFut, TokenProvider};
use aura_store::Store;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Mock {
    auth_headers: Mutex<Vec<String>>,
    bodies: Mutex<Vec<Value>>,
    fail_first_with_401: Mutex<bool>,
}

async fn serve(app: Router) -> String {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    url
}

fn sse(body: &'static str) -> impl IntoResponse {
    ([("content-type", "text/event-stream")], body)
}

fn record(m: &Mock, headers: &HeaderMap, body: &Value) {
    let auth = headers
        .get("authorization")
        .or(headers.get("x-api-key"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    m.auth_headers.lock().unwrap().push(auth);
    m.bodies.lock().unwrap().push(body.clone());
}

async fn mock_openai(
    State(m): State<Arc<Mock>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> axum::response::Response {
    record(&m, &headers, &body);
    let mut fail = m.fail_first_with_401.lock().unwrap();
    if *fail {
        *fail = false;
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"message": "expired"}})),
        )
            .into_response();
    }
    sse("event: response.created\ndata: {\"type\":\"response.created\",\"response\":{\"id\":\"r1\"}}\n\n").into_response()
}

async fn mock_chat(
    State(m): State<Arc<Mock>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    record(&m, &headers, &body);
    sse(include_str!("fixtures/chat/tools-parallel.sse"))
}

async fn mock_anthropic(
    State(m): State<Arc<Mock>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    record(&m, &headers, &body);
    sse(include_str!("fixtures/anthropic/tools-thinking.sse"))
}

async fn mock_429() -> impl IntoResponse {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [("retry-after", "20")],
        Json(json!({"error": {"message": "slow down"}})),
    )
}

struct Tokens {
    refreshed: AtomicUsize,
}

impl TokenProvider for Tokens {
    fn token(&self) -> BoxFut<'_, Result<Secret<String>, UpstreamError>> {
        Box::pin(async { Ok(Secret::from("access-1")) })
    }
    fn refresh(&self) -> BoxFut<'_, Result<Secret<String>, UpstreamError>> {
        self.refreshed.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(Secret::from("access-2")) })
    }
}

const CODEX_REQUEST: &str = r#"{"model":"m","instructions":"Sys","input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"clima e hora"}]}],
 "tools":[{"type":"function","name":"get_weather","description":"d","parameters":{"type":"object"}},{"type":"function","name":"get_time","description":"t","parameters":{"type":"object"}}],
 "tool_choice":"auto","parallel_tool_calls":true,"store":false,"stream":true}"#;

async fn post_responses(
    base: &str,
    provider: &str,
    token: &str,
    extra: Option<(&str, &str)>,
) -> reqwest::Response {
    let mut req = reqwest::Client::new()
        .post(format!("{base}/p/{provider}/v1/responses"))
        .bearer_auth(token)
        .header("content-type", "application/json")
        .body(CODEX_REQUEST);
    if let Some((k, v)) = extra {
        req = req.header(k, v);
    }
    req.send().await.unwrap()
}

#[tokio::test]
async fn auth_origin_and_unknown_provider() {
    let routes = Routes::new();
    let gw = start(routes, None).await.unwrap();
    let base = gw.base_url();
    assert_eq!(
        post_responses(&base, "x", "wrong", None).await.status(),
        401
    );
    assert_eq!(
        post_responses(
            &base,
            "x",
            gw.token.expose(),
            Some(("origin", "http://evil"))
        )
        .await
        .status(),
        403
    );
    let r = post_responses(&base, "x", gw.token.expose(), None).await;
    assert_eq!(r.status(), 404);
    assert_eq!(
        r.json::<Value>().await.unwrap()["error"]["code"],
        "provider_not_found"
    );
}

#[tokio::test]
async fn chatgpt_plan_injects_token_and_refreshes_once_on_401() {
    let mock = Arc::new(Mock::default());
    *mock.fail_first_with_401.lock().unwrap() = true;
    let upstream_url = serve(
        Router::new()
            .route("/v1/responses", post(mock_openai))
            .with_state(mock.clone()),
    )
    .await;
    let tokens = Arc::new(Tokens {
        refreshed: AtomicUsize::new(0),
    });
    let mut plan = ChatGptPlanUpstream::new(reqwest::Client::new(), tokens.clone());
    plan.base_url = format!("{upstream_url}/v1");
    let routes = Routes::new();
    routes.set("chatgpt-plan", Arc::new(plan));
    let gw = start(routes, None).await.unwrap();

    let r = post_responses(&gw.base_url(), "chatgpt-plan", gw.token.expose(), None).await;
    assert_eq!(r.status(), 200);
    assert!(r.text().await.unwrap().contains("response.created"));
    assert_eq!(tokens.refreshed.load(Ordering::SeqCst), 1);
    let auths = mock.auth_headers.lock().unwrap().clone();
    assert_eq!(auths, vec!["Bearer access-1", "Bearer access-2"]);
    // The loopback token never reaches OpenAI.
    assert!(auths.iter().all(|a| !a.contains(gw.token.expose())));
}

#[tokio::test]
async fn byok_chat_provider_is_translated_end_to_end() {
    let mock = Arc::new(Mock::default());
    let upstream_url = serve(
        Router::new()
            .route("/v1/chat/completions", post(mock_chat))
            .route(
                "/v1/models",
                get(|| async { Json(json!({"data": [{"id": "llama-x"}]})) }),
            )
            .with_state(mock.clone()),
    )
    .await;
    let store = Store::open_in_memory().unwrap();
    let creds = MemoryCredentialStore::new();
    let reg = ProviderRegistry::new(&store, &creds);
    let p = reg
        .upsert(ProviderDraft {
            id: Some("groq".into()),
            name: "Groq".into(),
            preset: "groq".into(),
            wire: None,
            base_url: Some(format!("{upstream_url}/v1")),
            extra_headers: Default::default(),
            credential: Some(Secret::from("gsk-test-key")),
        })
        .unwrap();
    assert_eq!(p.wire, Wire::Chat);
    let routes = Routes::new();
    routes.set(
        "groq",
        aura_gateway::build_upstream(&reg, &p, reqwest::Client::new()).unwrap(),
    );
    let gw = start(routes, None).await.unwrap();

    let r = post_responses(&gw.base_url(), "groq", gw.token.expose(), None).await;
    assert_eq!(r.status(), 200);
    let frames = parse_frames(&r.text().await.unwrap());
    validate_sequence(&frames).unwrap();
    let out = frames.last().unwrap()["response"]["output"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(
        out.iter()
            .map(|o| o["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["get_weather", "get_time"]
    );
    assert_eq!(mock.auth_headers.lock().unwrap()[0], "Bearer gsk-test-key");
    let sent = mock.bodies.lock().unwrap()[0].clone();
    assert_eq!(
        sent["messages"][0],
        json!({"role": "system", "content": "Sys"})
    );
    assert_eq!(sent["tools"].as_array().unwrap().len(), 2);

    let conn = test_connection(&p, &reg.target(&p).unwrap()).await;
    assert_eq!(conn.category, ConnectionCategory::Ok);
    assert_eq!(conn.models[0].id, "llama-x");
}

#[tokio::test]
async fn byok_anthropic_is_translated_end_to_end() {
    let mock = Arc::new(Mock::default());
    let upstream_url = serve(
        Router::new()
            .route("/v1/messages", post(mock_anthropic))
            .with_state(mock.clone()),
    )
    .await;
    let store = Store::open_in_memory().unwrap();
    let creds = MemoryCredentialStore::new();
    let reg = ProviderRegistry::new(&store, &creds);
    let p = reg
        .upsert(ProviderDraft {
            id: Some("claude".into()),
            name: "Anthropic".into(),
            preset: "anthropic".into(),
            wire: None,
            base_url: Some(format!("{upstream_url}/v1")),
            extra_headers: Default::default(),
            credential: Some(Secret::from("sk-ant-test")),
        })
        .unwrap();
    let routes = Routes::new();
    routes.set(
        "claude",
        aura_gateway::build_upstream(&reg, &p, reqwest::Client::new()).unwrap(),
    );
    let gw = start(routes, None).await.unwrap();
    let r = post_responses(&gw.base_url(), "claude", gw.token.expose(), None).await;
    let frames = parse_frames(&r.text().await.unwrap());
    validate_sequence(&frames).unwrap();
    assert_eq!(frames.last().unwrap()["type"], "response.completed");
    assert_eq!(mock.auth_headers.lock().unwrap()[0], "sk-ant-test");
    assert_eq!(mock.bodies.lock().unwrap()[0]["system"], "Sys");
}

#[tokio::test]
async fn upstream_429_is_forwarded_with_retry_after() {
    let upstream_url = serve(Router::new().route("/v1/chat/completions", post(mock_429))).await;
    let store = Store::open_in_memory().unwrap();
    let creds = MemoryCredentialStore::new();
    let reg = ProviderRegistry::new(&store, &creds);
    let p = reg
        .upsert(ProviderDraft {
            id: Some("x".into()),
            name: "X".into(),
            preset: "custom".into(),
            wire: Some(Wire::Chat),
            base_url: Some(format!("{upstream_url}/v1")),
            extra_headers: Default::default(),
            credential: None,
        })
        .unwrap();
    let routes = Routes::new();
    routes.set(
        "x",
        aura_gateway::build_upstream(&reg, &p, reqwest::Client::new()).unwrap(),
    );
    let gw = start(routes, None).await.unwrap();
    let r = post_responses(&gw.base_url(), "x", gw.token.expose(), None).await;
    assert_eq!(r.status(), 429);
    assert_eq!(r.headers()["retry-after"], "20");
    assert_eq!(
        r.json::<Value>().await.unwrap()["error"]["code"],
        "rate_limit_exceeded"
    );
}

#[tokio::test]
async fn connection_test_categories() {
    let upstream_url = serve(
        Router::new()
            .route("/bad/models", get(|| async { StatusCode::UNAUTHORIZED }))
            .route("/gone/x", get(|| async { StatusCode::OK })),
    )
    .await;
    let store = Store::open_in_memory().unwrap();
    let creds = MemoryCredentialStore::new();
    let reg = ProviderRegistry::new(&store, &creds);
    for (path, expected) in [
        ("bad", ConnectionCategory::Unauthorized),
        ("gone", ConnectionCategory::NotFound),
    ] {
        let p = reg
            .upsert(ProviderDraft {
                id: Some(path.into()),
                name: path.into(),
                preset: "custom".into(),
                wire: Some(Wire::Chat),
                base_url: Some(format!("{upstream_url}/{path}")),
                extra_headers: Default::default(),
                credential: None,
            })
            .unwrap();
        assert_eq!(
            test_connection(&p, &reg.target(&p).unwrap()).await.category,
            expected
        );
    }
    let p = reg
        .upsert(ProviderDraft {
            id: Some("down".into()),
            name: "down".into(),
            preset: "custom".into(),
            wire: Some(Wire::Chat),
            base_url: Some("http://127.0.0.1:9/v1".into()),
            extra_headers: Default::default(),
            credential: None,
        })
        .unwrap();
    assert_eq!(
        test_connection(&p, &reg.target(&p).unwrap()).await.category,
        ConnectionCategory::Network
    );
}

#[tokio::test]
async fn turns_with_several_images_fit_the_body_limit() {
    // QA-033: 8 keyframes of a clip (~1–2 MB each as base64) in one turn;
    // axum's default 2 MB limit answered 413 before routing.
    let gw = start(Routes::new(), None).await.unwrap();
    let image = "A".repeat(2 * 1024 * 1024);
    let parts: Vec<Value> = (0..8)
        .map(|_| json!({"type": "input_image", "image_url": format!("data:image/png;base64,{image}")}))
        .collect();
    let body = json!({"model": "m", "input": [{"role": "user", "content": parts}]}).to_string();
    let r = reqwest::Client::new()
        .post(format!("{}/p/none/v1/responses", gw.base_url()))
        .bearer_auth(gw.token.expose())
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        r.status(),
        404,
        "routed (unknown provider), not rejected by size"
    );
}
