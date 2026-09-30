//! BYOK Chat Completions providers (Groq, Gemini OpenAI-compatible,
//! DeepSeek, Mistral, xAI, Together, generic OpenAI-compatible).

use super::{
    BoxFut, StreamResponse, Upstream, authed_request, error_from_response, translated_stream,
    transport_error,
};
use crate::errors::UpstreamError;
use crate::registry::ProviderTarget;
use crate::translate::chat::{ChatQuirks, ChatStreamTranslator, to_chat_request};
use crate::translate::emitter::Emitter;
use crate::translate::request::normalize;
use bytes::Bytes;
use serde_json::Value;

pub struct ChatCompletionsAdapter {
    pub target: ProviderTarget,
    pub quirks: ChatQuirks,
    pub http: reqwest::Client,
}

impl Upstream for ChatCompletionsAdapter {
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>> {
        Box::pin(async move {
            let req: Value = serde_json::from_slice(&body)
                .map_err(|e| UpstreamError::new(400, "invalid_request_error", e.to_string()))?;
            let n = normalize(&req);
            if !n.dropped_tools.is_empty() {
                tracing::debug!(
                    "dropping hosted tools not available on this provider: {:?}",
                    n.dropped_tools
                );
            }
            let chat = to_chat_request(&n, &self.quirks);
            let resp = authed_request(
                &self.http,
                &self.target,
                reqwest::Method::POST,
                "chat/completions",
            )
            .json(&chat)
            .send()
            .await
            .map_err(transport_error)?;
            if !resp.status().is_success() {
                return Err(error_from_response(resp).await);
            }
            let mut translator = ChatStreamTranslator::new(
                Emitter::new(&n.model, n.custom_tools.clone())
                    .with_namespaces(n.namespaced.clone()),
            );
            let initial = translator.start();
            let stream = translated_stream(
                resp,
                translator,
                initial,
                |t: &mut ChatStreamTranslator, data| t.push(data),
                |t: &mut ChatStreamTranslator| t.finish(),
            );
            Ok(StreamResponse {
                content_type: "text/event-stream".into(),
                body: stream,
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
