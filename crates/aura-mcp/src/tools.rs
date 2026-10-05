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
pub const EXTENSIONS_LIST: &str = "extensions_list";
pub const SKILL_SAVE: &str = "skill_save";
pub const QUICK_COMMAND_SAVE: &str = "quick_command_save";
pub const MCP_SERVER_SAVE: &str = "mcp_server_save";
pub const SETTINGS_DESCRIBE: &str = "settings_describe";
pub const SETTINGS_PROPOSE: &str = "settings_propose";
pub const SETTINGS_APPLY: &str = "settings_apply";
pub const SETTINGS_UNDO: &str = "settings_undo";

/// Tools that change the user's configuration: Codex asks the user before
/// each call (017).
pub const WRITE_TOOLS: [&str; 5] = [
    SKILL_SAVE,
    QUICK_COMMAND_SAVE,
    MCP_SERVER_SAVE,
    SETTINGS_APPLY,
    SETTINGS_UNDO,
];

fn writes(title: &str) -> serde_json::Value {
    json!({"title": title, "readOnlyHint": false, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false})
}

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
        ToolDef {
            name: EXTENSIONS_LIST.into(),
            description: "List the user's Aura extensions: skills (name, description, origin, enabled), quick commands (name, template, builtin, enabled) and MCP servers (name, transport, command or URL, enabled, secret variable names). Call before creating one to avoid duplicates."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Extensões do Aura"),
        },
        ToolDef {
            name: SKILL_SAVE.into(),
            description: "Create (or, with replace=true, rewrite) an Aura skill: a reusable instruction set the agent loads when its description matches the task. name: lowercase letters, digits and single hyphens (max 64). description: when to use it, max 1024 chars. instructions: Markdown body. The user approves the call; it applies to new conversations."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "name": {"type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$", "maxLength": 64},
                "description": {"type": "string", "maxLength": 1024},
                "instructions": {"type": "string"},
                "replace": {"type": "boolean", "default": false}},
                "required": ["name", "description", "instructions"], "additionalProperties": false}),
            annotations: writes("Salvar Skill"),
        },
        ToolDef {
            name: QUICK_COMMAND_SAVE.into(),
            description: "Create (or, with replace=true, rewrite) a quick command the user runs as /name in Aura's input bar. The template is a prompt with placeholders: {selecao} = selected text, else the typed text; {texto} = typed text only; {args} or {args:default} = first word after the command; {tela} = attach a screenshot. Built-in commands cannot be changed. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "name": {"type": "string", "pattern": "^[a-z0-9-]+$", "maxLength": 64},
                "template": {"type": "string"},
                "replace": {"type": "boolean", "default": false}},
                "required": ["name", "template"], "additionalProperties": false}),
            annotations: writes("Salvar comando rápido"),
        },
        ToolDef {
            name: MCP_SERVER_SAVE.into(),
            description: "Add (or update) an MCP server for future conversations. transport=stdio needs command (+ args, plain env); transport=http needs url. Never ask for or pass secret values: list secret variable names in secret_env (stdio) or set bearer=true (http); such servers are saved disabled until the user types the secret in Settings > Extensions. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "name": {"type": "string", "pattern": "^[A-Za-z0-9_-]+$", "maxLength": 64},
                "transport": {"type": "string", "enum": ["stdio", "http"]},
                "command": {"type": "string"},
                "args": {"type": "array", "items": {"type": "string"}},
                "env": {"type": "object", "additionalProperties": {"type": "string"}, "description": "Non-secret environment variables."},
                "secret_env": {"type": "array", "items": {"type": "string"}, "description": "Names of secret environment variables (values are typed by the user)."},
                "url": {"type": "string"},
                "bearer": {"type": "boolean", "default": false},
                "approval": {"type": "string", "enum": ["alwaysAsk", "askForWrites", "auto"], "default": "askForWrites"}},
                "required": ["name", "transport"], "additionalProperties": false}),
            annotations: writes("Salvar servidor MCP"),
        },
        ToolDef {
            name: SETTINGS_DESCRIBE.into(),
            description: "Read the Aura settings the agent may change, with their current values and what each does, plus what only the user can change. Call before proposing a change."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Configurações do Aura"),
        },
        ToolDef {
            name: SETTINGS_PROPOSE.into(),
            description: "Validate a settings change WITHOUT saving it and return the before/after diff (and which changes widen what Aura captures, keeps or shows). Show the diff to the user in plain language and ask before calling settings_apply."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "changes": {"type": "object", "description": "Setting key to new value, keys from settings_describe."}},
                "required": ["changes"], "additionalProperties": false}),
            annotations: read_only("Propor configuração"),
        },
        ToolDef {
            name: SETTINGS_APPLY.into(),
            description: "Apply a settings change set that the user agreed to (same shape as settings_propose). Never touches secrets, YOLO, shortcuts or data. The user approves the call and can undo it."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "changes": {"type": "object"}},
                "required": ["changes"], "additionalProperties": false}),
            annotations: writes("Aplicar configuração"),
        },
        ToolDef {
            name: SETTINGS_UNDO.into(),
            description: "Undo the last settings change the agent applied in this app session. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: writes("Desfazer configuração"),
        },
    ]
}
