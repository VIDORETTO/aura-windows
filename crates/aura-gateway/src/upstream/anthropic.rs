//! BYOK Anthropic Messages provider.

use super::{
    BoxFut, StreamResponse, Upstream, error_from_response, translated_stream, transport_error,
};
use crate::errors::UpstreamError;
use crate::registry::ProviderTarget;
use crate::translate::anthropic::{
    ANTHROPIC_VERSION, AnthropicOptions, AnthropicStreamTranslator, to_messages_request,
};
use crate::translate::emitter::Emitter;
use crate::translate::request::normalize;
use bytes::Bytes;
use serde_json::Value;

pub struct AnthropicAdapter {
    pub target: ProviderTarget,
    pub options: AnthropicOptions,
    pub http: reqwest::Client,
}

impl AnthropicAdapter {
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.target.base_url.trim_end_matches('/'), path);
        let mut req = self
            .http
            .request(method, url)
            .header("anthropic-version", ANTHROPIC_VERSION);
        for (k, v) in &self.target.headers {
            req = req.header(k, v);
        }
        if let Some(key) = &self.target.credential {
            req = req.header("x-api-key", key.expose());
        }
        req
    }
}

impl Upstream for AnthropicAdapter {
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>> {
        Box::pin(async move {
            let req: Value = serde_json::from_slice(&body)
                .map_err(|e| UpstreamError::new(400, "invalid_request_error", e.to_string()))?;
            let n = normalize(&req);
            let payload = to_messages_request(&n, &self.options);
            let resp = self
                .request(reqwest::Method::POST, "messages")
                .json(&payload)
                .send()
                .await
                .map_err(transport_error)?;
            if !resp.status().is_success() {
                return Err(error_from_response(resp).await);
            }
            let mut translator = AnthropicStreamTranslator::new(
                Emitter::new(&n.model, n.custom_tools.clone())
                    .with_namespaces(n.namespaced.clone()),
            );
            let initial = translator.start();
            let stream = translated_stream(
                resp,
                translator,
                initial,
                |t: &mut AnthropicStreamTranslator, data| t.push(data),
                |t: &mut AnthropicStreamTranslator| t.finish(),
            );
            Ok(StreamResponse {
                content_type: "text/event-stream".into(),
                body: stream,
            })
        })
    }

    fn models(&self) -> BoxFut<'_, Result<Value, UpstreamError>> {
        Box::pin(async move {
            let resp = self
                .request(reqwest::Method::GET, "models")
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
