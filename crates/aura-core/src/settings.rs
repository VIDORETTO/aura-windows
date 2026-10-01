//! User settings with validation (`specs/001-fundacao-overlay`, FR-005).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Minimum and maximum Overlay opacity accepted by the UI (AC-010).
pub const MIN_OPACITY: f64 = 0.50;
pub const MAX_OPACITY: f64 = 1.00;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    #[default]
    PtBr,
    En,
}

/// Every persisted preference. Unknown keys found in storage are ignored so
/// newer versions never break older data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub theme: Theme,
    pub opacity: f64,
    pub language: Language,
    /// Accelerator in the form `Ctrl+Shift+Space`.
    pub invoke_shortcut: String,
    pub double_tap_ctrl: bool,
    pub start_with_windows: bool,
    /// Hide the Overlay when another window takes focus (off: it stays until
    /// the shortcut or "Minimizar para a bandeja"). Replaces the old
    /// `focusLoss`, whose stored `"hide"` default is deliberately ignored.
    pub hide_on_blur: bool,
    pub privacy_pause_shortcut: String,
    pub push_to_talk_shortcut: String,
    pub global_voice_shortcut: String,
    pub attach_screen_on_open: bool,
    pub send_after_dictation: bool,
    pub default_model: Option<String>,
    pub default_effort: Option<String>,
    pub personal_instructions: String,
    pub app_server_idle_minutes: u32,
    pub worker_idle_minutes: u32,
    /// Codex memories (008 TK-006): the agent may remember facts across conversations.
    pub memories: bool,
    /// Dictation language (ISO 639-1); `None` = the UI language.
    pub asr_language: Option<String>,
    /// Names and terms the speech model should get right (006 TK-005).
    pub asr_vocabulary: Vec<String>,
    /// BYOK provider id used for cloud transcription/fallback (006 TK-006).
    pub cloud_asr_provider: Option<String>,
    /// First-run onboarding finished (010 TK-003).
    pub onboarded: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            opacity: 0.92,
            language: Language::PtBr,
            invoke_shortcut: "Ctrl+Shift+Space".into(),
            double_tap_ctrl: false,
            start_with_windows: false,
            hide_on_blur: false,
            privacy_pause_shortcut: "Ctrl+Shift+Alt+P".into(),
            push_to_talk_shortcut: "Ctrl+Space".into(),
            global_voice_shortcut: "Ctrl+Alt+Space".into(),
            attach_screen_on_open: false,
            send_after_dictation: false,
            default_model: None,
            default_effort: None,
            personal_instructions: String::new(),
            app_server_idle_minutes: 15,
            worker_idle_minutes: 2,
            memories: false,
            asr_language: None,
            asr_vocabulary: Vec::new(),
            cloud_asr_provider: None,
            onboarded: false,
        }
    }
}

/// Partial update sent by the UI; `None` keeps the current value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SettingsPatch {
    pub theme: Option<Theme>,
    pub opacity: Option<f64>,
    pub language: Option<Language>,
    pub invoke_shortcut: Option<String>,
    pub double_tap_ctrl: Option<bool>,
    pub start_with_windows: Option<bool>,
    pub hide_on_blur: Option<bool>,
    pub privacy_pause_shortcut: Option<String>,
    pub push_to_talk_shortcut: Option<String>,
    pub global_voice_shortcut: Option<String>,
    pub attach_screen_on_open: Option<bool>,
    pub send_after_dictation: Option<bool>,
    pub default_model: Option<Option<String>>,
    pub default_effort: Option<Option<String>>,
    pub personal_instructions: Option<String>,
    pub app_server_idle_minutes: Option<u32>,
    pub worker_idle_minutes: Option<u32>,
    pub memories: Option<bool>,
    pub asr_language: Option<Option<String>>,
    pub asr_vocabulary: Option<Vec<String>>,
    pub cloud_asr_provider: Option<Option<String>>,
    pub onboarded: Option<bool>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("value out of range: {field}")]
    OutOfRange { field: &'static str },
    #[error("invalid shortcut: {0}")]
    InvalidShortcut(String),
    #[error("shortcut already registered by another application")]
    ShortcutInUse,
    #[error("storage error: {0}")]
    Storage(String),
}

impl Settings {
    /// Returns a new validated `Settings` with `patch` applied. The receiver is
    /// left untouched when validation fails.
    pub fn apply(&self, patch: &SettingsPatch) -> Result<Settings, SettingsError> {
        let mut next = self.clone();
        if let Some(theme) = patch.theme {
            next.theme = theme;
        }
        if let Some(opacity) = patch.opacity {
            if !opacity.is_finite() || !(MIN_OPACITY..=MAX_OPACITY).contains(&opacity) {
                return Err(SettingsError::OutOfRange { field: "opacity" });
            }
            next.opacity = (opacity * 100.0).round() / 100.0;
        }
        if let Some(language) = patch.language {
            next.language = language;
        }
        for (value, target) in [
            (&patch.invoke_shortcut, &mut next.invoke_shortcut),
            (
                &patch.privacy_pause_shortcut,
                &mut next.privacy_pause_shortcut,
            ),
            (
                &patch.push_to_talk_shortcut,
                &mut next.push_to_talk_shortcut,
            ),
            (
                &patch.global_voice_shortcut,
                &mut next.global_voice_shortcut,
            ),
        ] {
            if let Some(shortcut) = value {
                *target = Shortcut::parse(shortcut)?.to_string();
            }
        }
        if let Some(v) = patch.double_tap_ctrl {
            next.double_tap_ctrl = v;
        }
        if let Some(v) = patch.start_with_windows {
            next.start_with_windows = v;
        }
        if let Some(v) = patch.hide_on_blur {
            next.hide_on_blur = v;
        }
        if let Some(v) = patch.attach_screen_on_open {
            next.attach_screen_on_open = v;
        }
        if let Some(v) = patch.send_after_dictation {
            next.send_after_dictation = v;
        }
        if let Some(v) = &patch.default_model {
            next.default_model = v.clone();
        }
        if let Some(v) = &patch.default_effort {
            next.default_effort = v.clone();
        }
        if let Some(v) = &patch.personal_instructions {
            if v.chars().count() > 4000 {
                return Err(SettingsError::OutOfRange {
                    field: "personalInstructions",
                });
            }
            next.personal_instructions = v.clone();
        }
        if let Some(v) = patch.app_server_idle_minutes {
            if !(1..=240).contains(&v) {
                return Err(SettingsError::OutOfRange {
                    field: "appServerIdleMinutes",
                });
            }
            next.app_server_idle_minutes = v;
        }
        if let Some(v) = patch.worker_idle_minutes {
            if !(1..=60).contains(&v) {
                return Err(SettingsError::OutOfRange {
                    field: "workerIdleMinutes",
                });
            }
            next.worker_idle_minutes = v;
        }
        if let Some(v) = patch.memories {
            next.memories = v;
        }
        if let Some(v) = &patch.asr_language {
            if v.as_deref().is_some_and(|l| {
                l.len() < 2
                    || l.len() > 5
                    || !l.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
            }) {
                return Err(SettingsError::OutOfRange {
                    field: "asrLanguage",
                });
            }
            next.asr_language = v.clone();
        }
        if let Some(v) = &patch.asr_vocabulary {
            let cleaned: Vec<String> = v
                .iter()
                .map(|w| w.trim().to_string())
                .filter(|w| !w.is_empty())
                .collect();
            if cleaned.len() > 200 || cleaned.iter().any(|w| w.chars().count() > 64) {
                return Err(SettingsError::OutOfRange {
                    field: "asrVocabulary",
                });
            }
            next.asr_vocabulary = cleaned;
        }
        if let Some(v) = patch.onboarded {
            next.onboarded = v;
        }
        if let Some(v) = &patch.cloud_asr_provider {
            next.cloud_asr_provider = v.clone().filter(|s| !s.trim().is_empty());
        }
        Ok(next)
    }
}

/// Keyboard modifiers supported in accelerators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

/// A validated global shortcut such as `Ctrl+Shift+Space`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    /// Canonical key name (`Space`, `A`, `F5`, `Enter`...).
    pub key: String,
}

const NAMED_KEYS: &[&str] = &[
    "Space",
    "Enter",
    "Tab",
    "Escape",
    "Backspace",
    "Delete",
    "Insert",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Up",
    "Down",
    "Left",
    "Right",
    "Backquote",
    "Minus",
    "Equal",
    "Comma",
    "Period",
    "Slash",
    "Semicolon",
    "Quote",
    "BracketLeft",
    "BracketRight",
    "Backslash",
];

impl Shortcut {
    pub fn parse(input: &str) -> Result<Shortcut, SettingsError> {
        let invalid = || SettingsError::InvalidShortcut(input.to_string());
        let mut modifiers = Modifiers::default();
        let mut key: Option<String> = None;
        for raw in input.split('+').map(str::trim) {
            if raw.is_empty() {
                return Err(invalid());
            }
            match raw.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "option" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "win" | "super" | "meta" | "cmd" => modifiers.win = true,
                _ => {
                    if key.is_some() {
                        return Err(invalid());
                    }
                    key = Some(canonical_key(raw).ok_or_else(invalid)?);
                }
            }
        }
        let key = key.ok_or_else(invalid)?;
        let has_modifier = modifiers.ctrl || modifiers.alt || modifiers.win;
        // Function keys may be used alone; everything else needs Ctrl/Alt/Win so
        // typing never triggers the Overlay by accident.
        if !has_modifier && !key.starts_with('F') {
            return Err(invalid());
        }
        Ok(Shortcut { modifiers, key })
    }
}

fn canonical_key(raw: &str) -> Option<String> {
    if raw.chars().count() == 1 {
        let c = raw.chars().next()?;
        if c.is_ascii_alphanumeric() {
            return Some(c.to_ascii_uppercase().to_string());
        }
        return None;
    }
    let lower = raw.to_ascii_lowercase();
    if let Some(n) = lower.strip_prefix('f')
        && let Ok(n) = n.parse::<u8>()
        && (1..=24).contains(&n)
    {
        return Some(format!("F{n}"));
    }
    let alias = match lower.as_str() {
        "esc" => "Escape",
        "return" => "Enter",
        "del" => "Delete",
        "ins" => "Insert",
        "pgup" => "PageUp",
        "pgdn" => "PageDown",
        other => other,
    };
    NAMED_KEYS
        .iter()
        .find(|k| k.eq_ignore_ascii_case(alias))
        .map(|k| (*k).to_string())
}

impl std::fmt::Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = self.modifiers;
        let mut parts: Vec<&str> = Vec::new();
        if m.ctrl {
            parts.push("Ctrl");
        }
        if m.alt {
            parts.push("Alt");
        }
        if m.shift {
            parts.push("Shift");
        }
        if m.win {
            parts.push("Win");
        }
        parts.push(&self.key);
        f.write_str(&parts.join("+"))
    }
}

#[cfg(test)]
mod voice_settings_tests {
    use super::*;

    #[test]
    fn vocabulary_and_language_are_validated() {
        let s = Settings::default();
        let p = SettingsPatch {
            asr_vocabulary: Some(vec![" Aura ".into(), "".into(), "Parakeet".into()]),
            asr_language: Some(Some("pt".into())),
            memories: Some(true),
            ..Default::default()
        };
        let n = s.apply(&p).unwrap();
        assert_eq!(
            n.asr_vocabulary,
            vec!["Aura".to_string(), "Parakeet".to_string()]
        );
        assert_eq!(n.asr_language.as_deref(), Some("pt"));
        assert!(n.memories);
        let bad = SettingsPatch {
            asr_language: Some(Some("português".into())),
            ..Default::default()
        };
        assert!(s.apply(&bad).is_err());
        let long = SettingsPatch {
            asr_vocabulary: Some(vec!["x".repeat(65)]),
            ..Default::default()
        };
        assert!(s.apply(&long).is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_within_range_is_rounded_and_applied() {
        let s = Settings::default();
        let next = s
            .apply(&SettingsPatch {
                opacity: Some(0.8049),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(next.opacity, 0.80);
    }

    #[test]
    fn half_opacity_is_the_minimum() {
        let s = Settings::default();
        let next = s
            .apply(&SettingsPatch {
                opacity: Some(0.5),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(next.opacity, 0.5);
    }

    #[test]
    fn overlay_stays_open_on_blur_by_default() {
        assert!(!Settings::default().hide_on_blur);
    }

    #[test]
    fn opacity_below_minimum_is_rejected_and_previous_value_kept() {
        let s = Settings::default();
        let err = s
            .apply(&SettingsPatch {
                opacity: Some(0.49),
                ..Default::default()
            })
            .unwrap_err();
        assert_eq!(err, SettingsError::OutOfRange { field: "opacity" });
        assert_eq!(s.opacity, 0.92);
    }

    #[test]
    fn shortcut_is_canonicalised() {
        assert_eq!(
            Shortcut::parse("ctrl + shift + space").unwrap().to_string(),
            "Ctrl+Shift+Space"
        );
        assert_eq!(Shortcut::parse("Alt+k").unwrap().to_string(), "Alt+K");
        assert_eq!(Shortcut::parse("F9").unwrap().to_string(), "F9");
    }

    #[test]
    fn shortcut_without_modifier_or_with_two_keys_is_invalid() {
        assert!(Shortcut::parse("Space").is_err());
        assert!(Shortcut::parse("Shift+A").is_err());
        assert!(Shortcut::parse("Ctrl+A+B").is_err());
        assert!(Shortcut::parse("Ctrl+").is_err());
        assert!(Shortcut::parse("Ctrl+ç").is_err());
    }

    #[test]
    fn patch_with_invalid_shortcut_keeps_settings() {
        let s = Settings::default();
        let err = s
            .apply(&SettingsPatch {
                invoke_shortcut: Some("Space".into()),
                ..Default::default()
            })
            .unwrap_err();
        assert!(matches!(err, SettingsError::InvalidShortcut(_)));
    }

    #[test]
    fn unknown_keys_are_ignored_when_deserialising() {
        let s: Settings = serde_json::from_str(r#"{"theme":"dark","futureKey":1}"#).unwrap();
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(s.opacity, 0.92);
    }
}
