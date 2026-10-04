//! End-to-end with the REAL pinned Codex app-server (no account needed):
//! Aura host → app-server → Aura gateway (Chat Completions translation, BYOK
//! key injection) → mock provider; the mock makes the model call Aura's MCP
//! tool `active_window_info`, which Codex reaches through `/mcp`.
//!
//! Ignored by default. Run with the binary of the pinned release:
//! `AURA_CODEX_BIN=/path/to/codex-app-server cargo test -p aura-app --test real_app_server -- --ignored --nocapture`

use aura_app::events::HostEvent;
use aura_app::host::SendRequest;
use aura_app::paths::AppPaths;
use aura_app::platform::Platform;
use aura_app::{CodexRuntime, Host, HostConfig};
use aura_codex::events::ConversationEvent;
use aura_codex::service::StartOptions;
use axum::Router;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct Seen {
    requests: Vec<Value>,
    auth: Vec<String>,
}

fn chunk(delta: Value, finish: Option<&str>) -> String {
    let v = json!({"id": "chatcmpl-1", "object": "chat.completion.chunk", "created": 0, "model": "mock-model",
        "choices": [{"index": 0, "delta": delta, "finish_reason": finish}]});
    format!("data: {v}\n\n")
}

async fn completions(seen: Arc<Mutex<Seen>>, headers: HeaderMap, body: Value) -> impl IntoResponse {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let messages = body["messages"].as_array().cloned().unwrap_or_default();
    let tool_name = body["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t["function"]["name"].as_str())
        .find(|n| n.contains("active_window_info"))
        .map(str::to_string);
    {
        let mut s = seen.lock().unwrap();
        s.requests.push(body.clone());
        s.auth.push(auth);
    }
    let last_role = messages
        .last()
        .and_then(|m| m["role"].as_str())
        .unwrap_or_default()
        .to_string();
    let wants_window = messages
        .iter()
        .any(|m| m["role"] == "user" && m["content"].to_string().contains("janela"));
    let mut sse = String::new();
    match (last_role.as_str(), tool_name) {
        ("tool", _) => {
            let tool_text = messages
                .last()
                .map(|m| m["content"].to_string())
                .unwrap_or_default();
            let app = if tool_text.contains("Code.exe") {
                "Code.exe"
            } else {
                "desconhecido"
            };
            sse.push_str(&chunk(
                json!({"role": "assistant", "content": format!("pong: a janela ativa é {app}")}),
                None,
            ));
            sse.push_str(&chunk(json!({}), Some("stop")));
        }
        (_, Some(name)) if wants_window => {
            sse.push_str(&chunk(
                json!({"role": "assistant", "tool_calls": [{"index": 0, "id": "call_1", "type": "function",
                    "function": {"name": name, "arguments": "{}"}}]}),
                None,
            ));
            sse.push_str(&chunk(json!({}), Some("tool_calls")));
        }
        _ => {
            sse.push_str(&chunk(
                json!({"role": "assistant", "content": "pong"}),
                None,
            ));
            sse.push_str(&chunk(json!({}), Some("stop")));
        }
    }
    sse.push_str("data: [DONE]\n\n");
    ([("content-type", "text/event-stream")], sse)
}

async fn mock_provider() -> (String, Arc<Mutex<Seen>>) {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let s2 = seen.clone();
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(move |h: HeaderMap, axum::Json(b): axum::Json<Value>| {
                completions(s2.clone(), h, b)
            }),
        )
        .route(
            "/v1/models",
            get(|| async {
                axum::Json(
                    json!({"object": "list", "data": [{"id": "mock-model", "object": "model"}]}),
                )
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://127.0.0.1:{port}/v1"), seen)
}

async fn turn(
    host: &Host,
    rx: &mut tokio::sync::broadcast::Receiver<HostEvent>,
    thread: &str,
    text: &str,
) -> (String, Vec<String>) {
    host.send(SendRequest {
        thread_id: thread.into(),
        text: text.into(),
        tray: thread.into(),
        accepts_images: false,
        options: Default::default(),
    })
    .await
    .expect("send");
    let mut answer = String::new();
    let mut tools = Vec::new();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(120), rx.recv())
            .await
            .expect("turn finished in time")
            .expect("open");
        match e {
            HostEvent::Conversation(ConversationEvent::MessageDelta { delta, .. }) => {
                answer.push_str(&delta)
            }
            HostEvent::Conversation(ConversationEvent::MessageCompleted { text, .. }) => {
                answer = text
            }
            HostEvent::Conversation(ConversationEvent::ToolCall { title, status, .. }) => {
                tools.push(format!("{title}:{status:?}"))
            }
            HostEvent::Conversation(ConversationEvent::TurnCompleted { status, error, .. }) => {
                assert!(error.is_none(), "turn failed: {error:?} ({status:?})");
                return (answer, tools);
            }
            _ => {}
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs AURA_CODEX_BIN (real app-server)"]
async fn real_app_server_byok_turn_and_mcp_tool() {
    let Some(bin) = std::env::var_os("AURA_CODEX_BIN") else {
        return;
    };
    let (base_url, seen) = mock_provider().await;
    let dir = tempfile::Builder::new()
        .prefix("aura-e2e")
        .tempdir_in(std::env::var("AURA_E2E_DIR").unwrap_or_else(|_| ".".into()))
        .unwrap();
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    cfg.codex = CodexRuntime::Binary {
        program: Some(bin.into()),
        on_spawn: None,
    };
    let host = Host::start(cfg).await.expect("host");

    // Reproduce the mounted Overlay: catalog loading starts the engine before
    // the first provider is registered. New conversations must work immediately.
    let warming = host
        .save_provider(
            serde_json::from_value(
                json!({"name": "Warming", "preset": "custom", "baseUrl": base_url}),
            )
            .unwrap(),
            Some("sk-test-warming".into()),
        )
        .unwrap();
    host.start_conversation(StartOptions {
        provider: format!("aura-{}", warming.id),
        model: Some("mock-model".into()),
        ..Default::default()
    })
    .await
    .expect("warm engine with an existing provider");

    let draft =
        serde_json::from_value(json!({"name": "Mock", "preset": "custom", "baseUrl": base_url}))
            .unwrap();
    let provider = host
        .save_provider(draft, Some("sk-test-e2e".into()))
        .expect("provider");
    let mut rx = host.subscribe();
    let conv = host
        .start_conversation(StartOptions {
            provider: format!("aura-{}", provider.id),
            model: Some("mock-model".into()),
            ..Default::default()
        })
        .await
        .expect("thread/start on the real app-server");

    // 1) Plain turn through the gateway translation.
    let (answer, _) = turn(&host, &mut rx, &conv.thread_id, "ping").await;
    assert_eq!(answer.trim(), "pong");
    {
        let s = seen.lock().unwrap();
        assert!(
            s.auth.iter().all(|a| a == "Bearer sk-test-e2e"),
            "gateway must inject the vault key: {:?}",
            s.auth
        );
        assert_eq!(s.requests[0]["model"], "mock-model");
    }

    // 2) The model calls Aura's MCP tool; Codex reaches /mcp with the per-run token.
    let (answer, tools) = turn(&host, &mut rx, &conv.thread_id, "qual janela está ativa?").await;
    eprintln!("tools: {tools:?}\nanswer: {answer}");
    {
        let s = seen.lock().unwrap();
        let last = s.requests.last().unwrap();
        eprintln!(
            "tool names: {:?}",
            last["tools"].as_array().map(|a| a
                .iter()
                .filter_map(|t| t["function"]["name"].as_str())
                .collect::<Vec<_>>())
        );
        eprintln!(
            "last messages: {}",
            serde_json::to_string_pretty(
                &last["messages"]
                    .as_array()
                    .map(|m| m.iter().rev().take(2).collect::<Vec<_>>())
            )
            .unwrap()
        );
    }
    assert!(
        answer.contains("Code.exe"),
        "tool result must reach the model: {answer}"
    );
    host.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "probe: prints the raw Responses request Codex sends"]
async fn probe_raw_responses_request() {
    let Some(bin) = std::env::var_os("AURA_CODEX_BIN") else {
        return;
    };
    let captured = Arc::new(Mutex::new(None::<Value>));
    let c2 = captured.clone();
    let app = Router::new()
        .route(
            "/v1/responses",
            post(move |axum::Json(b): axum::Json<Value>| {
                let c = c2.clone();
                async move {
                    *c.lock().unwrap() = Some(b);
                    (
                        axum::http::StatusCode::BAD_REQUEST,
                        axum::Json(
                            json!({"error": {"message": "probe", "type": "invalid_request_error"}}),
                        ),
                    )
                }
            }),
        )
        .route(
            "/v1/models",
            get(|| async {
                axum::Json(
                    json!({"object": "list", "data": [{"id": "mock-model", "object": "model"}]}),
                )
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let dir = tempfile::Builder::new()
        .prefix("aura-probe")
        .tempdir_in(std::env::var("AURA_E2E_DIR").unwrap_or_else(|_| ".".into()))
        .unwrap();
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    cfg.codex = CodexRuntime::Binary {
        program: Some(bin.into()),
        on_spawn: None,
    };
    let host = Host::start(cfg).await.unwrap();
    let draft = serde_json::from_value(json!({"name": "Probe", "preset": "openai", "baseUrl": format!("http://127.0.0.1:{port}/v1")})).unwrap();
    let p = host.save_provider(draft, Some("sk-x".into())).unwrap();
    let mut rx = host.subscribe();
    let conv = host
        .start_conversation(StartOptions {
            provider: format!("aura-{}", p.id),
            model: Some("gpt-5.5".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    for text in ["oi", "de novo"] {
        host.send(SendRequest {
            thread_id: conv.thread_id.clone(),
            text: text.into(),
            tray: conv.thread_id.clone(),
            accepts_images: false,
            options: Default::default(),
        })
        .await
        .unwrap();
        let _ = tokio::time::timeout(Duration::from_secs(60), async {
            while let Ok(e) = rx.recv().await {
                if matches!(
                    e,
                    HostEvent::Conversation(ConversationEvent::TurnCompleted { .. })
                ) {
                    break;
                }
            }
        })
        .await;
    }
    let _ = tokio::time::timeout(Duration::from_secs(60), async {
        while let Ok(e) = rx.recv().await {
            if matches!(
                e,
                HostEvent::Conversation(ConversationEvent::TurnCompleted { .. })
            ) {
                break;
            }
        }
    })
    .await;
    let body = captured.lock().unwrap().clone().expect("request captured");
    let cfg_text = std::fs::read_to_string(
        dir.path()
            .join("Aura")
            .join("codex-home")
            .join("config.toml"),
    )
    .unwrap_or_default();
    eprintln!(
        "CONFIG features: {:?}",
        cfg_text
            .lines()
            .skip_while(|l| !l.starts_with("[features]"))
            .take(4)
            .collect::<Vec<_>>()
    );
    for t in body["tools"].as_array().unwrap() {
        let nested: Vec<String> = t["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|x| format!("{}:{}", x["type"], x["name"]))
            .collect();
        let keys: Vec<&String> = t
            .as_object()
            .map(|o| o.keys().collect())
            .unwrap_or_default();
        eprintln!(
            "TOOL type={} name={} keys={:?} nested={:?}",
            t["type"], t["name"], keys, nested
        );
    }
    host.shutdown().await;
}
