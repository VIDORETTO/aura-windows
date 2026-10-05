//! MCP protocol behaviour with a scripted tool handler.

use aura_mcp::{BoxFut, CallContext, Content, ToolHandler, ToolOutput, router, tools};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

struct Handler {
    calls: Mutex<Vec<(String, Value, String)>>,
}

impl ToolHandler for Handler {
    fn call<'a>(&'a self, tool: &'a str, args: Value, ctx: CallContext) -> BoxFut<'a, ToolOutput> {
        self.calls
            .lock()
            .unwrap()
            .push((tool.to_string(), args.clone(), ctx.conversation.clone()));
        Box::pin(async move {
            match tool {
                "screen_capture" if ctx.conversation == "blocked" => {
                    ToolOutput::error("denied", "Permissão negada")
                }
                "screen_capture" => ToolOutput {
                    content: vec![
                        Content::Image {
                            png_or_jpeg: vec![0x89, b'P', b'N', b'G'],
                            mime: "image/png".into(),
                        },
                        Content::Text("notepad.exe — Sem título".into()),
                    ],
                    is_error: false,
                },
                _ => ToolOutput::text("{\"process\":\"notepad.exe\"}"),
            }
        })
    }
}

async fn start() -> (String, Arc<Handler>) {
    let handler = Arc::new(Handler {
        calls: Mutex::new(vec![]),
    });
    let app = router(tools::all(), handler.clone(), "tok".into());
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mcp", l.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    (url, handler)
}

async fn rpc(url: &str, body: Value, conversation: Option<&str>) -> reqwest::Response {
    let mut req = reqwest::Client::new()
        .post(url)
        .bearer_auth("tok")
        .json(&body);
    if let Some(c) = conversation {
        req = req.header("X-Aura-Conversation", c);
    }
    req.send().await.unwrap()
}

#[tokio::test]
async fn handshake_list_and_call() {
    let (url, handler) = start().await;
    let r = rpc(&url, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "codex", "version": "x"}}}), None).await;
    assert!(r.headers().contains_key("mcp-session-id"));
    let v: Value = r.json().await.unwrap();
    assert_eq!(v["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(v["result"]["serverInfo"]["name"], "aura");

    let r = rpc(
        &url,
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        None,
    )
    .await;
    assert_eq!(r.status(), 202);

    let v: Value = rpc(
        &url,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        None,
    )
    .await
    .json()
    .await
    .unwrap();
    let names: Vec<&str> = v["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "screen_capture",
            "active_window_info",
            "screen_text",
            "screen_recent",
            "audio_recent",
            "attachment_read",
            "extensions_list",
            "skill_save",
            "quick_command_save",
            "mcp_server_save",
            "settings_describe",
            "settings_propose",
            "settings_apply",
            "settings_undo",
            "clock_now",
            "reminder_create",
            "reminder_list",
            "reminder_delete",
            "note_save",
            "note_search",
            "meeting_search",
            "meeting_get",
            "meeting_brief_save",
            "recipe_list",
            "recipe_save",
            "open_windows",
            "exclusion_list",
            "exclusion_add",
            "profile_list",
            "profile_save",
            "action_save",
            "action_list",
            "action_done",
            "project_list",
            "project_save",
            "meeting_set_project",
            "meeting_stats",
        ]
    );
    assert_eq!(v["result"]["tools"][0]["annotations"]["readOnlyHint"], true);

    let v: Value = rpc(&url, json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "screen_capture", "arguments": {"target": "monitor", "reason": "ver erro"}}}), Some("conv-1")).await.json().await.unwrap();
    assert_eq!(v["result"]["isError"], false);
    assert_eq!(v["result"]["content"][0]["type"], "image");
    assert_eq!(v["result"]["content"][0]["mimeType"], "image/png");
    assert_eq!(v["result"]["content"][0]["data"], "iVBORw==");
    let (tool, args, conv) = handler.calls.lock().unwrap()[0].clone();
    assert_eq!((tool.as_str(), conv.as_str()), ("screen_capture", "conv-1"));
    assert_eq!(args["reason"], "ver erro");
}

#[tokio::test]
async fn denied_tool_returns_structured_error() {
    let (url, _) = start().await;
    let v: Value = rpc(
        &url,
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "screen_capture", "arguments": {"reason": "x"}}}),
        Some("blocked"),
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(v["result"]["isError"], true);
    let payload: Value =
        serde_json::from_str(v["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(payload["code"], "denied");
}

#[tokio::test]
async fn auth_origin_unknown_tool_and_method() {
    let (url, _) = start().await;
    let r = reqwest::Client::new()
        .post(&url)
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 401);
    let r = reqwest::Client::new()
        .post(&url)
        .bearer_auth("tok")
        .header("origin", "http://evil")
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
    let v: Value = rpc(
        &url,
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "rm_rf"}}),
        None,
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(v["error"]["code"], -32602);
    let v: Value = rpc(
        &url,
        json!({"jsonrpc": "2.0", "id": 2, "method": "resources/list"}),
        None,
    )
    .await
    .json()
    .await
    .unwrap();
    assert_eq!(v["error"]["code"], -32601);
    let v: Value = rpc(&url, json!([{"jsonrpc": "2.0", "id": 5, "method": "ping"}, {"jsonrpc": "2.0", "method": "notifications/initialized"}]), None).await.json().await.unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
}
