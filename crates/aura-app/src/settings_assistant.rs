//! "Configurar com IA" (022): what the agent may read and change in the
//! user's settings. Pure and testable: `propose` validates a change set and
//! returns a before → after diff without saving; the host applies it and keeps
//! an undo snapshot. Secrets, YOLO, shortcuts and data deletion are not here.

use aura_core::settings::{Settings, SettingsPatch};
use serde_json::{Map, Value, json};

/// `(key, description for the model)` of every setting the agent may change.
pub const EDITABLE: &[(&str, &str)] = &[
    ("theme", "Theme: \"system\", \"light\" or \"dark\"."),
    ("opacity", "Overlay opacity from 0.5 to 1.0."),
    ("language", "UI language: \"ptBr\" or \"en\"."),
    (
        "doubleTapCtrl",
        "Also open the Overlay by tapping Ctrl twice.",
    ),
    ("startWithWindows", "Start Aura with Windows."),
    (
        "hideOnBlur",
        "Hide the Overlay when the user clicks elsewhere.",
    ),
    (
        "attachScreenOnOpen",
        "Attach the screen as a chip every time the Overlay opens (widens what is sent).",
    ),
    (
        "sendAfterDictation",
        "Send the message right after dictating.",
    ),
    (
        "personalInstructions",
        "Free text the agent always follows (max 4000 chars).",
    ),
    (
        "memories",
        "Let the agent remember facts across conversations (widens what is kept).",
    ),
    (
        "asrLanguage",
        "Dictation language, ISO 639-1 (\"pt\", \"en\"); null follows the UI language.",
    ),
    (
        "asrVocabulary",
        "List of names and terms the speech model should get right.",
    ),
    ("autoRead", "Read every finished answer aloud."),
    (
        "meetingKeepAudio",
        "Keep the audio of a meeting after it ends; off keeps only the text (turning it on widens what is kept).",
    ),
    (
        "meetingRedactPii",
        "Mask CPF, cards, e-mails and phone numbers before the model reads a meeting.",
    ),
    (
        "hideFromCapture",
        "Keep Aura out of screen sharing and recordings (turning it off widens exposure).",
    ),
    (
        "accentColor",
        "Accent color as #rrggbb; null restores the default.",
    ),
];

/// Changes that widen what Aura captures, keeps or shows to others: the user
/// gets an explicit warning before they apply.
fn widens(key: &str, after: &Value) -> bool {
    matches!(
        (key, after),
        ("attachScreenOnOpen", Value::Bool(true))
            | ("memories", Value::Bool(true))
            | ("meetingKeepAudio", Value::Bool(true))
            | ("hideFromCapture", Value::Bool(false))
    )
}

/// Current values plus the rules, for `settings_describe`.
pub fn describe(settings: &Settings) -> Value {
    let current = serde_json::to_value(settings).unwrap_or(Value::Null);
    let items: Vec<Value> = EDITABLE
        .iter()
        .map(|(key, what)| json!({"key": key, "value": current[key], "about": what}))
        .collect();
    json!({
        "settings": items,
        "not_editable": "API keys and tokens, YOLO mode, global shortcuts, privacy exclusions and deleting data are changed only by the user in Settings."
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub key: String,
    pub before: Value,
    pub after: Value,
    pub widens: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Proposal {
    pub changes: Vec<Change>,
    /// Problems the agent can fix and retry; nothing is applied when non-empty.
    pub errors: Vec<String>,
}

impl Proposal {
    pub fn to_json(&self) -> Value {
        json!({
            "ok": self.errors.is_empty(),
            "errors": self.errors,
            "changes": self.changes.iter().map(|c| json!({
                "key": c.key, "before": c.before, "after": c.after, "widens_exposure": c.widens,
            })).collect::<Vec<_>>(),
        })
    }
}

/// Validates `changes` (an object of key → value) against `current`.
pub fn propose(current: &Settings, changes: &Value) -> (Proposal, Option<SettingsPatch>) {
    let mut proposal = Proposal::default();
    let Some(map) = changes.as_object().filter(|m| !m.is_empty()) else {
        proposal
            .errors
            .push("changes must be a non-empty object".into());
        return (proposal, None);
    };
    for key in map.keys() {
        if !EDITABLE.iter().any(|(k, _)| k == key) {
            proposal
                .errors
                .push(format!("\"{key}\" cannot be changed by the agent"));
        }
    }
    if !proposal.errors.is_empty() {
        return (proposal, None);
    }
    let patch: SettingsPatch = match serde_json::from_value(Value::Object(map.clone())) {
        Ok(p) => p,
        Err(e) => {
            proposal.errors.push(format!("invalid value: {e}"));
            return (proposal, None);
        }
    };
    let next = match current.apply(&patch) {
        Ok(n) => n,
        Err(e) => {
            proposal.errors.push(e.to_string());
            return (proposal, None);
        }
    };
    let (before, after) = (
        serde_json::to_value(current).unwrap_or(Value::Null),
        serde_json::to_value(&next).unwrap_or(Value::Null),
    );
    for key in map.keys() {
        if before[key] != after[key] {
            proposal.changes.push(Change {
                key: key.clone(),
                before: before[key].clone(),
                after: after[key].clone(),
                widens: widens(key, &after[key]),
            });
        }
    }
    (proposal, Some(patch))
}

/// The patch that restores the values `proposal` changed.
pub fn inverse(proposal: &Proposal) -> Value {
    let mut m = Map::new();
    for c in &proposal.changes {
        m.insert(c.key.clone(), c.before.clone());
    }
    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_editable_key_is_a_real_setting_and_a_patch_field() {
        let s = serde_json::to_value(Settings::default()).unwrap();
        let p = serde_json::to_value(SettingsPatch::default()).unwrap();
        let _ = p;
        for (key, _) in EDITABLE {
            assert!(s.get(key).is_some(), "{key} is not a setting");
            // Accepted by the patch type (a typo would be ignored silently).
            let patch: SettingsPatch =
                serde_json::from_value(json!({ *key: s[key].clone() })).unwrap();
            assert_ne!(
                patch,
                SettingsPatch::default(),
                "{key} not in SettingsPatch"
            );
        }
    }

    #[test]
    fn secrets_yolo_and_shortcuts_are_not_editable() {
        for key in [
            "yolo",
            "invokeShortcut",
            "pushToTalkShortcut",
            "ttsCloudConsent",
            "disabledSkills",
        ] {
            let (p, patch) = propose(&Settings::default(), &json!({ key: true }));
            assert!(patch.is_none() && !p.errors.is_empty(), "{key}");
        }
    }

    #[test]
    fn propose_returns_a_diff_without_changing_anything() {
        let s = Settings::default();
        let (p, patch) = propose(&s, &json!({"opacity": 0.7, "autoRead": true}));
        assert!(p.errors.is_empty() && patch.is_some());
        let keys: Vec<_> = p.changes.iter().map(|c| c.key.as_str()).collect();
        assert!(keys.contains(&"opacity") && keys.contains(&"autoRead"));
        let o = p.changes.iter().find(|c| c.key == "opacity").unwrap();
        assert_eq!(
            (o.before.clone(), o.after.clone()),
            (json!(0.92), json!(0.7))
        );
        assert_eq!(inverse(&p)["opacity"], json!(0.92));
    }

    #[test]
    fn out_of_range_and_unchanged_values_are_reported_or_skipped() {
        let s = Settings::default();
        let (p, patch) = propose(&s, &json!({"opacity": 0.1}));
        assert!(patch.is_none() && !p.errors.is_empty());
        let (p, _) = propose(&s, &json!({"hideFromCapture": true}));
        assert!(p.changes.is_empty(), "already true: no diff");
    }

    #[test]
    fn exposure_widening_changes_are_flagged() {
        let s = Settings::default();
        let (p, _) = propose(
            &s,
            &json!({"memories": true, "attachScreenOnOpen": true, "hideFromCapture": false}),
        );
        assert!(p.changes.len() == 3 && p.changes.iter().all(|c| c.widens));
        let (p, _) = propose(&s, &json!({"autoRead": true}));
        assert!(!p.changes[0].widens);
    }

    #[test]
    fn describe_lists_values_and_the_forbidden_areas() {
        let d = describe(&Settings::default());
        assert_eq!(d["settings"].as_array().unwrap().len(), EDITABLE.len());
        assert!(d["not_editable"].as_str().unwrap().contains("YOLO"));
    }
}
