//! BYOK providers that already speak the Responses API (OpenAI, Azure,
//! OpenRouter, Ollama, LM Studio, vLLM): URL rewrite + credential injection.

use super::{
    BoxFut, StreamResponse, Upstream, authed_request, error_from_response, transport_error,
};
use crate::errors::UpstreamError;
use crate::registry::ProviderTarget;
use bytes::Bytes;
use futures::StreamExt;
use serde_json::Value;

pub struct Passthrough {
    pub target: ProviderTarget,
    pub http: reqwest::Client,
}

impl Upstream for Passthrough {
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>> {
        Box::pin(async move {
            let resp = authed_request(&self.http, &self.target, reqwest::Method::POST, "responses")
                .header("content-type", "application/json")
                .body(body)
                .send()
                .await
                .map_err(transport_error)?;
            if !resp.status().is_success() {
                return Err(error_from_response(resp).await);
            }
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("text/event-stream")
                .to_string();
            Ok(StreamResponse {
                content_type,
                body: Box::pin(
                    resp.bytes_stream()
                        .map(|r| r.map_err(std::io::Error::other)),
                ),
            })
        })
    }

    fn models(&self) -> BoxFut<'_, Result<Value, UpstreamError>> {
        Box::pin(async move {
            let resp = authed_request(&self.http, &self.target, reqwest::Method::GET, "models")
                .send()
                .await
                .map_err(transport_error)?;
            if !resp.status().is_success() {
                return Err(error_from_response(resp).await);
            }
            resp.json()
                .await
                .map_err(|e| UpstreamError::unreachable(e.to_string()))
        })
    }
}
