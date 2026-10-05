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
pub const CLOCK_NOW: &str = "clock_now";
pub const REMINDER_CREATE: &str = "reminder_create";
pub const REMINDER_LIST: &str = "reminder_list";
pub const REMINDER_DELETE: &str = "reminder_delete";
pub const NOTE_SAVE: &str = "note_save";
pub const NOTE_SEARCH: &str = "note_search";
pub const MEETING_SEARCH: &str = "meeting_search";
pub const MEETING_GET: &str = "meeting_get";
pub const MEETING_BRIEF_SAVE: &str = "meeting_brief_save";
pub const RECIPE_LIST: &str = "recipe_list";
pub const RECIPE_SAVE: &str = "recipe_save";
pub const ACTION_SAVE: &str = "action_save";
pub const ACTION_LIST: &str = "action_list";
pub const ACTION_DONE: &str = "action_done";
pub const OPEN_WINDOWS: &str = "open_windows";
pub const EXCLUSION_LIST: &str = "exclusion_list";
pub const EXCLUSION_ADD: &str = "exclusion_add";
pub const PROFILE_LIST: &str = "profile_list";
pub const PROFILE_SAVE: &str = "profile_save";
pub const SETTINGS_DESCRIBE: &str = "settings_describe";
pub const SETTINGS_PROPOSE: &str = "settings_propose";
pub const SETTINGS_APPLY: &str = "settings_apply";
pub const SETTINGS_UNDO: &str = "settings_undo";

/// Tools that change the user's configuration: Codex asks the user before
/// each call (017).
pub const WRITE_TOOLS: [&str; 13] = [
    ACTION_SAVE,
    ACTION_DONE,
    EXCLUSION_ADD,
    PROFILE_SAVE,
    RECIPE_SAVE,
    MEETING_BRIEF_SAVE,
    REMINDER_CREATE,
    REMINDER_DELETE,
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
        ToolDef {
            name: CLOCK_NOW.into(),
            description: "Current local date and time with the user's UTC offset (RFC 3339). Call before creating a reminder from a relative or clock time such as 'at 3 pm' or 'tomorrow'."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Data e hora"),
        },
        ToolDef {
            name: REMINDER_CREATE.into(),
            description: "Create a reminder that shows a Windows notification. Give either delay_minutes or at (local time 'YYYY-MM-DDTHH:MM', or RFC 3339). repeat: none, daily, weekdays or weekly. text: what to remind, in the user's language. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "text": {"type": "string", "maxLength": 500},
                "at": {"type": "string"},
                "delay_minutes": {"type": "integer", "minimum": 1},
                "repeat": {"type": "string", "enum": ["none", "daily", "weekdays", "weekly"], "default": "none"}},
                "required": ["text"], "additionalProperties": false}),
            annotations: writes("Criar lembrete"),
        },
        ToolDef {
            name: REMINDER_LIST.into(),
            description: "List the user's active reminders (id, text, due time, repeat).".into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Lembretes"),
        },
        ToolDef {
            name: REMINDER_DELETE.into(),
            description: "Delete an active reminder by id (from reminder_list). The user approves the call.".into(),
            input_schema: json!({"type": "object", "properties": {"id": {"type": "string"}},
                "required": ["id"], "additionalProperties": false}),
            annotations: writes("Apagar lembrete"),
        },
        ToolDef {
            name: NOTE_SAVE.into(),
            description: "Save a quick note the user dictated or typed ('anota: …') or, with kind=saved, an answer the user wants to keep. Keep the user's own words; do not add content."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "text": {"type": "string"},
                "kind": {"type": "string", "enum": ["note", "saved"], "default": "note"}},
                "required": ["text"], "additionalProperties": false}),
            annotations: writes("Salvar nota"),
        },
        ToolDef {
            name: NOTE_SEARCH.into(),
            description: "Search the user's notes (kind=note) or saved answers (kind=saved), newest first. Every word of query must appear; empty query lists the latest."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "query": {"type": "string"},
                "kind": {"type": "string", "enum": ["note", "saved"], "default": "note"}},
                "additionalProperties": false}),
            annotations: read_only("Buscar notas"),
        },
        ToolDef {
            name: MEETING_SEARCH.into(),
            description: "Search what was said in the user's saved meetings. Every word of query must appear (accents ignored). Returns meeting id, title, date, minute and speaker (you = the user's microphone, them = the other side). Use meeting_get to read one meeting. Cite the meeting and minute in answers."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "query": {"type": "string"},
                "meeting_id": {"type": "string", "description": "Limit the search to one meeting."}},
                "required": ["query"], "additionalProperties": false}),
            annotations: read_only("Buscar nas reuniões"),
        },
        ToolDef {
            name: MEETING_GET.into(),
            description: "Read one saved meeting: title, briefing and the full timed transcript (minute:second, speaker, text). Without meeting_id returns the list of recent meetings."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "meeting_id": {"type": "string"}},
                "additionalProperties": false}),
            annotations: read_only("Ler reunião"),
        },
        ToolDef {
            name: MEETING_BRIEF_SAVE.into(),
            description: "Save the meeting briefing ('Entendi assim': objective, what to leave with, agenda points, cautions) for the user to review in the Meeting panel. This does NOT start the meeting or any recording: only the user starts it. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "briefing": {"type": "string", "maxLength": 8000}},
                "required": ["briefing"], "additionalProperties": false}),
            annotations: writes("Salvar briefing da reunião"),
        },
        ToolDef {
            name: RECIPE_LIST.into(),
            description: "List meeting Recipes (id, name, description, notes_template, help_level, builtin). Call before creating one to avoid duplicates."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Receitas de reunião"),
        },
        ToolDef {
            name: RECIPE_SAVE.into(),
            description: "Create (or, with replace=true, rewrite) a meeting Recipe: how notes are organized for a kind of meeting. id: lowercase letters, digits, hyphens. notes_template: the sections to fill, separated by ' · '. help_level: silent, onDemand, balanced or active. Built-in Recipes cannot be changed. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "id": {"type": "string", "pattern": "^[a-z0-9-]+$", "maxLength": 40},
                "name": {"type": "string"},
                "description": {"type": "string"},
                "notes_template": {"type": "string"},
                "help_level": {"type": "string", "enum": ["silent", "onDemand", "balanced", "active"], "default": "onDemand"},
                "replace": {"type": "boolean", "default": false}},
                "required": ["id", "name", "notes_template"], "additionalProperties": false}),
            annotations: writes("Salvar Receita"),
        },
        ToolDef {
            name: OPEN_WINDOWS.into(),
            description: "List the windows currently open on the user's monitor (process name and title, front to back, max 30). Use it to propose privacy exclusions or app profiles for the apps the user names. Titles may contain private data: use them only for the user's request."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Janelas abertas"),
        },
        ToolDef {
            name: EXCLUSION_LIST.into(),
            description: "List the privacy exclusions: windows Aura never captures nor gives to the agent (id, process, title pattern, enabled, builtin)."
                .into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Janelas excluídas"),
        },
        ToolDef {
            name: EXCLUSION_ADD.into(),
            description: "Add a privacy exclusion so Aura never captures or reads matching windows. Give process (executable name, wildcards * and ?, e.g. 'KeePass.exe') and/or title_glob (e.g. '*Nubank*'); prefer process for a whole app. Only reduces exposure. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "process": {"type": "string"},
                "title_glob": {"type": "string"}},
                "additionalProperties": false}),
            annotations: writes("Excluir janela"),
        },
        ToolDef {
            name: PROFILE_LIST.into(),
            description: "List app profiles (id, name, process pattern, title pattern, instructions, attach_screen, default mode and model).".into(),
            input_schema: json!({"type": "object", "properties": {}, "additionalProperties": false}),
            annotations: read_only("Perfis de aplicativo"),
        },
        ToolDef {
            name: PROFILE_SAVE.into(),
            description: "Create or update an app profile applied when that app is in front: instructions (max 4000 chars), whether to attach the screen on open, default mode (chat, task or plan). Pass id from profile_list to update. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"},
                "process": {"type": "string", "description": "Executable pattern, e.g. 'code.exe' or '*chrome*'."},
                "title_glob": {"type": "string"},
                "instructions": {"type": "string", "maxLength": 4000},
                "attach_screen": {"type": "boolean", "default": false},
                "default_mode": {"type": "string", "enum": ["chat", "task", "plan"]}},
                "required": ["name", "process"], "additionalProperties": false}),
            annotations: writes("Salvar perfil de aplicativo"),
        },
        ToolDef {
            name: ACTION_SAVE.into(),
            description: "Save a commitment taken from a meeting: what, who owns it (owner=you when the user owes it, owner=them when someone promised the user), optional due date YYYY-MM-DD, the meeting_id and the minute 'mm:ss' it came from. Never invent commitments: save only what was said. The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "text": {"type": "string"},
                "owner": {"type": "string", "enum": ["you", "them"], "default": "you"},
                "due": {"type": "string", "description": "YYYY-MM-DD"},
                "meeting_id": {"type": "string"},
                "minute": {"type": "string", "description": "mm:ss in the meeting"}},
                "required": ["text"], "additionalProperties": false}),
            annotations: writes("Salvar compromisso"),
        },
        ToolDef {
            name: ACTION_LIST.into(),
            description: "List commitments (promises): status open or done, owner you (the user owes) or them (owed to the user). Open ones come by due date with overdue=true when late. Use it to answer 'what do I owe?' and 'what did they promise me?'."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "status": {"type": "string", "enum": ["open", "done"]},
                "owner": {"type": "string", "enum": ["you", "them"]}},
                "additionalProperties": false}),
            annotations: read_only("Compromissos"),
        },
        ToolDef {
            name: ACTION_DONE.into(),
            description: "Mark a commitment done (done=true, default) or open again (done=false). The user approves the call."
                .into(),
            input_schema: json!({"type": "object", "properties": {
                "id": {"type": "string"},
                "done": {"type": "boolean", "default": true}},
                "required": ["id"], "additionalProperties": false}),
            annotations: writes("Atualizar compromisso"),
        },
    ]
}
