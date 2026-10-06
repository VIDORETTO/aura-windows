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
    // 017: "crie o comando" makes the model call Aura's quick_command_save.
    let save_tool = body["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| t["function"]["name"].as_str())
        .find(|n| n.contains("quick_command_save"))
        .map(str::to_string);
    let wants_command = messages.last().is_some_and(|m| {
        m["role"] == "user" && m["content"].to_string().contains("crie o comando")
    });
    let mut sse = String::new();
    match (last_role.as_str(), tool_name) {
        ("tool", _) if save_tool.is_some() && !wants_window => {
            let tool_text = messages
                .last()
                .map(|m| m["content"].to_string())
                .unwrap_or_default();
            sse.push_str(&chunk(
                json!({"role": "assistant", "content": format!("resultado: {tool_text}")}),
                None,
            ));
            sse.push_str(&chunk(json!({}), Some("stop")));
        }
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
        _ if wants_command && save_tool.is_some() => {
            let name = save_tool.unwrap();
            sse.push_str(&chunk(
                json!({"role": "assistant", "tool_calls": [{"index": 0, "id": "call_q", "type": "function",
                    "function": {"name": name, "arguments": "{\"name\":\"qa-formal\",\"template\":\"Reescreva formal: {texto}\"}"}}]}),
                None,
            ));
            sse.push_str(&chunk(json!({}), Some("tool_calls")));
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
    let bin = std::env::var_os("AURA_CODEX_BIN")
        .expect("set AURA_CODEX_BIN to the pinned app-server before running ignored E2E tests");
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
    let bin = std::env::var_os("AURA_CODEX_BIN")
        .expect("set AURA_CODEX_BIN to the pinned app-server before running ignored E2E tests");
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

/// Task mode must not write outside the conversation workspace and granted
/// folders without an Approval. The mock model asks the shell to create a
/// file in a folder that is neither; the turn must request approval (we
/// decline) or the sandbox must deny the write.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs AURA_CODEX_BIN (real app-server)"]
async fn real_app_server_task_mode_confines_writes() {
    use aura_codex::modes::ConversationMode;
    let bin = std::env::var_os("AURA_CODEX_BIN")
        .expect("set AURA_CODEX_BIN to the pinned app-server before running ignored E2E tests");
    let dir = tempfile::Builder::new()
        .prefix("aura-sbx")
        .tempdir_in(std::env::var("AURA_E2E_DIR").unwrap_or_else(|_| ".".into()))
        .unwrap();
    // AURA_SBX_OUTSIDE picks another folder (e.g. the real Desktop).
    let outside = std::env::var_os("AURA_SBX_OUTSIDE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| dir.path().join("outside"));
    std::fs::create_dir_all(&outside).unwrap();
    let target = outside.join("aura-sbx-escaped.txt");
    let _ = std::fs::remove_file(&target);

    let tools_seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let ts = tools_seen.clone();
    let target_s = target.display().to_string();
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(move |axum::Json(b): axum::Json<Value>| {
                let ts = ts.clone();
                let target_s = target_s.clone();
                async move {
                    let names: Vec<String> = b["tools"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|t| t["function"]["name"].as_str().map(str::to_string))
                        .collect();
                    *ts.lock().unwrap() = names.clone();
                    let escalate = b["messages"].to_string().contains("escalado");
                    let last_role = b["messages"]
                        .as_array()
                        .and_then(|m| m.last())
                        .and_then(|m| m["role"].as_str())
                        .unwrap_or_default()
                        .to_string();
                    let shell = names
                        .iter()
                        .find(|n| *n == "shell_command" || *n == "exec_command" || *n == "shell")
                        .cloned();
                    let mut sse = String::new();
                    match (last_role.as_str(), shell) {
                        ("tool", _) | (_, None) => {
                            sse.push_str(&chunk(json!({"role": "assistant", "content": "done"}), None));
                            sse.push_str(&chunk(json!({}), Some("stop")));
                        }
                        (_, Some(name)) => {
                            let script = format!("Set-Content -LiteralPath '{target_s}' -Value escaped");
                            let args = match name.as_str() {
                                "exec_command" if escalate => json!({"cmd": script,
                                    "sandbox_permissions": "require_escalated", "justification": "teste"}),
                                "exec_command" => json!({"cmd": script}),
                                "shell_command" => json!({"command": script}),
                                _ => json!({"command": ["powershell", "-NoProfile", "-Command", script]}),
                            };
                            sse.push_str(&chunk(
                                json!({"role": "assistant", "tool_calls": [{"index": 0, "id": "call_w", "type": "function",
                                    "function": {"name": name, "arguments": args.to_string()}}]}),
                                None,
                            ));
                            sse.push_str(&chunk(json!({}), Some("tool_calls")));
                        }
                    }
                    sse.push_str("data: [DONE]\n\n");
                    ([("content-type", "text/event-stream")], sse)
                }
            }),
        )
        .route(
            "/v1/models",
            get(|| async {
                axum::Json(json!({"object": "list", "data": [{"id": "mock-model", "object": "model"}]}))
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform);
    cfg.in_memory_store = true;
    cfg.codex = CodexRuntime::Binary {
        program: Some(bin.into()),
        on_spawn: None,
    };
    let host = Host::start(cfg).await.expect("host");
    let draft = serde_json::from_value(
        json!({"name": "Mock", "preset": "custom", "baseUrl": format!("http://127.0.0.1:{port}/v1")}),
    )
    .unwrap();
    let provider = host.save_provider(draft, Some("sk-x".into())).unwrap();
    let mut rx = host.subscribe();
    let conv = host
        .start_conversation(StartOptions {
            mode: ConversationMode::Task {
                granted: vec![],
                network: false,
            },
            provider: format!("aura-{}", provider.id),
            model: Some("mock-model".into()),
            ..Default::default()
        })
        .await
        .expect("thread/start");
    let mut approvals = Vec::new();
    let mut commands = Vec::new();
    // 1) Direct write: the sandbox must deny it. 2) Escalation: Approval, declined.
    for text in ["crie o arquivo", "crie o arquivo escalado"] {
        host.send(SendRequest {
            thread_id: conv.thread_id.clone(),
            text: text.into(),
            tray: conv.thread_id.clone(),
            accepts_images: false,
            options: Default::default(),
        })
        .await
        .unwrap();
        loop {
            let e = tokio::time::timeout(Duration::from_secs(120), rx.recv())
                .await
                .expect("turn finished in time")
                .expect("open");
            match e {
                HostEvent::Conversation(ConversationEvent::ApprovalRequested {
                    request_id,
                    command,
                    ..
                }) => {
                    approvals.push(command.unwrap_or_default());
                    host.respond(&request_id, aura_codex::approvals::Decision::Decline)
                        .await
                        .unwrap();
                }
                HostEvent::Conversation(ConversationEvent::TurnCompleted { .. }) => break,
                HostEvent::Conversation(ev) => {
                    let s = format!("{ev:?}");
                    if s.contains("Command") {
                        commands.push(s);
                    }
                }
                _ => {}
            }
        }
        assert!(!target.exists(), "{text}: wrote outside the workspace");
    }
    assert_eq!(
        approvals.len(),
        1,
        "escalation must ask for approval: {approvals:?}"
    );
    eprintln!("tools offered: {:?}", tools_seen.lock().unwrap());
    eprintln!("approvals: {approvals:?}");
    for c in &commands {
        eprintln!("cmd: {}", &c[..c.len().min(400)]);
    }
    host.shutdown().await;
    let escaped = target.exists();
    let _ = std::fs::remove_file(&target);
    assert!(
        !escaped,
        "Task mode wrote outside the workspace without approval: {}",
        target.display()
    );
}

/// History: archive → listed only among archived → `thread/unarchive` → active again.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs AURA_CODEX_BIN (real app-server)"]
async fn real_app_server_archive_and_unarchive() {
    let bin = std::env::var_os("AURA_CODEX_BIN")
        .expect("set AURA_CODEX_BIN to the pinned app-server before running ignored E2E tests");
    let (base_url, _seen) = mock_provider().await;
    let dir = tempfile::Builder::new()
        .prefix("aura-arch")
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
    let draft =
        serde_json::from_value(json!({"name": "Mock", "preset": "custom", "baseUrl": base_url}))
            .unwrap();
    let provider = host.save_provider(draft, Some("sk-x".into())).unwrap();
    let mut rx = host.subscribe();
    let conv = host
        .start_conversation(StartOptions {
            provider: format!("aura-{}", provider.id),
            model: Some("mock-model".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    turn(&host, &mut rx, &conv.thread_id, "ping").await;
    let listed = |archived: bool| {
        let host = &host;
        async move {
            host.history(aura_codex::service::HistoryQuery {
                archived,
                ..Default::default()
            })
            .await
            .unwrap()
            .items
            .into_iter()
            .map(|i| i.id)
            .collect::<Vec<_>>()
        }
    };
    assert!(listed(false).await.contains(&conv.thread_id));
    host.archive(&conv.thread_id).await.unwrap();
    assert!(!listed(false).await.contains(&conv.thread_id));
    assert!(listed(true).await.contains(&conv.thread_id));
    host.unarchive(&conv.thread_id).await.unwrap();
    assert!(listed(false).await.contains(&conv.thread_id));
    assert!(!listed(true).await.contains(&conv.thread_id));
    host.shutdown().await;
}

/// 017 AC-006: the generated config (`approval_mode = "prompt"` for Aura's
/// write tools) is accepted by the pinned app-server, the user is asked
/// before `quick_command_save` runs, and approving it saves the command.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs AURA_CODEX_BIN (real app-server)"]
async fn real_app_server_asks_before_aura_write_tools() {
    let bin = std::env::var_os("AURA_CODEX_BIN")
        .expect("set AURA_CODEX_BIN to the pinned app-server before running ignored E2E tests");
    let (base_url, _seen) = mock_provider().await;
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
    let provider = host
        .save_provider(
            serde_json::from_value(
                json!({"name": "Mock", "preset": "custom", "baseUrl": base_url}),
            )
            .unwrap(),
            Some("sk-test-e2e".into()),
        )
        .unwrap();
    let mut rx = host.subscribe();
    let conv = host
        .start_conversation(StartOptions {
            provider: format!("aura-{}", provider.id),
            model: Some("mock-model".into()),
            ..Default::default()
        })
        .await
        .expect("thread/start accepts the config");
    let config = std::fs::read_to_string(dir.path().join("Aura/codex-home/config.toml")).unwrap();
    assert!(config.contains("approval_mode = \"prompt\""), "{config}");

    host.send(SendRequest {
        thread_id: conv.thread_id.clone(),
        text: "crie o comando qa-formal".into(),
        tray: conv.thread_id.clone(),
        accepts_images: false,
        options: Default::default(),
    })
    .await
    .expect("send");
    let mut asked = false;
    let mut answer = String::new();
    loop {
        let e = tokio::time::timeout(Duration::from_secs(120), rx.recv())
            .await
            .expect("turn finished in time")
            .expect("open");
        match e {
            HostEvent::Conversation(ConversationEvent::UserInputRequested {
                request_id,
                prompt,
                source,
                ..
            }) => {
                eprintln!("user input requested ({source}): {prompt}");
                assert!(
                    host.quick_commands()
                        .unwrap()
                        .iter()
                        .all(|q| q.name != "qa-formal"),
                    "nothing saved before the user answers"
                );
                asked = true;
                let decision = approve(&prompt);
                host.respond(&request_id, decision).await.expect("respond");
            }
            HostEvent::Conversation(ConversationEvent::ApprovalRequested {
                request_id,
                kind,
                ..
            }) => {
                eprintln!("approval requested: {kind:?}");
                asked = true;
                host.respond(&request_id, aura_codex::approvals::Decision::Accept)
                    .await
                    .expect("respond");
            }
            HostEvent::Conversation(ConversationEvent::MessageCompleted { text, .. }) => {
                answer = text
            }
            HostEvent::Conversation(ConversationEvent::TurnCompleted { error, .. }) => {
                assert!(error.is_none(), "turn failed: {error:?}");
                break;
            }
            _ => {}
        }
    }
    eprintln!("answer: {answer}");
    assert!(asked, "Codex must ask before quick_command_save");
    let formal = host
        .quick_commands()
        .unwrap()
        .into_iter()
        .find(|q| q.name == "qa-formal")
        .expect("saved after approval");
    assert_eq!(formal.template, "Reescreva formal: {texto}");
    host.shutdown().await;
}

/// Accepts a Codex MCP tool-approval prompt (elicitation or question form).
fn approve(prompt: &Value) -> aura_codex::approvals::Decision {
    if let Some(questions) = prompt["questions"].as_array() {
        let mut answers = serde_json::Map::new();
        for q in questions {
            let option = q["options"]
                .as_array()
                .and_then(|o| o.first())
                .and_then(|o| o["label"].as_str())
                .unwrap_or("Allow")
                .to_string();
            answers.insert(
                q["id"].as_str().unwrap_or_default().to_string(),
                json!({"answers": [option]}),
            );
        }
        return aura_codex::approvals::Decision::Answer {
            content: json!({"answers": answers}),
        };
    }
    aura_codex::approvals::Decision::Answer { content: json!({}) }
}

/// 018 AC-004: the pinned app-server accepts Task mode with YOLO
/// (`danger-full-access` + `never`) on `thread/start` and `turn/start`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs AURA_CODEX_BIN (real app-server)"]
async fn real_app_server_runs_task_mode_with_yolo() {
    let bin = std::env::var_os("AURA_CODEX_BIN")
        .expect("set AURA_CODEX_BIN to the pinned app-server before running ignored E2E tests");
    let (base_url, _seen) = mock_provider().await;
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
    host.set_yolo(true, "ACEITO").unwrap();
    let provider = host
        .save_provider(
            serde_json::from_value(
                json!({"name": "Mock", "preset": "custom", "baseUrl": base_url}),
            )
            .unwrap(),
            Some("sk-test-e2e".into()),
        )
        .unwrap();
    let mut rx = host.subscribe();
    let conv = host
        .start_conversation(StartOptions {
            provider: format!("aura-{}", provider.id),
            model: Some("mock-model".into()),
            mode: aura_codex::modes::ConversationMode::Task {
                granted: vec![],
                network: false,
            },
            ..Default::default()
        })
        .await
        .expect("thread/start with danger-full-access + never");
    let (answer, _) = turn(&host, &mut rx, &conv.thread_id, "ping").await;
    assert_eq!(answer.trim(), "pong");
    host.shutdown().await;
}
