//! Local Responses API gateway (ADR 0003, ADR 0007).
//!
//! Every model request from the Codex app-server goes through here:
//! * `chatgpt-plan` → passthrough to `api.openai.com/v1` with the Sign in with
//!   ChatGPT access token;
//! * BYOK Responses providers → passthrough with the stored credential;
//! * BYOK Chat Completions / Anthropic providers → translated.
//!
//! Credentials never reach the Codex process: it only knows a per-run
//! loopback token.

pub mod codex_config;
pub mod discovery;
pub mod errors;
pub mod registry;
pub mod server;
pub mod sse;
pub mod translate;
pub mod upstream;

use registry::{Provider, ProviderRegistry, RegistryError, Wire};
use std::sync::Arc;
use upstream::Upstream;

/// Builds the upstream for a BYOK provider.
pub fn build_upstream(
    registry: &ProviderRegistry<'_>,
    provider: &Provider,
    http: reqwest::Client,
) -> Result<Arc<dyn Upstream>, RegistryError> {
    let target = registry.target(provider)?;
    Ok(match provider.wire {
        Wire::Responses => Arc::new(upstream::passthrough::Passthrough { target, http }),
        Wire::Chat => Arc::new(upstream::chat::ChatCompletionsAdapter {
            target,
            http,
            quirks: translate::chat::ChatQuirks {
                no_parallel_tool_calls: provider.quirks.no_parallel_tool_calls,
                reasoning_effort: provider.quirks.reasoning_effort,
                no_stream_options: provider.quirks.no_stream_options,
            },
        }),
        Wire::Anthropic => Arc::new(upstream::anthropic::AnthropicAdapter {
            target,
            http,
            options: translate::anthropic::AnthropicOptions {
                max_tokens: 8192,
                thinking: true,
            },
        }),
    })
}
