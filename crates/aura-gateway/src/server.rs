//! Loopback HTTP server (OT-002 of 003): `127.0.0.1`, ephemeral port, random
//! bearer token per run, requests with an `Origin` header rejected (browsers
//! cannot reach it). Also hosts the Aura MCP router under `/mcp`.

use crate::errors::UpstreamError;
use crate::tool_results::ToolResultResolver;
use crate::upstream::Upstream;
use aura_core::Secret;
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Routing table from provider id (`chatgpt-plan`, `<byok id>`) to upstream.
#[derive(Default)]
pub struct Routes {
    map: RwLock<HashMap<String, Arc<dyn Upstream>>>,
    tool_results: RwLock<Option<Arc<dyn ToolResultResolver>>>,
}

impl Routes {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub fn set(&self, id: &str, upstream: Arc<dyn Upstream>) {
        self.map.write().unwrap().insert(id.to_string(), upstream);
    }
    pub fn remove(&self, id: &str) {
        self.map.write().unwrap().remove(id);
    }
    pub fn get(&self, id: &str) -> Option<Arc<dyn Upstream>> {
        self.map.read().unwrap().get(id).cloned()
    }
    pub fn ids(&self) -> Vec<String> {
        self.map.read().unwrap().keys().cloned().collect()
    }
    pub fn set_tool_results(&self, resolver: Arc<dyn ToolResultResolver>) {
        *self.tool_results.write().unwrap() = Some(resolver);
    }
}

pub struct GatewayHandle {
    pub port: u16,
    pub token: Secret<String>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl GatewayHandle {
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for GatewayHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

#[derive(Clone)]
struct AppState {
    routes: Arc<Routes>,
    token: Arc<String>,
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn auth(State(st): State<AppState>, req: Request, next: Next) -> Response {
    if req.headers().contains_key("origin") {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    let ok = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| constant_time_eq(t.as_bytes(), st.token.as_bytes()));
    if !ok {
        return (StatusCode::UNAUTHORIZED, "invalid gateway token").into_response();
    }
    next.run(req).await
}

async fn responses(
    State(st): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(upstream) = st.routes.get(&id) else {
        return UpstreamError::new(404, "provider_not_found", format!("unknown provider {id}"))
            .into_response();
    };
    let resolver = st.routes.tool_results.read().unwrap().clone();
    let mut scopes = headers.get_all("thread-id").iter();
    let thread = scopes.next().and_then(|value| value.to_str().ok());
    let thread = if scopes.next().is_some() {
        None
    } else {
        thread
    };
    let scope = resolver
        .as_ref()
        .and_then(|resolver| thread.and_then(|thread| resolver.begin_response(thread)));
    let private = scope.is_some() || crate::tool_results::contains_private_marker(&body);
    let protect_stream = resolver.is_some();
    let body = if let Some(resolver) = &resolver {
        match crate::tool_results::prepare(body, thread, resolver.as_ref()) {
            Ok((body, _)) => body,
            Err(error) => return error.into_response(),
        }
    } else {
        body
    };
    // Diagnostics for protocol work only (never enabled in releases): raw
    // Codex requests, which may contain conversation content.
    if !private && let Some(dir) = std::env::var_os("AURA_GATEWAY_DUMP_DIR") {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(
            std::path::Path::new(&dir).join(format!("{id}-{n}.json")),
            &body,
        );
    }
    let started = std::time::Instant::now();
    match upstream.responses(body).await {
        Ok(mut stream) => {
            tracing::info!(provider = %id, elapsed_ms = started.elapsed().as_millis() as u64, "responses stream opened");
            if protect_stream {
                if !stream.content_type.starts_with("text/event-stream") {
                    return UpstreamError::new(
                        502,
                        "web_stream_invalid",
                        "Expected a streaming model response",
                    )
                    .into_response();
                }
                stream.body = crate::private_calls::protect(stream.body, scope);
            }
            let mut resp = Response::new(Body::from_stream(stream.body));
            if let Ok(v) = HeaderValue::from_str(&stream.content_type) {
                resp.headers_mut().insert("content-type", v);
            }
            resp.headers_mut()
                .insert("cache-control", HeaderValue::from_static("no-cache"));
            resp
        }
        Err(mut e) => {
            if private {
                e.message = "The model provider could not process the web result".into();
                if !matches!(
                    e.code.as_str(),
                    "context_length_exceeded"
                        | "invalid_api_key"
                        | "rate_limit_exceeded"
                        | "server_overloaded"
                        | "server_error"
                ) {
                    e.code = "web_provider_error".into();
                }
            }
            tracing::warn!(provider = %id, status = e.status, code = %e.code, "upstream error");
            e.into_response()
        }
    }
}

async fn models(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    let Some(upstream) = st.routes.get(&id) else {
        return UpstreamError::new(404, "provider_not_found", format!("unknown provider {id}"))
            .into_response();
    };
    match upstream.models().await {
        Ok(v) => axum::Json(v).into_response(),
        Err(e) => e.into_response(),
    }
}

fn random_token() -> String {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).expect("random");
    hex::encode(b)
}

/// Largest Responses request accepted from Codex.
pub const MAX_TURN_BYTES: usize = 256 * 1024 * 1024;

/// Starts the server. `extra` (the MCP router) is merged as-is; it performs
/// its own authentication.
pub async fn start(routes: Arc<Routes>, extra: Option<Router>) -> std::io::Result<GatewayHandle> {
    let token = random_token();
    let state = AppState {
        routes,
        token: Arc::new(token.clone()),
    };
    let provider_routes = Router::new()
        .route("/p/{id}/v1/responses", post(responses))
        .route("/p/{id}/v1/models", get(models))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        // Turns carry images as base64 (up to 10 per turn, 20 MB each).
        .layer(axum::extract::DefaultBodyLimit::max(MAX_TURN_BYTES))
        .with_state(state);
    let app = match extra {
        Some(extra) => provider_routes.merge(extra),
        None => provider_routes,
    };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await;
    });
    Ok(GatewayHandle {
        port,
        token: Secret::new(token),
        shutdown: Some(tx),
    })
}
