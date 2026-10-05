//! Connection tests and model discovery (003 TK-001, TK-007).

use crate::registry::{ModelSpec, Provider, ProviderTarget, Wire};
use crate::upstream::authed_request;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionCategory {
    Ok,
    Unauthorized,
    NotFound,
    Timeout,
    Tls,
    Network,
    RateLimited,
    Unexpected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResult {
    pub category: ConnectionCategory,
    pub detail: String,
    pub models: Vec<ModelSpec>,
}

fn categorize_error(e: &reqwest::Error) -> ConnectionCategory {
    let text = format!("{e:?}").to_lowercase();
    if e.is_timeout() {
        ConnectionCategory::Timeout
    } else if text.contains("certificate") || text.contains("tls") || text.contains("ssl") {
        ConnectionCategory::Tls
    } else {
        ConnectionCategory::Network
    }
}

/// `GET <base>/models` with a 10 s timeout; parses models on success.
pub async fn test_connection(provider: &Provider, target: &ProviderTarget) -> ConnectionResult {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client");
    let mut req = authed_request(&http, target, reqwest::Method::GET, "models");
    if provider.wire == Wire::Anthropic {
        req = req.header(
            "anthropic-version",
            crate::translate::anthropic::ANTHROPIC_VERSION,
        );
    }
    match req.send().await {
        Err(e) => ConnectionResult {
            category: categorize_error(&e),
            detail: e.to_string(),
            models: vec![],
        },
        Ok(resp) => {
            let status = resp.status().as_u16();
            let category = match status {
                200..=299 => ConnectionCategory::Ok,
                401 | 403 => ConnectionCategory::Unauthorized,
                404 => ConnectionCategory::NotFound,
                429 => ConnectionCategory::RateLimited,
                _ => ConnectionCategory::Unexpected,
            };
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            let models = if category == ConnectionCategory::Ok {
                parse_models(&body)
            } else {
                vec![]
            };
            ConnectionResult {
                category,
                detail: format!("HTTP {status}"),
                models,
            }
        }
    }
}

/// Parses OpenAI-compatible (`data[].id`), OpenRouter (`architecture`,
/// `supported_parameters`, `context_length`) and Anthropic (`data[].id`,
/// `display_name`) model lists.
pub fn parse_models(body: &Value) -> Vec<ModelSpec> {
    body["data"]
        .as_array()
        .or(body["models"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|m| {
            let id = m["id"].as_str().or(m["slug"].as_str())?.to_string();
            let modalities: Option<Vec<String>> =
                m["architecture"]["input_modalities"].as_array().map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                });
            let params: Option<Vec<String>> = m["supported_parameters"].as_array().map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            });
            let (images_est, tools_est, reasoning_est) = estimate_capabilities(&id);
            let estimated = modalities.is_none() || params.is_none();
            if let Some(k) = aura_core::model_catalog::known_model(&id) {
                // Known model (013): exact efforts and limits.
                return Some(ModelSpec {
                    display_name: m["display_name"]
                        .as_str()
                        .or(m["name"].as_str())
                        .map(str::to_string)
                        .or_else(|| Some(k.display_name.to_string())),
                    context_window: Some(k.context_window),
                    max_output: Some(k.max_output),
                    supports_images: k.images,
                    supports_tools: k.tools,
                    supports_reasoning: true,
                    estimated: false,
                    manual: false,
                    efforts: k.efforts.iter().map(|e| e.to_string()).collect(),
                    default_effort: Some(k.default_effort.to_string()),
                    id,
                });
            }
            Some(ModelSpec {
                display_name: m["display_name"]
                    .as_str()
                    .or(m["name"].as_str())
                    .map(str::to_string),
                context_window: m["context_length"]
                    .as_u64()
                    .or(m["context_window"].as_u64())
                    .map(|v| v as u32),
                max_output: m["top_provider"]["max_completion_tokens"]
                    .as_u64()
                    .map(|v| v as u32),
                supports_images: modalities
                    .as_ref()
                    .map(|v| v.iter().any(|x| x == "image"))
                    .unwrap_or(images_est),
                supports_tools: params
                    .as_ref()
                    .map(|v| v.iter().any(|x| x == "tools"))
                    .unwrap_or(tools_est),
                supports_reasoning: params
                    .as_ref()
                    .map(|v| {
                        v.iter()
                            .any(|x| x == "reasoning" || x == "include_reasoning")
                    })
                    .unwrap_or(reasoning_est),
                estimated,
                manual: false,
                efforts: Vec::new(),
                default_effort: None,
                id,
            })
        })
        .collect()
}

/// Name-based guesses (marked `estimated` and editable in the UI).
pub fn estimate_capabilities(id: &str) -> (bool, bool, bool) {
    let l = id.to_lowercase();
    let images = [
        "vision",
        "-vl",
        "vl-",
        "4o",
        "gpt-4.1",
        "gpt-5",
        "gemini",
        "claude",
        "llava",
        "pixtral",
        "grok-4",
        "llama-4",
        "qwen2.5-vl",
        "gemma-3",
    ]
    .iter()
    .any(|k| l.contains(k));
    let tools = !["embed", "whisper", "tts", "moderation", "dall-e", "image"]
        .iter()
        .any(|k| l.contains(k));
    let reasoning = [
        "o1",
        "o3",
        "o4",
        "reasoner",
        "r1",
        "thinking",
        "gpt-5",
        "qwq",
        "claude-opus",
        "claude-sonnet",
        "gemini-2.5",
        "gemini-3",
    ]
    .iter()
    .any(|k| l.contains(k));
    (images, tools, reasoning)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn openrouter_metadata_wins_over_estimates() {
        let body = json!({"data": [
            {"id": "acme/text-only", "architecture": {"input_modalities": ["text"]}, "supported_parameters": ["tools"], "context_length": 32000},
            {"id": "acme/vision-x", "architecture": {"input_modalities": ["text", "image"]}, "supported_parameters": []}]});
        let m = parse_models(&body);
        assert!(!m[0].supports_images && m[0].supports_tools && !m[0].estimated);
        assert_eq!(m[0].context_window, Some(32000));
        assert!(m[1].supports_images && !m[1].supports_tools);
    }

    #[test]
    fn known_models_get_their_efforts_and_limits() {
        // 013 AC-001.
        let m = parse_models(
            &json!({"data": [{"id": "gpt-6.1-sol"}, {"id": "openai/gpt-6.1-sol"}, {"id": "llama3"}]}),
        );
        for sol in &m[..2] {
            assert_eq!(sol.display_name.as_deref(), Some("GPT-6.1 Sol"));
            assert_eq!(
                (sol.context_window, sol.max_output),
                (Some(1_050_000), Some(128_000))
            );
            assert!(
                sol.supports_images
                    && sol.supports_tools
                    && sol.supports_reasoning
                    && !sol.estimated
            );
            assert_eq!(sol.efforts, ["low", "medium", "high", "xhigh", "max"]);
            assert_eq!(sol.default_effort.as_deref(), Some("medium"));
        }
        assert!(m[2].efforts.is_empty() && m[2].default_effort.is_none());
    }

    #[test]
    fn plain_lists_are_estimated() {
        let m = parse_models(
            &json!({"data": [{"id": "llama-3.2-90b-vision"}, {"id": "text-embedding-3"}]}),
        );
        assert!(m[0].supports_images && m[0].estimated);
        assert!(!m[1].supports_tools);
    }
}
