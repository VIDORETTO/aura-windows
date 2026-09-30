//! Upstream adapters behind `/p/<provider>/v1/*`.

pub mod anthropic;
pub mod chat;
pub mod chatgpt_plan;
pub mod passthrough;

use crate::errors::UpstreamError;
use aura_core::Secret;
use bytes::Bytes;
use futures::Stream;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

/// A successful streaming response to hand back to Codex.
pub struct StreamResponse {
    pub content_type: String,
    pub body: ByteStream,
}

pub trait Upstream: Send + Sync {
    /// `POST /responses` with the raw Codex request body.
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>>;
    /// `GET /models`, returned verbatim (OpenAI-compatible shape).
    fn models(&self) -> BoxFut<'_, Result<Value, UpstreamError>>;
}

/// Supplies the bearer token of the ChatGPT plan account (implemented by the
/// host on top of `aura-auth`).
pub trait TokenProvider: Send + Sync {
    fn token(&self) -> BoxFut<'_, Result<Secret<String>, UpstreamError>>;
    /// Called once after an upstream 401.
    fn refresh(&self) -> BoxFut<'_, Result<Secret<String>, UpstreamError>>;
}

/// Converts a translator driven by upstream SSE into a Responses byte stream.
pub(crate) fn translated_stream<T, F, G>(
    upstream: reqwest::Response,
    mut translator: T,
    initial: Vec<String>,
    mut push: F,
    mut finish: G,
) -> ByteStream
where
    T: Send + 'static,
    F: FnMut(&mut T, &str) -> Vec<String> + Send + 'static,
    G: FnMut(&mut T) -> Vec<String> + Send + 'static,
{
    use futures::StreamExt;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(64);
    tokio::spawn(async move {
        let mut parser = crate::sse::SseParser::new();
        let mut stream = upstream.bytes_stream();
        let send = |frames: Vec<String>| {
            let tx = tx.clone();
            async move {
                if frames.is_empty() {
                    return true;
                }
                tx.send(Ok(Bytes::from(frames.concat()))).await.is_ok()
            }
        };
        let mut ok = send(initial).await;
        while ok {
            let Some(chunk) = stream.next().await else {
                break;
            };
            match chunk {
                Ok(bytes) => {
                    let mut frames = Vec::new();
                    for ev in parser.push(&bytes) {
                        frames.extend(push(&mut translator, &ev.data));
                    }
                    if !send(frames).await {
                        ok = false;
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("upstream stream error: {e}");
                    break;
                }
            }
        }
        if ok {
            let mut frames = Vec::new();
            if let Some(ev) = parser.finish() {
                frames.extend(push(&mut translator, &ev.data));
            }
            frames.extend(finish(&mut translator));
            let _ = send(frames).await;
        }
    });
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    }))
}

/// Maps a transport error from `reqwest`.
pub(crate) fn transport_error(e: reqwest::Error) -> UpstreamError {
    if e.is_timeout() {
        UpstreamError::new(504, "upstream_timeout", "provider did not answer in time")
    } else if e.is_connect() {
        UpstreamError::unreachable(format!("could not connect to provider: {e}"))
    } else {
        UpstreamError::unreachable(e.to_string())
    }
}

/// Reads a non-success response into an [`UpstreamError`].
pub(crate) async fn error_from_response(resp: reqwest::Response) -> UpstreamError {
    let status = resp.status().as_u16();
    let retry = crate::errors::retry_after(resp.headers());
    let body = resp.text().await.unwrap_or_default();
    UpstreamError::from_http(status, &body, retry)
}

/// Builds a request to a BYOK provider with its headers, query and credential.
pub(crate) fn authed_request(
    http: &reqwest::Client,
    target: &crate::registry::ProviderTarget,
    method: reqwest::Method,
    path: &str,
) -> reqwest::RequestBuilder {
    use crate::registry::AuthStyle;
    let url = format!("{}/{}", target.base_url.trim_end_matches('/'), path);
    let mut req = http.request(method, url);
    for (k, v) in &target.query {
        req = req.query(&[(k, v)]);
    }
    for (k, v) in &target.headers {
        req = req.header(k, v);
    }
    if let Some(key) = &target.credential {
        req = match target.auth {
            AuthStyle::Bearer => req.bearer_auth(key.expose()),
            AuthStyle::ApiKeyHeader => req.header("api-key", key.expose()),
            AuthStyle::XApiKey => req.header("x-api-key", key.expose()),
            AuthStyle::None => req,
        };
    }
    req
}
