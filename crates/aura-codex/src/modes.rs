//! Chat/Task modes and composition of instructions (AC-024/025 of 002,
//! AC-016 of 008).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "mode",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ConversationMode {
    /// Read-only: answers and read tools, never writes.
    Chat,
    /// May write in the conversation workspace and granted folders.
    Task {
        granted: Vec<PathBuf>,
        network: bool,
    },
    /// Planning: reads and researches, never executes.
    Plan,
}

impl ConversationMode {
    pub fn key(&self) -> &'static str {
        match self {
            ConversationMode::Chat => "chat",
            ConversationMode::Task { .. } => "task",
            ConversationMode::Plan => "plan",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum UiLanguage {
    #[default]
    PtBr,
    En,
}

pub fn persona(lang: UiLanguage) -> &'static str {
    match lang {
        UiLanguage::PtBr => include_str!("../persona/pt-BR.md"),
        UiLanguage::En => include_str!("../persona/en.md"),
    }
}

fn mode_instructions(mode: &ConversationMode, lang: UiLanguage) -> &'static str {
    match (mode, lang) {
        (ConversationMode::Chat, UiLanguage::PtBr) => {
            "Modo Chat: responda e use apenas ferramentas de leitura. Não altere arquivos nem execute comandos."
        }
        (ConversationMode::Chat, UiLanguage::En) => {
            "Chat mode: answer and use read-only tools only. Do not change files or run commands."
        }
        (ConversationMode::Task { .. }, UiLanguage::PtBr) => {
            "Modo Tarefa: planeje antes de agir, trabalhe no diretório da conversa e nas pastas concedidas e peça aprovação para efeitos."
        }
        (ConversationMode::Task { .. }, UiLanguage::En) => {
            "Task mode: plan before acting, work in the conversation directory and granted folders, and request approval for side effects."
        }
        (ConversationMode::Plan, UiLanguage::PtBr) => {
            "Modo Plano: leia e pesquise, mas não execute comandos nem altere arquivos. Termine com um plano numerado e objetivo para aprovação."
        }
        (ConversationMode::Plan, UiLanguage::En) => {
            "Plan mode: read and research, but do not run commands or change files. Finish with a concise numbered plan for approval."
        }
    }
}

/// Marker of the per-turn mode announcement (014): stripped from transcripts.
pub const MODE_NOTE_OPEN: &str = "<aura-mode>";
pub const MODE_NOTE_CLOSE: &str = "</aura-mode>";

/// Announces a mode chosen in the middle of a conversation. The developer
/// instructions of a thread are fixed when it starts, so the new mode is
/// stated in the turn and overrides the earlier one.
pub fn mode_change_note(mode: &ConversationMode, lang: UiLanguage) -> String {
    let lead = match lang {
        UiLanguage::PtBr => {
            "Mudança de modo pedida pelo usuário: a partir desta mensagem vale o modo abaixo, que substitui a instrução de modo anterior."
        }
        UiLanguage::En => {
            "Mode change requested by the user: from this message on the mode below applies and replaces the earlier mode instruction."
        }
    };
    format!(
        "{MODE_NOTE_OPEN}{lead} {}{MODE_NOTE_CLOSE}",
        mode_instructions(mode, lang)
    )
}

/// Removes mode announcements from a user message (transcripts).
pub fn strip_mode_notes(text: &str) -> String {
    let mut out = text.to_string();
    while let Some(start) = out.find(MODE_NOTE_OPEN) {
        let Some(end) = out[start..].find(MODE_NOTE_CLOSE) else {
            break;
        };
        out.replace_range(start..start + end + MODE_NOTE_CLOSE.len(), "");
    }
    out.trim().to_string()
}

/// Layers, in order: mode, personal instructions, app profile, conversation extras.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstructionLayers {
    pub personal: String,
    pub profile: Option<(String, String)>,
    pub conversation: String,
    pub previous_summary: Option<String>,
}

pub fn developer_instructions(
    mode: &ConversationMode,
    lang: UiLanguage,
    layers: &InstructionLayers,
) -> String {
    let mut parts = vec![mode_instructions(mode, lang).to_string()];
    let (user_label, profile_label, conv_label, summary_label) = match lang {
        UiLanguage::PtBr => (
            "Instruções do usuário",
            "Perfil",
            "Nesta conversa",
            "Resumo da conversa anterior",
        ),
        UiLanguage::En => (
            "User instructions",
            "Profile",
            "In this conversation",
            "Summary of the previous conversation",
        ),
    };
    if !layers.personal.trim().is_empty() {
        parts.push(format!("{user_label}: {}", layers.personal.trim()));
    }
    if let Some((name, text)) = &layers.profile
        && !text.trim().is_empty()
    {
        parts.push(format!("{profile_label} {name}: {}", text.trim()));
    }
    if !layers.conversation.trim().is_empty() {
        parts.push(format!("{conv_label}: {}", layers.conversation.trim()));
    }
    if let Some(summary) = layers
        .previous_summary
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        parts.push(format!("{summary_label}:\n{}", summary.trim()));
    }
    parts.join("\n\n")
}

fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// `sandbox`/`approvalPolicy`/`cwd` for `thread/start`.
pub fn thread_params(mode: &ConversationMode, workspace: &Path) -> Value {
    let sandbox = match mode {
        ConversationMode::Task { .. } => "workspace-write",
        _ => "read-only",
    };
    // Protocol spellings (v0.159): SandboxMode and AskForApproval are kebab-case.
    json!({"sandbox": sandbox, "approvalPolicy": "on-request", "cwd": path_str(workspace)})
}

/// Per-turn overrides (`sandboxPolicy`, `approvalPolicy`, `cwd`).
pub fn turn_overrides(mode: &ConversationMode, workspace: &Path) -> Value {
    let policy = match mode {
        ConversationMode::Task { granted, network } => {
            let mut roots = vec![path_str(workspace)];
            roots.extend(granted.iter().map(|p| path_str(p)));
            roots.dedup();
            json!({"type": "workspaceWrite", "writableRoots": roots, "networkAccess": network})
        }
        _ => json!({"type": "readOnly"}),
    };
    json!({"sandboxPolicy": policy, "approvalPolicy": "on-request", "cwd": path_str(workspace)})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_is_read_only() {
        let w = Path::new("/w/t1");
        assert_eq!(
            thread_params(&ConversationMode::Chat, w),
            json!({"sandbox": "read-only", "approvalPolicy": "on-request", "cwd": "/w/t1"})
        );
        assert_eq!(
            turn_overrides(&ConversationMode::Plan, w)["sandboxPolicy"],
            json!({"type": "readOnly"})
        );
    }

    #[test]
    fn task_writes_only_to_workspace_and_grants() {
        let w = Path::new("/w/t1");
        let mode = ConversationMode::Task {
            granted: vec!["/docs".into()],
            network: false,
        };
        assert_eq!(thread_params(&mode, w)["sandbox"], "workspace-write");
        assert_eq!(
            turn_overrides(&mode, w)["sandboxPolicy"],
            json!({"type": "workspaceWrite", "writableRoots": ["/w/t1", "/docs"], "networkAccess": false})
        );
    }

    #[test]
    fn instruction_layers_compose_in_order() {
        let layers = InstructionLayers {
            personal: "Responda em português, seja direto.".into(),
            profile: Some(("VS Code".into(), "Responda com código TypeScript".into())),
            conversation: "Foque em Excel".into(),
            previous_summary: None,
        };
        let text = developer_instructions(&ConversationMode::Chat, UiLanguage::PtBr, &layers);
        assert_eq!(
            text,
            "Modo Chat: responda e use apenas ferramentas de leitura. Não altere arquivos nem execute comandos.\n\n\
             Instruções do usuário: Responda em português, seja direto.\n\n\
             Perfil VS Code: Responda com código TypeScript\n\n\
             Nesta conversa: Foque em Excel"
        );
    }

    #[test]
    fn persona_asks_for_web_search_on_live_data() {
        // 015 AC-010: news, results, prices and scores change; search instead of guessing.
        assert!(persona(UiLanguage::PtBr).contains("use a busca na web"));
        assert!(persona(UiLanguage::En).contains("use web search"));
    }

    #[test]
    fn persona_mentions_aura_and_not_coding_agent_role() {
        assert!(persona(UiLanguage::PtBr).starts_with("Você é o Aura"));
        assert!(persona(UiLanguage::En).contains("not a coding agent by default"));
    }
}
