//! Definitions (names, descriptions, schemas) of the Aura tools. Descriptions
//! are written for the model: concise, English, with when-to-use guidance.

use crate::ToolDef;
use serde_json::json;

fn read_only(title: &str) -> serde_json::Value {
    json!({"title": title, "readOnlyHint": true, "destructiveHint": false, "openWorldHint": false})
}

pub const SCREEN_CAPTURE: &str = "screen_capture";
pub const ACTIVE_WINDOW_INFO: &str = "active_window_info";
pub const SCREEN_TEXT: &str = "screen_text";
pub const SCREEN_RECENT: &str = "screen_recent";
pub const AUDIO_RECENT: &str = "audio_recent";
pub const ATTACHMENT_READ: &str = "attachment_read";

pub fn all() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: SCREEN_CAPTURE.into(),
            description: "Capture what the user currently sees. Use when the question depends on the screen and nothing was attached. \
                          target=monitor captures the monitor of the app the user was using; target=window captures only that window. \
                          Sensitive windows are covered or refused by the user's privacy settings; the user may be asked for permission."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "target": {"type": "string", "enum": ["monitor", "window"], "default": "monitor"},
                "reason": {"type": "string", "description": "Short reason shown to the user."}},
                "required": ["reason"], "additionalProperties": false}),
            annotations: read_only("Ver a tela"),
        },
        ToolDef {
            name: ACTIVE_WINDOW_INFO.into(),
            description: "Return the app, window title and, for browsers, the URL of the window the user was using. Cheap; prefer it before capturing."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("App ativo"),
        },
        ToolDef {
            name: SCREEN_TEXT.into(),
            description: "Read the text of the active window through UI Automation (exact text) or OCR (fallback). Use for reading documents, pages or error messages."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "source": {"type": "string", "enum": ["auto", "uia", "ocr"], "default": "auto"},
                "max_chars": {"type": "integer", "minimum": 100, "maximum": 20000, "default": 8000}},
                "additionalProperties": false}),
            annotations: read_only("Ler texto da tela"),
        },
        ToolDef {
            name: SCREEN_RECENT.into(),
            description: "Return up to 8 frames sampled from the last N minutes of screen recording, oldest first. Only works when the user enabled the recent screen buffer."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "minutes": {"type": "number", "minimum": 0.25, "maximum": 30},
                "max_frames": {"type": "integer", "minimum": 1, "maximum": 8, "default": 6},
                "reason": {"type": "string"}},
                "required": ["minutes", "reason"], "additionalProperties": false}),
            annotations: read_only("Tela recente"),
        },
        ToolDef {
            name: AUDIO_RECENT.into(),
            description: "Return the transcript of the last N minutes of audio (microphone as 'Você', system audio as 'Sistema') with timestamps. Only works when the user enabled the recent audio buffer."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "minutes": {"type": "number", "minimum": 0.25, "maximum": 30},
                "source": {"type": "string", "enum": ["mic", "system", "both"], "default": "both"},
                "reason": {"type": "string"}},
                "required": ["minutes", "reason"], "additionalProperties": false}),
            annotations: read_only("Áudio recente"),
        },
        ToolDef {
            name: ATTACHMENT_READ.into(),
            description: "Read a specific part of a file the user attached in this conversation: pages ('3-5'), a sheet and row range, slides, a time range or a section."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "attachment_id": {"type": "string"},
                "selector": {"type": "object", "properties": {
                    "pages": {"type": "string"}, "sheet": {"type": "string"}, "rows": {"type": "string"},
                    "slides": {"type": "string"}, "time": {"type": "string"}, "section": {"type": "string"}},
                    "additionalProperties": false}},
                "required": ["attachment_id"], "additionalProperties": false}),
            annotations: read_only("Ler anexo"),
        },
    ]
}
