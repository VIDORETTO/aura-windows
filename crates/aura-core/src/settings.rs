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
    /// Preferred microphone device id; `None` follows the current OS default.
    pub microphone_device_id: Option<String>,
    /// Output device recorded as System audio (loopback); None = OS default.
    pub system_audio_device_id: Option<String>,
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
    /// SKILL.md paths turned off in Settings (008 AC-001); changed through
    /// the skills commands, not settings patches.
    pub disabled_skills: Vec<String>,
    /// Offline Windows voice for reading answers (None = by UI language).
    pub tts_voice: Option<String>,
    /// BYOK provider used as a cloud voice (None = offline Windows voice).
    pub tts_provider: Option<String>,
    /// Voice name sent to the cloud provider.
    pub tts_cloud_voice: String,
    /// Providers the user agreed to send answer text to (009 AC-006);
    /// changed through `speech_consent`, not settings patches.
    pub tts_cloud_consent: Vec<String>,
    /// Read every finished answer aloud (009 AC-005).
    pub auto_read: bool,
    /// Accent color `#rrggbb` chosen by the user (012); None = Aura's default.
    pub accent_color: Option<String>,
    /// Reasoning effort per `provider::model` and mode (013).
    pub effort_presets: std::collections::BTreeMap<String, ModeEfforts>,
    /// YOLO (018): Task mode runs without asking for permission. Changed
    /// only through `yolo_set` (needs the typed confirmation), not patches.
    pub yolo: bool,
    /// Keep every Aura window out of screenshots, recordings and screen
    /// sharing (Meet, Teams, Discord, AnyDesk…), so only the user sees it.
    /// On by default; off lets the Aura appear in captures (demos, tutorials).
    pub hide_from_capture: bool,
    /// Keep the audio of a Meeting after it ends (off: only the text stays).
    pub meeting_keep_audio: bool,
    /// Broadcast mode (021): Aura shows no Windows notifications, so nothing
    /// of it appears on a shared screen; messages stay inside the Overlay.
    pub broadcast_mode: bool,
    /// Mask CPF, cards, e-mails and phones before the model reads a Meeting.
    pub meeting_redact_pii: bool,
}

/// Words that confirm turning YOLO on (018), in any case.
pub const YOLO_CONFIRMATIONS: [&str; 2] = ["ACEITO", "ACCEPT"];

/// Whether `typed` confirms turning YOLO on.
pub fn yolo_confirmed(typed: &str) -> bool {
    let typed = typed.trim().to_uppercase();
    YOLO_CONFIRMATIONS.contains(&typed.as_str())
}

/// Effort chosen for one model in each conversation mode (013).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ModeEfforts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
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
            microphone_device_id: None,
            system_audio_device_id: None,
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
            disabled_skills: Vec::new(),
            tts_voice: None,
            tts_provider: None,
            tts_cloud_voice: "alloy".into(),
            tts_cloud_consent: Vec::new(),
            auto_read: false,
            accent_color: None,
            effort_presets: Default::default(),
            yolo: false,
            hide_from_capture: true,
            meeting_keep_audio: false,
            broadcast_mode: false,
            meeting_redact_pii: false,
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
    #[serde(
        default,
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub microphone_device_id: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub system_audio_device_id: Option<Option<String>>,
    pub default_model: Option<Option<String>>,
    pub default_effort: Option<Option<String>>,
    pub personal_instructions: Option<String>,
    pub app_server_idle_minutes: Option<u32>,
    pub worker_idle_minutes: Option<u32>,
    pub memories: Option<bool>,
    #[serde(
        default,
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub asr_language: Option<Option<String>>,
    pub asr_vocabulary: Option<Vec<String>>,
    pub cloud_asr_provider: Option<Option<String>>,
    pub onboarded: Option<bool>,
    #[serde(
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub tts_voice: Option<Option<String>>,
    #[serde(
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub tts_provider: Option<Option<String>>,
    pub tts_cloud_voice: Option<String>,
    pub auto_read: Option<bool>,
    #[serde(
        deserialize_with = "nullable_patch",
        skip_serializing_if = "Option::is_none"
    )]
    pub accent_color: Option<Option<String>>,
    /// Replaces every preset (the UI sends the whole map).
    pub effort_presets: Option<std::collections::BTreeMap<String, ModeEfforts>>,
    pub hide_from_capture: Option<bool>,
    pub meeting_keep_audio: Option<bool>,
    pub broadcast_mode: Option<bool>,
    pub meeting_redact_pii: Option<bool>,
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

/// A missing patch field keeps its value; explicit JSON null clears it.
fn nullable_patch<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
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
        if let Some(v) = patch.broadcast_mode {
            next.broadcast_mode = v;
        }
        if let Some(v) = patch.meeting_keep_audio {
            next.meeting_keep_audio = v;
        }
        if let Some(v) = patch.meeting_redact_pii {
            next.meeting_redact_pii = v;
        }
        if let Some(v) = patch.hide_from_capture {
            next.hide_from_capture = v;
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
        if let Some(v) = &patch.microphone_device_id {
            if v.as_deref()
                .is_some_and(|id| id.chars().count() > 1024 || id.contains('\0'))
            {
                return Err(SettingsError::OutOfRange {
                    field: "microphoneDeviceId",
                });
            }
            next.microphone_device_id = v.clone().filter(|id| !id.trim().is_empty());
        }
        if let Some(v) = &patch.system_audio_device_id {
            if v.as_deref()
                .is_some_and(|id| id.chars().count() > 1024 || id.contains('\0'))
            {
                return Err(SettingsError::OutOfRange {
                    field: "systemAudioDeviceId",
                });
            }
            next.system_audio_device_id = v.clone().filter(|id| !id.trim().is_empty());
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
        if let Some(v) = &patch.tts_voice {
            if v.as_deref().is_some_and(|id| id.chars().count() > 512) {
                return Err(SettingsError::OutOfRange { field: "ttsVoice" });
            }
            next.tts_voice = v.clone().filter(|s| !s.trim().is_empty());
        }
        if let Some(v) = &patch.tts_provider {
            next.tts_provider = v.clone().filter(|s| !s.trim().is_empty());
        }
        if let Some(v) = &patch.tts_cloud_voice {
            let v = v.trim();
            if v.is_empty()
                || v.len() > 64
                || !v
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
            {
                return Err(SettingsError::OutOfRange {
                    field: "ttsCloudVoice",
                });
            }
            next.tts_cloud_voice = v.to_string();
        }
        if let Some(v) = patch.auto_read {
            next.auto_read = v;
        }
        if let Some(map) = &patch.effort_presets {
            let ok = map.len() <= 500
                && map.iter().all(|(k, e)| {
                    !k.is_empty()
                        && k.chars().count() <= 300
                        && [&e.chat, &e.task, &e.plan]
                            .iter()
                            .all(|v| v.as_deref().is_none_or(crate::model_catalog::is_effort))
                });
            if !ok {
                return Err(SettingsError::OutOfRange {
                    field: "effortPresets",
                });
            }
            next.effort_presets = map
                .iter()
                .filter(|(_, e)| e.chat.is_some() || e.task.is_some() || e.plan.is_some())
                .map(|(k, e)| (k.clone(), e.clone()))
                .collect();
        }
        if let Some(v) = &patch.accent_color {
            next.accent_color = match v {
                None => None,
                Some(hex) => {
                    let h = hex.trim();
                    let valid = h.len() == 7
                        && h.starts_with('#')
                        && h[1..].chars().all(|c| c.is_ascii_hexdigit());
                    if !valid {
                        return Err(SettingsError::OutOfRange {
                            field: "accentColor",
                        });
                    }
                    Some(h.to_ascii_lowercase())
                }
            };
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
    fn automatic_asr_language_clears_a_fixed_json_preference() {
        assert!(
            serde_json::to_value(SettingsPatch::default())
                .unwrap()
                .get("asrLanguage")
                .is_none()
        );
        let fixed = Settings::default()
            .apply(&serde_json::from_value(serde_json::json!({ "asrLanguage": "es" })).unwrap())
            .unwrap();
        let kept = fixed
            .apply(&serde_json::from_value(serde_json::json!({ "language": "en" })).unwrap())
            .unwrap();
        assert_eq!(kept.asr_language.as_deref(), Some("es"));
        let auto = kept
            .apply(&serde_json::from_value(serde_json::json!({ "asrLanguage": null })).unwrap())
            .unwrap();
        assert_eq!(auto.asr_language, None);
    }

    #[test]
    fn microphone_json_preference_can_be_selected_kept_and_cleared() {
        let selected = Settings::default()
            .apply(
                &serde_json::from_value(serde_json::json!({
                    "microphoneDeviceId": "Mic B (USB)"
                }))
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(&selected).unwrap()["microphoneDeviceId"],
            "Mic B (USB)"
        );
        let kept = selected
            .apply(&serde_json::from_value(serde_json::json!({ "language": "en" })).unwrap())
            .unwrap();
        assert_eq!(
            serde_json::to_value(&kept).unwrap()["microphoneDeviceId"],
            "Mic B (USB)"
        );
        let cleared = kept
            .apply(
                &serde_json::from_value(serde_json::json!({ "microphoneDeviceId": null })).unwrap(),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(cleared).unwrap()["microphoneDeviceId"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn serializing_an_unspecified_microphone_patch_does_not_clear_the_preference() {
        let wire = serde_json::to_value(SettingsPatch::default()).unwrap();
        assert!(wire.get("microphoneDeviceId").is_none());
    }

    #[test]
    fn system_audio_device_is_nullable_bounded_and_kept_when_omitted() {
        assert_eq!(
            serde_json::from_str::<Settings>("{}")
                .unwrap()
                .system_audio_device_id,
            None
        );
        let patch: SettingsPatch =
            serde_json::from_str(r#"{"systemAudioDeviceId":"Speakers (USB)"}"#).unwrap();
        let chosen = Settings::default().apply(&patch).unwrap();
        assert_eq!(
            chosen.system_audio_device_id.as_deref(),
            Some("Speakers (USB)")
        );
        let kept = chosen
            .apply(&serde_json::from_str(r#"{"opacity":0.9}"#).unwrap())
            .unwrap();
        assert_eq!(
            kept.system_audio_device_id.as_deref(),
            Some("Speakers (USB)")
        );
        let cleared = kept
            .apply(&serde_json::from_str(r#"{"systemAudioDeviceId":null}"#).unwrap())
            .unwrap();
        assert_eq!(cleared.system_audio_device_id, None);
        assert_eq!(
            chosen
                .apply(&SettingsPatch {
                    system_audio_device_id: Some(Some("Out\0".into())),
                    ..Default::default()
                })
                .unwrap_err(),
            SettingsError::OutOfRange {
                field: "systemAudioDeviceId"
            }
        );
    }

    #[test]
    fn microphone_ids_are_preserved_bounded_and_default_on_blank_or_legacy_settings() {
        assert_eq!(
            serde_json::from_str::<Settings>("{}")
                .unwrap()
                .microphone_device_id,
            None
        );
        let selected = Settings::default()
            .apply(&SettingsPatch {
                microphone_device_id: Some(Some(" Mic B ".into())),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(selected.microphone_device_id.as_deref(), Some(" Mic B "));
        assert_eq!(
            selected
                .apply(&SettingsPatch {
                    microphone_device_id: Some(Some("  ".into())),
                    ..Default::default()
                })
                .unwrap()
                .microphone_device_id,
            None
        );
        for invalid in ["x".repeat(1025), "Mic\0B".into()] {
            assert_eq!(
                selected
                    .apply(&SettingsPatch {
                        microphone_device_id: Some(Some(invalid)),
                        ..Default::default()
                    })
                    .unwrap_err(),
                SettingsError::OutOfRange {
                    field: "microphoneDeviceId"
                }
            );
        }
    }

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
    fn effort_presets_per_model_and_mode() {
        // 013 AC-002: provider::model → effort per mode; null clears a mode.
        let s = Settings::default();
        assert!(s.effort_presets.is_empty());
        let patch = |v: serde_json::Value| -> SettingsPatch {
            serde_json::from_value(serde_json::json!({ "effortPresets": v })).unwrap()
        };
        let next = s
            .apply(&patch(serde_json::json!({
                "aura-qa::qa-reasoner": {"chat": "low", "task": "max"},
                "aura-chatgpt-plan::gpt-6.1-sol": {"plan": "xhigh", "chat": null}
            })))
            .unwrap();
        let qa = &next.effort_presets["aura-qa::qa-reasoner"];
        assert_eq!(
            (qa.chat.as_deref(), qa.task.as_deref(), qa.plan.as_deref()),
            (Some("low"), Some("max"), None)
        );
        assert_eq!(
            next.effort_presets["aura-chatgpt-plan::gpt-6.1-sol"]
                .plan
                .as_deref(),
            Some("xhigh")
        );
        assert_eq!(
            next.apply(&patch(serde_json::json!({"a::b": {"chat": "ultra"}})))
                .unwrap_err(),
            SettingsError::OutOfRange {
                field: "effortPresets"
            }
        );
        // Entries without any effort are dropped.
        let cleared = next
            .apply(&patch(serde_json::json!({"aura-qa::qa-reasoner": {}})))
            .unwrap();
        assert!(cleared.effort_presets.is_empty());
    }

    #[test]
    fn accent_color_is_a_hex_rgb_or_the_default() {
        // 012 AC-002.
        let s = Settings::default();
        assert_eq!(s.accent_color, None);
        let patch = |v: serde_json::Value| -> SettingsPatch {
            serde_json::from_value(serde_json::json!({ "accentColor": v })).unwrap()
        };
        let next = s.apply(&patch(serde_json::json!("#E4572E"))).unwrap();
        assert_eq!(next.accent_color.as_deref(), Some("#e4572e"));
        for bad in ["red", "#e4572", "#e4572ez", "e4572e", "#e4572e00"] {
            assert_eq!(
                next.apply(&patch(serde_json::json!(bad))).unwrap_err(),
                SettingsError::OutOfRange {
                    field: "accentColor"
                },
                "{bad}"
            );
        }
        assert_eq!(
            next.apply(&patch(serde_json::Value::Null))
                .unwrap()
                .accent_color,
            None
        );
    }

    #[test]
    fn windows_are_hidden_from_capture_by_default_and_can_be_shown() {
        let s = Settings::default();
        assert!(s.hide_from_capture);
        let off = s
            .apply(&SettingsPatch {
                hide_from_capture: Some(false),
                ..Default::default()
            })
            .unwrap();
        assert!(!off.hide_from_capture);
        // A stored settings blob from before this option keeps the safe default.
        let old: Settings = serde_json::from_str(r#"{"opacity":0.9}"#).unwrap();
        assert!(old.hide_from_capture);
    }

    #[test]
    fn broadcast_mode_is_off_by_default_and_toggles() {
        let s = Settings::default();
        assert!(!s.broadcast_mode);
        let on = s
            .apply(&SettingsPatch {
                broadcast_mode: Some(true),
                ..Default::default()
            })
            .unwrap();
        assert!(on.broadcast_mode);
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
