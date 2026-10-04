//! Native UI and built-in prompts use the canonical frontend dictionaries.
use aura_core::settings::Language;

include!(concat!(env!("OUT_DIR"), "/messages.rs"));

pub fn text(language: Language, key: &str) -> &'static str {
    let table = match language {
        Language::PtBr => PT_BR,
        Language::En => EN,
    };
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, value)| *value)
        .unwrap_or_else(|| panic!("unknown canonical UI message: {key}"))
}
