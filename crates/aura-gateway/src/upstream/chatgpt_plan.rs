//! ChatGPT plan passthrough (ADR 0007): forwards Codex requests to
//! `https://api.openai.com/v1` with the Sign in with ChatGPT access token,
//! refreshing once after a 401.

use super::{
    BoxFut, StreamResponse, TokenProvider, Upstream, error_from_response, transport_error,
};
use crate::errors::UpstreamError;
use bytes::Bytes;
use futures::StreamExt;
use serde_json::Value;
use std::sync::Arc;

pub struct ChatGptPlanUpstream {
    pub base_url: String,
    pub http: reqwest::Client,
    pub tokens: Arc<dyn TokenProvider>,
}

impl ChatGptPlanUpstream {
    pub fn new(http: reqwest::Client, tokens: Arc<dyn TokenProvider>) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".into(),
            http,
            tokens,
        }
    }

    async fn send(
        &self,
        path: &str,
        body: Option<Bytes>,
        token: &str,
    ) -> Result<reqwest::Response, UpstreamError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
        let req = match body {
            Some(b) => self
                .http
                .post(url)
                .header("content-type", "application/json")
                .body(b),
            None => self.http.get(url),
        };
        req.bearer_auth(token).send().await.map_err(transport_error)
    }

    async fn with_retry(
        &self,
        path: &str,
        body: Option<Bytes>,
    ) -> Result<reqwest::Response, UpstreamError> {
        let token = self.tokens.token().await?;
        let resp = self.send(path, body.clone(), token.expose()).await?;
        if resp.status().as_u16() != 401 {
            return Ok(resp);
        }
        let token = self.tokens.refresh().await?;
        self.send(path, body, token.expose()).await
    }
}

impl Upstream for ChatGptPlanUpstream {
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>> {
        Box::pin(async move {
            let resp = self.with_retry("responses", Some(body)).await?;
            if !resp.status().is_success() {
                return Err(error_from_response(resp).await);
            }
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("text/event-stream")
                .to_string();
            let body = resp
                .bytes_stream()
                .map(|r| r.map_err(std::io::Error::other));
            Ok(StreamResponse {
                content_type,
                body: Box::pin(body),
            })
        })
    }

    fn models(&self) -> BoxFut<'_, Result<Value, UpstreamError>> {
        Box::pin(async move {
            let resp = self.with_retry("models", None).await?;
            if !resp.status().is_success() {
                return Err(error_from_response(resp).await);
            }
            resp.json()
                .await
                .map_err(|e| UpstreamError::unreachable(e.to_string()))
        })
    }
}
