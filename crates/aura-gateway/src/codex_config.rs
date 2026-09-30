//! Registers BYOK providers in the generated Codex `config.toml`.

use crate::registry::Provider;
use aura_codex::home::{ConfigContributor, GATEWAY_TOKEN_ENV, gateway_base_url, section};
use toml::{Table, Value};

pub fn codex_provider_id(provider_id: &str) -> String {
    format!("aura-{provider_id}")
}

pub struct GatewayConfigContributor {
    pub providers: Vec<Provider>,
    pub port: u16,
}

impl ConfigContributor for GatewayConfigContributor {
    fn contribute(&self, config: &mut Table) {
        let providers = section(config, "model_providers");
        for p in &self.providers {
            let mut t = Table::new();
            t.insert("name".into(), Value::String(p.name.clone()));
            t.insert(
                "base_url".into(),
                Value::String(gateway_base_url(self.port, &p.id)),
            );
            t.insert("env_key".into(), Value::String(GATEWAY_TOKEN_ENV.into()));
            t.insert("wire_api".into(), Value::String("responses".into()));
            t.insert("requires_openai_auth".into(), Value::Boolean(false));
            t.insert("supports_websockets".into(), Value::Boolean(false));
            t.insert("request_max_retries".into(), Value::Integer(2));
            t.insert("stream_idle_timeout_ms".into(), Value::Integer(120_000));
            providers.insert(codex_provider_id(&p.id), Value::Table(t));
        }
    }
}

/// Per-thread overrides so Codex knows limits of models it has no metadata for.
pub fn thread_overrides(p: &Provider, model: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut m = serde_json::Map::new();
    if let Some(spec) = p.models.iter().find(|s| s.id == model) {
        if let Some(cw) = spec.context_window {
            m.insert("model_context_window".into(), serde_json::json!(cw));
        }
        if let Some(mo) = spec.max_output {
            m.insert("model_max_output_tokens".into(), serde_json::json!(mo));
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{AuthStyle, ProviderStatus, Quirks, Wire};

    #[test]
    fn providers_point_at_the_gateway_without_secrets() {
        let p = Provider {
            id: "groq-1a2b3c".into(),
            name: "Groq".into(),
            preset: "groq".into(),
            wire: Wire::Chat,
            base_url: "https://api.groq.com/openai/v1".into(),
            auth: AuthStyle::Bearer,
            extra_headers: Default::default(),
            models: vec![],
            credential_hint: Some("••••abcd".into()),
            status: ProviderStatus::Verified,
            last_error: None,
            quirks: Quirks::default(),
        };
        let doc = aura_codex::home::render(
            &aura_codex::home::BaseConfig {
                gateway_port: 4100,
                ..Default::default()
            },
            &[&GatewayConfigContributor {
                providers: vec![p],
                port: 4100,
            }],
        );
        let text = toml::to_string(&doc).unwrap();
        assert!(text.contains("[model_providers.aura-groq-1a2b3c]"));
        assert!(text.contains("base_url = \"http://127.0.0.1:4100/p/groq-1a2b3c/v1\""));
        assert!(!text.contains("api.groq.com"));
        assert!(!text.contains("abcd"));
    }
}
