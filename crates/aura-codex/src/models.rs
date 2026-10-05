//! Model catalog parsing (002 TK-004).
//!
//! * ChatGPT plan: `GET /v1/models` through the gateway returns
//!   `{"models": [{"slug", "display_name", "visibility"}]}`; only
//!   `visibility == "list"` is shown, in server order.
//! * Codex `model/list` provides efforts/modalities for known models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub efforts: Vec<String>,
    pub default_effort: Option<String>,
    pub input_modalities: Vec<String>,
    pub is_default: bool,
}

impl ModelInfo {
    pub fn accepts_images(&self) -> bool {
        self.input_modalities.is_empty() || self.input_modalities.iter().any(|m| m == "image")
    }
}

pub fn parse_codex_model_list(v: &Value) -> Vec<ModelInfo> {
    v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| !m["hidden"].as_bool().unwrap_or(false))
        .map(|m| ModelInfo {
            id: m["model"]
                .as_str()
                .or(m["id"].as_str())
                .unwrap_or_default()
                .to_string(),
            display_name: m["displayName"]
                .as_str()
                .or(m["id"].as_str())
                .unwrap_or_default()
                .to_string(),
            efforts: m["supportedReasoningEfforts"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| e["reasoningEffort"].as_str().map(str::to_string))
                .collect(),
            default_effort: m["defaultReasoningEffort"].as_str().map(str::to_string),
            input_modalities: m["inputModalities"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_else(|| vec!["text".into(), "image".into()]),
            is_default: m["isDefault"].as_bool().unwrap_or(false),
        })
        .collect()
}

/// Parses the ChatGPT plan `/v1/models` body and enriches entries with the
/// Codex catalog when the same id is known there.
pub fn parse_plan_models(v: &Value, codex_catalog: &[ModelInfo]) -> Vec<ModelInfo> {
    let list = v["models"].as_array().or(v["data"].as_array());
    let mut out: Vec<ModelInfo> = list
        .into_iter()
        .flatten()
        .filter(|m| m["visibility"].as_str().is_none_or(|vis| vis == "list"))
        .filter_map(|m| {
            let id = m["slug"].as_str().or(m["id"].as_str())?.to_string();
            let display = m["display_name"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| id.clone());
            let known = codex_catalog.iter().find(|k| k.id == id);
            // Newer than the pinned Codex catalog: Aura's own catalog (013).
            if known.is_none()
                && let Some(k) = aura_core::model_catalog::known_model(&id)
            {
                return Some(from_catalog(k, Some(display)));
            }
            Some(ModelInfo {
                id,
                display_name: display,
                efforts: known
                    .map(|k| k.efforts.clone())
                    .unwrap_or_else(|| vec!["low".into(), "medium".into(), "high".into()]),
                default_effort: known
                    .and_then(|k| k.default_effort.clone())
                    .or(Some("medium".into())),
                input_modalities: known
                    .map(|k| k.input_modalities.clone())
                    .unwrap_or_else(|| vec!["text".into(), "image".into()]),
                is_default: false,
            })
        })
        .collect();
    if let Some(first) = out.first_mut() {
        first.is_default = true;
    }
    // Plan models Aura knows were released after the server list was built
    // (or that the list does not show): offered too; the server still decides
    // access when the turn starts.
    if !out.is_empty() {
        for k in aura_core::model_catalog::KNOWN.iter().filter(|k| k.in_plan) {
            if !out.iter().any(|m| m.id == k.id) {
                out.push(from_catalog(k, None));
            }
        }
    }
    out
}

fn from_catalog(k: &aura_core::model_catalog::KnownModel, display: Option<String>) -> ModelInfo {
    ModelInfo {
        id: k.id.to_string(),
        display_name: display.unwrap_or_else(|| k.display_name.to_string()),
        efforts: k.efforts.iter().map(|e| e.to_string()).collect(),
        default_effort: Some(k.default_effort.to_string()),
        input_modalities: if k.images {
            vec!["text".into(), "image".into()]
        } else {
            vec!["text".into()]
        },
        is_default: false,
    }
}

/// Keeps a saved effort only if the model supports it (AC-011).
pub fn effective_effort(model: &ModelInfo, saved: Option<&str>) -> Option<String> {
    match saved {
        Some(e) if model.efforts.iter().any(|x| x == e) => Some(e.to_string()),
        _ => model
            .default_effort
            .clone()
            .or_else(|| model.efforts.first().cloned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plan_models_keep_only_listed_in_server_order() {
        let v = json!({"models": [
            {"slug": "m-a", "display_name": "Modelo A", "visibility": "list"},
            {"slug": "m-x", "visibility": "hidden"},
            {"slug": "m-b", "display_name": "Modelo B", "visibility": "list"}]});
        let models = parse_plan_models(&v, &[]);
        // Server order first; Aura's known plan models (013) come after.
        assert_eq!(
            models
                .iter()
                .map(|m| m.display_name.as_str())
                .collect::<Vec<_>>(),
            ["Modelo A", "Modelo B", "GPT-6.1 Sol"]
        );
        assert!(models[0].is_default);
    }

    #[test]
    fn plan_list_gets_gpt_6_1_sol_with_its_efforts() {
        // 013 AC-001: the server list omits it → appended; listed but unknown
        // to the pinned Codex catalog → efforts from Aura's catalog.
        let without = json!({"models": [{"slug": "gpt-6-astra", "display_name": "GPT-6 Astra", "visibility": "list"}]});
        let models = parse_plan_models(&without, &[]);
        let sol = models
            .iter()
            .find(|m| m.id == "gpt-6.1-sol")
            .expect("appended");
        assert_eq!(sol.display_name, "GPT-6.1 Sol");
        assert_eq!(sol.efforts, ["low", "medium", "high", "xhigh", "max"]);
        assert_eq!(sol.default_effort.as_deref(), Some("medium"));
        assert!(sol.accepts_images() && !sol.is_default);
        assert_eq!(models[0].id, "gpt-6-astra");

        let with = json!({"models": [{"slug": "gpt-6.1-sol", "display_name": "GPT-6.1 Sol", "visibility": "list"}]});
        let models = parse_plan_models(&with, &[]);
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].efforts.last().map(String::as_str), Some("max"));

        // Signed out / empty list: nothing invented.
        assert!(parse_plan_models(&json!({"models": []}), &[]).is_empty());
    }

    #[test]
    fn unsupported_saved_effort_falls_back_to_model_default() {
        let m = ModelInfo {
            id: "m-b".into(),
            display_name: "B".into(),
            efforts: vec!["medium".into()],
            default_effort: Some("medium".into()),
            input_modalities: vec!["text".into()],
            is_default: false,
        };
        assert_eq!(
            effective_effort(&m, Some("high")).as_deref(),
            Some("medium")
        );
        assert_eq!(
            effective_effort(&m, Some("medium")).as_deref(),
            Some("medium")
        );
        assert!(!m.accepts_images());
    }

    #[test]
    fn codex_list_parsing() {
        let v = json!({"data": [{"id": "g", "model": "g", "displayName": "G", "hidden": false, "isDefault": true,
            "defaultReasoningEffort": "medium", "inputModalities": ["text", "image"],
            "supportedReasoningEfforts": [{"reasoningEffort": "low"}, {"reasoningEffort": "medium"}]},
            {"id": "h", "hidden": true}]});
        let m = parse_codex_model_list(&v);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].efforts, ["low", "medium"]);
        assert!(m[0].accepts_images());
    }
}
