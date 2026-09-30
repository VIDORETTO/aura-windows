//! Aura's MCP server (ADR 0006, `specs/004-contexto-de-tela` TK-004).
//!
//! Implements the subset of the MCP *streamable HTTP* transport a client such
//! as Codex needs: `POST /mcp` with JSON-RPC (`initialize`,
//! `notifications/initialized`, `ping`, `tools/list`, `tools/call`), JSON
//! responses (no server-initiated stream), `Mcp-Session-Id`, bearer-token auth
//! and `Origin` rejection. Tool behaviour lives in the host behind
//! [`ToolHandler`], which applies the privacy policy (OT-004).

pub mod tools;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub const PROTOCOL_VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26"];
pub const CONVERSATION_HEADER: &str = "x-aura-conversation";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    pub annotations: Value,
}

/// One content block of a tool result.
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Text(String),
    Image { png_or_jpeg: Vec<u8>, mime: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutput {
    pub content: Vec<Content>,
    pub is_error: bool,
}

impl ToolOutput {
    pub fn text(t: impl Into<String>) -> Self {
        Self {
            content: vec![Content::Text(t.into())],
            is_error: false,
        }
    }
    /// Structured error (OT-004): `code` in {denied, paused, excluded,
    /// unavailable, timeout}.
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            content: vec![Content::Text(
                json!({"code": code, "message": message}).to_string(),
            )],
            is_error: true,
        }
    }
    fn to_json(&self) -> Value {
        let content: Vec<Value> = self
            .content
            .iter()
            .map(|c| match c {
                Content::Text(t) => json!({"type": "text", "text": t}),
                Content::Image { png_or_jpeg, mime } => json!({
                    "type": "image",
                    "data": base64::engine::general_purpose::STANDARD.encode(png_or_jpeg),
                    "mimeType": mime,
                }),
            })
            .collect();
        json!({"content": content, "isError": self.is_error})
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallContext {
    /// Aura conversation uuid from `X-Aura-Conversation` (empty if absent).
    pub conversation: String,
}

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait ToolHandler: Send + Sync {
    fn call<'a>(&'a self, tool: &'a str, args: Value, ctx: CallContext) -> BoxFut<'a, ToolOutput>;
}

#[derive(Clone)]
struct McpState {
    tools: Arc<Vec<ToolDef>>,
    handler: Arc<dyn ToolHandler>,
    token: Arc<String>,
    version: &'static str,
}

fn authorized(headers: &HeaderMap, token: &str) -> bool {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| {
            t.len() == token.len()
                && t.bytes()
                    .zip(token.bytes())
                    .fold(0u8, |a, (x, y)| a | (x ^ y))
                    == 0
        })
}

fn rpc_result(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn rpc_error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

async fn handle_one(st: &McpState, msg: &Value, ctx: &CallContext) -> Option<Value> {
    let method = msg["method"].as_str().unwrap_or_default();
    let id = msg.get("id").cloned();
    let id = id?; // notifications get no reply
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => {
            let requested = params["protocolVersion"]
                .as_str()
                .unwrap_or(PROTOCOL_VERSIONS[1]);
            let version = PROTOCOL_VERSIONS
                .iter()
                .find(|v| **v == requested)
                .copied()
                .unwrap_or(PROTOCOL_VERSIONS[1]);
            rpc_result(
                &id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {"listChanged": false}},
                    "serverInfo": {"name": "aura", "title": "Aura", "version": st.version},
                    "instructions": "Ferramentas do desktop do usuário. Respeite recusas e pausas de privacidade."
                }),
            )
        }
        "ping" => rpc_result(&id, json!({})),
        "tools/list" => rpc_result(&id, json!({"tools": *st.tools})),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default().to_string();
            if !st.tools.iter().any(|t| t.name == name) {
                return Some(rpc_error(&id, -32602, &format!("unknown tool {name}")));
            }
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let out = st.handler.call(&name, args, ctx.clone()).await;
            rpc_result(&id, out.to_json())
        }
        _ => rpc_error(&id, -32601, &format!("method not found: {method}")),
    })
}

async fn post_mcp(State(st): State<McpState>, headers: HeaderMap, body: Bytes) -> Response {
    if headers.contains_key("origin") {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    if !authorized(&headers, &st.token) {
        return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
    }
    let Ok(v) = serde_json::from_slice::<Value>(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(rpc_error(&Value::Null, -32700, "parse error")),
        )
            .into_response();
    };
    let ctx = CallContext {
        conversation: headers
            .get(CONVERSATION_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string(),
    };
    let is_initialize = v["method"] == "initialize";
    let reply = match &v {
        Value::Array(batch) => {
            let mut out = Vec::new();
            for m in batch {
                if let Some(r) = handle_one(&st, m, &ctx).await {
                    out.push(r);
                }
            }
            (!out.is_empty()).then_some(Value::Array(out))
        }
        m => handle_one(&st, m, &ctx).await,
    };
    match reply {
        None => StatusCode::ACCEPTED.into_response(),
        Some(r) => {
            let mut resp = axum::Json(r).into_response();
            if is_initialize && let Ok(v) = HeaderValue::from_str(&uuid::Uuid::new_v4().to_string())
            {
                resp.headers_mut().insert("mcp-session-id", v);
            }
            resp
        }
    }
}

/// The `/mcp` router to merge into the loopback server.
pub fn router(tools: Vec<ToolDef>, handler: Arc<dyn ToolHandler>, token: String) -> Router {
    let st = McpState {
        tools: Arc::new(tools),
        handler,
        token: Arc::new(token),
        version: env!("CARGO_PKG_VERSION"),
    };
    Router::new()
        .route(
            "/mcp",
            post(post_mcp)
                .get(|| async { StatusCode::METHOD_NOT_ALLOWED })
                .delete(|| async { StatusCode::OK }),
        )
        .with_state(st)
}
