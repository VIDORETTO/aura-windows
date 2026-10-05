//! Models Aura knows more about than the provider lists say (013): efforts,
//! limits and modalities used to complete the ChatGPT plan list and model
//! discovery of compatible providers.

/// Reasoning efforts the pinned Codex app-server accepts, lowest first.
pub const EFFORTS: [&str; 7] = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownModel {
    pub id: &'static str,
    pub display_name: &'static str,
    pub efforts: &'static [&'static str],
    pub default_effort: &'static str,
    pub images: bool,
    pub tools: bool,
    /// Published limits; `None` when Aura does not know them.
    pub context_window: Option<u32>,
    pub max_output: Option<u32>,
    /// Offered in the ChatGPT plan (017: the plan shows only these).
    pub in_plan: bool,
}

/// Efforts used when the Codex `model/list` does not describe a model.
const GPT6_EFFORTS: &[&str] = &["low", "medium", "high", "xhigh", "max"];

/// The ChatGPT plan catalog, in the order the picker shows it (017):
/// GPT-6 Luna (fast), GPT-6.1 Sol and GPT-6 Astra (flagship; no `none`).
/// GPT-6.1 Sol (2026-09-29): `low`…`max`, default `medium`, 1,050,000
/// context, 128,000 output (013).
pub const KNOWN: &[KnownModel] = &[
    KnownModel {
        id: "gpt-6-luna",
        display_name: "GPT-6 Luna",
        efforts: GPT6_EFFORTS,
        default_effort: "medium",
        images: true,
        tools: true,
        context_window: None,
        max_output: None,
        in_plan: true,
    },
    KnownModel {
        id: "gpt-6.1-sol",
        display_name: "GPT-6.1 Sol",
        efforts: GPT6_EFFORTS,
        default_effort: "medium",
        images: true,
        tools: true,
        context_window: Some(1_050_000),
        max_output: Some(128_000),
        in_plan: true,
    },
    KnownModel {
        id: "gpt-6-astra",
        display_name: "GPT-6 Astra",
        efforts: GPT6_EFFORTS,
        default_effort: "medium",
        images: true,
        tools: true,
        context_window: None,
        max_output: None,
        in_plan: true,
    },
];

/// Looks a model up by id, also as `vendor/id` (OpenRouter style).
pub fn known_model(id: &str) -> Option<&'static KnownModel> {
    let id = id.trim().to_ascii_lowercase();
    let bare = id.rsplit('/').next().unwrap_or(&id);
    KNOWN.iter().find(|k| k.id == bare)
}

/// Whether `effort` is one Aura can send.
pub fn is_effort(effort: &str) -> bool {
    EFFORTS.contains(&effort)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpt_6_1_sol_is_known_with_its_efforts_and_limits() {
        let k = known_model("gpt-6.1-sol").unwrap();
        assert_eq!(k.efforts, ["low", "medium", "high", "xhigh", "max"]);
        assert_eq!(k.default_effort, "medium");
        assert_eq!(
            (k.context_window, k.max_output),
            (Some(1_050_000), Some(128_000))
        );
        assert!(k.images && k.tools && k.in_plan);
        assert_eq!(
            known_model("openai/GPT-6.1-Sol").map(|k| k.id),
            Some("gpt-6.1-sol")
        );
        assert!(known_model("gpt-6.1").is_none());
        assert!(is_effort("max") && is_effort("none") && !is_effort("ultra"));
    }

    #[test]
    fn the_plan_catalog_is_luna_sol_astra() {
        // 017 FR-001: only the GPT-6 family, in this order.
        let plan: Vec<_> = KNOWN.iter().filter(|k| k.in_plan).map(|k| k.id).collect();
        assert_eq!(plan, ["gpt-6-luna", "gpt-6.1-sol", "gpt-6-astra"]);
        assert_eq!(
            known_model("gpt-6-astra").unwrap().display_name,
            "GPT-6 Astra"
        );
        assert!(
            !known_model("gpt-6-astra")
                .unwrap()
                .efforts
                .contains(&"none")
        );
    }
}
