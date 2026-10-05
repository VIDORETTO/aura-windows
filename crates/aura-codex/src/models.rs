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

/// Parses the ChatGPT plan `/v1/models` body (ids the plan offers) and
/// keeps only Aura's plan catalog ([`plan_catalog`]).
pub fn parse_plan_models(v: &Value, codex_catalog: &[ModelInfo]) -> Vec<ModelInfo> {
    let list = v["models"].as_array().or(v["data"].as_array());
    let listed: Vec<ModelInfo> = list
        .into_iter()
        .flatten()
        .filter(|m| m["visibility"].as_str().is_none_or(|vis| vis == "list"))
        .filter_map(|m| {
            let id = m["slug"].as_str().or(m["id"].as_str())?;
            Some(
                codex_catalog
                    .iter()
                    .find(|k| k.id == id)
                    .cloned()
                    .unwrap_or_else(|| ModelInfo {
                        id: id.to_string(),
                        display_name: id.to_string(),
                        efforts: vec![],
                        default_effort: None,
                        input_modalities: vec![],
                        is_default: false,
                    }),
            )
        })
        .collect();
    // Signed out / empty list: nothing invented.
    if listed.is_empty() {
        return vec![];
    }
    plan_catalog(&listed)
}

/// The models the ChatGPT plan route offers (017): exactly Aura's plan
/// catalog (GPT-6 Luna, GPT-6.1 Sol, GPT-6 Astra), in that order, whatever
/// else the server lists. Efforts and modalities come from the server entry
/// when it describes the model (Codex `model/list`), else from the catalog.
/// The default is the first of them the server lists (Luna when it does).
pub fn plan_catalog(server: &[ModelInfo]) -> Vec<ModelInfo> {
    let mut out: Vec<ModelInfo> = aura_core::model_catalog::KNOWN
        .iter()
        .filter(|k| k.in_plan)
        .map(|k| {
            let mut m = from_catalog(k);
            if let Some(s) = server.iter().find(|s| s.id == k.id) {
                if !s.efforts.is_empty() {
                    m.efforts = s.efforts.clone();
                    m.default_effort = s
                        .default_effort
                        .clone()
                        .filter(|d| s.efforts.contains(d))
                        .or_else(|| m.default_effort.clone().filter(|d| s.efforts.contains(d)))
                        .or_else(|| s.efforts.first().cloned());
                }
                if !s.input_modalities.is_empty() {
                    m.input_modalities = s.input_modalities.clone();
                }
            }
            m
        })
        .collect();
    let default = out
        .iter()
        .position(|m| server.iter().any(|s| s.id == m.id))
        .unwrap_or(0);
    if let Some(m) = out.get_mut(default) {
        m.is_default = true;
    }
    out
}

fn from_catalog(k: &aura_core::model_catalog::KnownModel) -> ModelInfo {
    ModelInfo {
        id: k.id.to_string(),
        display_name: k.display_name.to_string(),
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

    fn names(models: &[ModelInfo]) -> Vec<&str> {
        models.iter().map(|m| m.display_name.as_str()).collect()
    }

    fn server_model(id: &str, efforts: &[&str], default: &str) -> ModelInfo {
        ModelInfo {
            id: id.into(),
            display_name: id.into(),
            efforts: efforts.iter().map(|e| e.to_string()).collect(),
            default_effort: Some(default.into()),
            input_modalities: vec!["text".into(), "image".into()],
            is_default: false,
        }
    }

    #[test]
    fn the_plan_offers_only_luna_sol_and_astra() {
        // 017 AC-001: older models listed by the server are dropped; the
        // GPT-6 trio always appears, in this order, Luna as the default.
        let server = [
            server_model("gpt-5.6-sol", &["low", "medium"], "medium"),
            server_model("gpt-6-luna", &["low", "medium", "high"], "low"),
            server_model("gpt-5.5", &["low"], "low"),
        ];
        let models = plan_catalog(&server);
        assert_eq!(names(&models), ["GPT-6 Luna", "GPT-6.1 Sol", "GPT-6 Astra"]);
        assert!(models[0].is_default && !models[1].is_default && !models[2].is_default);
        // Luna as the server describes it; Sol and Astra from the catalog.
        assert_eq!(models[0].efforts, ["low", "medium", "high"]);
        assert_eq!(models[0].default_effort.as_deref(), Some("low"));
        assert_eq!(models[1].efforts, ["low", "medium", "high", "xhigh", "max"]);
        assert_eq!(models[2].default_effort.as_deref(), Some("medium"));
        assert!(models.iter().all(ModelInfo::accepts_images));
    }

    #[test]
    fn the_default_is_the_first_trio_model_the_server_lists() {
        let models = plan_catalog(&[server_model("gpt-6-astra", &["high"], "high")]);
        assert_eq!(names(&models), ["GPT-6 Luna", "GPT-6.1 Sol", "GPT-6 Astra"]);
        assert!(models[2].is_default && !models[0].is_default);
        // Server default outside its own efforts: first supported one.
        let odd = plan_catalog(&[server_model("gpt-6-luna", &["high", "max"], "minimal")]);
        assert_eq!(odd[0].default_effort.as_deref(), Some("high"));
        // Nothing from the server: the trio, Luna first.
        assert!(plan_catalog(&[])[0].is_default);
    }

    #[test]
    fn plan_models_body_keeps_only_the_catalog() {
        let v = json!({"models": [
            {"slug": "gpt-5.6-luna", "display_name": "GPT-5.6 Luna", "visibility": "list"},
            {"slug": "gpt-6-astra", "visibility": "hidden"},
            {"slug": "gpt-6.1-sol", "display_name": "GPT-6.1 Sol", "visibility": "list"}]});
        let models = parse_plan_models(&v, &[]);
        assert_eq!(names(&models), ["GPT-6 Luna", "GPT-6.1 Sol", "GPT-6 Astra"]);
        assert!(models[1].is_default);
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
