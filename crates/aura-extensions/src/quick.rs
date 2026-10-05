//! Quick commands (`/tldr`, `/traduzir inglês`…): local, deterministic prompt
//! templates expanded before the turn is sent (AC-010, AC-011).
//!
//! Placeholders:
//! - `{selecao}` — the selection chip text, falling back to the typed text;
//! - `{texto}` — the typed text only;
//! - `{args}` / `{args:padrão}` — the first word after the command;
//! - `{area}` — the clipboard text (`/colar`); the host reads the clipboard
//!   only when the command uses it.

use aura_store::{Store, StoreError};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickCommand {
    pub name: String,
    pub template: String,
    pub builtin: bool,
    pub enabled: bool,
}

/// Context available when the command runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuickContext {
    pub selection: Option<String>,
    pub typed: String,
    /// Screen or attachment chips are present (the model gets them anyway).
    pub has_chips: bool,
    /// Clipboard text, read only for commands that use `{area}`.
    pub clipboard: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Expansion {
    /// Sent to the model.
    pub prompt_text: String,
    /// Shown in the conversation (compact, AC-010).
    pub display: String,
    /// The command wants a screen capture chip (e.g. `/resumir-tela`).
    pub needs_screen: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum QuickError {
    #[error("comando desconhecido: /{0}")]
    Unknown(String),
    #[error("o comando /{0} está desativado")]
    Disabled(String),
    #[error("selecione um texto, anexe algo ou digite o conteúdo depois do comando")]
    NoInput,
    #[error("a área de transferência está vazia: copie um texto primeiro")]
    EmptyClipboard,
    #[error("nome inválido: use letras minúsculas, números e hífens")]
    BadName,
    #[error("o modelo está vazio")]
    EmptyTemplate,
    #[error("não é possível alterar um comando embutido")]
    Builtin,
    #[error("já existe um comando /{0}")]
    Exists(String),
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for QuickError {
    fn from(e: StoreError) -> Self {
        QuickError::Store(e.to_string())
    }
}

/// Marker for commands that attach the screen.
const SCREEN_MARK: &str = "{tela}";
/// Marker for commands that read the clipboard.
pub const AREA_MARK: &str = "{area}";

pub fn builtins() -> Vec<QuickCommand> {
    let b = |name: &str, template: &str| QuickCommand {
        name: name.into(),
        template: template.into(),
        builtin: true,
        enabled: true,
    };
    vec![
        b(
            "tldr",
            "Resuma em até três frases, direto ao ponto:\n\n{selecao}",
        ),
        b("traduzir", "Traduza para {args:inglês}:\n\n{selecao}"),
        b(
            "reescrever",
            "Reescreva o texto com mais clareza, mantendo o sentido e o idioma:\n\n{selecao}",
        ),
        b(
            "explicar",
            "Explique de forma simples e objetiva:\n\n{selecao}",
        ),
        b(
            "corrigir",
            "Corrija ortografia e gramática sem mudar o estilo. Responda só com o texto corrigido:\n\n{selecao}",
        ),
        b(
            "resumir-tela",
            "{tela}Resuma o que está na tela e destaque o que parece mais importante.\n\n{texto}",
        ),
        b(
            "formal",
            "Reescreva em tom formal e profissional, mantendo o sentido e o idioma. Responda só com o texto:\n\n{selecao}",
        ),
        b(
            "curto",
            "Reescreva de forma mais curta e direta, sem perder o essencial. Responda só com o texto:\n\n{selecao}",
        ),
        b(
            "amigavel",
            "Reescreva em tom amigável e natural, mantendo o sentido e o idioma. Responda só com o texto:\n\n{selecao}",
        ),
        b(
            "golpe",
            "Analise se o conteúdo abaixo parece golpe, phishing ou fraude (WhatsApp, e-mail, SMS, link, boleto, Pix, falsa central). Responda em linguagem simples, nesta ordem: 1) Veredito: Parece golpe, Suspeito ou Parece seguro, com o motivo principal; 2) Sinais encontrados; 3) O que fazer agora; 4) Como confirmar com segurança pelo canal oficial. Não peça nem repita senhas, códigos ou dados pessoais. Se não der para ter certeza, diga isso.\n\n{selecao}",
        ),
        b(
            "responder",
            "{tela}Leia o e-mail ou a conversa que está na tela e escreva um rascunho de resposta no mesmo idioma, pronto para colar. Tom: {args:cordial}. Responda só com o texto da resposta.\n\n{texto}",
        ),
        b(
            "lembrar",
            "Crie um lembrete com as ferramentas clock_now e reminder_create. Pedido do usuário: {texto}",
        ),
        b(
            "anota",
            "Guarde esta nota com a ferramenta note_save, com as palavras do usuário e sem acrescentar nada, e confirme em uma linha: {texto}",
        ),
        b(
            "notas",
            "Procure nas minhas notas com a ferramenta note_search (consulta: {texto}) e liste o que achar, da mais recente para a mais antiga.",
        ),
        b(
            "colar",
            "Converta o texto abaixo para este formato: {args:lista com marcadores}. Responda só com o resultado, pronto para colar.\n\n{area}",
        ),
        b(
            "configurar",
            "$aura-configurar Quero configurar o Aura: {texto}",
        ),
        b("preparo", "$aura-preparo Quero me preparar para: {texto}"),
        b(
            "parei",
            "Use a ferramenta screen_recent para ver os últimos minutos da minha tela e diga, em poucas linhas, o que eu estava fazendo, em que ponto parei e qual seria o próximo passo. Se o buffer de tela estiver desligado, explique como ligá-lo em Configurações › Privacidade.\n\n{texto}",
        ),
    ]
}

/// Splits `/traduzir inglês resto` into (`traduzir`, `inglês resto`).
pub fn parse_invocation(input: &str) -> Option<(&str, &str)> {
    let rest = input.trim_start().strip_prefix('/')?;
    let (name, tail) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    (!name.is_empty()).then_some((name, tail.trim()))
}

fn uses_args(template: &str) -> bool {
    template.contains("{args}") || template.contains("{args:")
}

pub fn expand(cmd: &QuickCommand, tail: &str, ctx: &QuickContext) -> Result<Expansion, QuickError> {
    if !cmd.enabled {
        return Err(QuickError::Disabled(cmd.name.clone()));
    }
    let (arg, typed_after) = if uses_args(&cmd.template) {
        let (a, t) = tail.split_once(char::is_whitespace).unwrap_or((tail, ""));
        (a.trim(), t.trim())
    } else {
        ("", tail)
    };
    let typed = [typed_after, ctx.typed.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    let selection = ctx
        .selection
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let needs_screen = cmd.template.contains(SCREEN_MARK);

    let needs_clipboard = cmd.template.contains(AREA_MARK);
    let clipboard = ctx
        .clipboard
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if needs_clipboard && clipboard.is_none() {
        return Err(QuickError::EmptyClipboard);
    }
    let needs_input = cmd.template.contains("{selecao}");
    let input = selection
        .map(str::to_string)
        .or_else(|| (!typed.is_empty()).then(|| typed.clone()));
    if needs_input && input.is_none() && !ctx.has_chips && !needs_screen {
        return Err(QuickError::NoInput);
    }

    let mut out = cmd.template.replace(SCREEN_MARK, "");
    out = out.replace("{selecao}", input.as_deref().unwrap_or(""));
    out = out.replace(AREA_MARK, clipboard.unwrap_or(""));
    out = out.replace("{texto}", &typed);
    // {args:default}
    while let Some(start) = out.find("{args") {
        let Some(end) = out[start..].find('}').map(|e| start + e) else {
            break;
        };
        let default = out[start + 5..end]
            .strip_prefix(':')
            .unwrap_or("")
            .to_string();
        let value = if arg.is_empty() {
            default.as_str()
        } else {
            arg
        };
        out.replace_range(start..=end, value);
    }
    let prompt_text = out.trim().to_string();

    let display = match (arg.is_empty(), typed_after.is_empty()) {
        (true, true) => format!("/{}", cmd.name),
        (false, true) => format!("/{} {arg}", cmd.name),
        (true, false) => format!("/{} {typed_after}", cmd.name),
        (false, false) => format!("/{} {arg} {typed_after}", cmd.name),
    };
    Ok(Expansion {
        prompt_text,
        display,
        needs_screen,
    })
}

pub fn validate_name(name: &str) -> Result<(), QuickError> {
    let ok = !name.is_empty()
        && name.len() <= 40
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok { Ok(()) } else { Err(QuickError::BadName) }
}

/// SQLite-backed registry; built-ins are seeded and can only be toggled.
pub struct QuickCommandsRepo {
    store: Store,
}

impl QuickCommandsRepo {
    pub fn new(store: Store) -> Result<Self, QuickError> {
        store.with_conn(|c| {
            for b in builtins() {
                // Refresh built-in templates on upgrade, keep the user's toggle.
                c.execute(
                    "INSERT INTO quick_commands(name, template, builtin, enabled) VALUES (?1, ?2, 1, 1)
                     ON CONFLICT(name) DO UPDATE SET template = excluded.template WHERE builtin = 1",
                    params![b.name, b.template],
                )?;
            }
            Ok(())
        })?;
        Ok(Self { store })
    }

    pub fn list(&self) -> Result<Vec<QuickCommand>, QuickError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT name, template, builtin, enabled FROM quick_commands ORDER BY builtin DESC, name")?;
            let rows = st
                .query_map([], |r| Ok(QuickCommand { name: r.get(0)?, template: r.get(1)?, builtin: r.get::<_, i64>(2)? == 1, enabled: r.get::<_, i64>(3)? == 1 }))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    pub fn get(&self, name: &str) -> Result<QuickCommand, QuickError> {
        self.list()?
            .into_iter()
            .find(|c| c.name == name)
            .ok_or_else(|| QuickError::Unknown(name.into()))
    }

    pub fn save(&self, name: &str, template: &str, replace: bool) -> Result<(), QuickError> {
        validate_name(name)?;
        if template.trim().is_empty() {
            return Err(QuickError::EmptyTemplate);
        }
        match self.get(name) {
            Ok(c) if c.builtin => return Err(QuickError::Builtin),
            Ok(_) if !replace => return Err(QuickError::Exists(name.into())),
            _ => {}
        }
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO quick_commands(name, template, builtin, enabled) VALUES (?1, ?2, 0, 1)
                 ON CONFLICT(name) DO UPDATE SET template = excluded.template",
                params![name, template],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn set_enabled(&self, name: &str, enabled: bool) -> Result<(), QuickError> {
        let n = self.store.with_conn(|c| {
            Ok(c.execute(
                "UPDATE quick_commands SET enabled = ?2 WHERE name = ?1",
                params![name, enabled as i64],
            )?)
        })?;
        if n == 0 {
            Err(QuickError::Unknown(name.into()))
        } else {
            Ok(())
        }
    }

    pub fn delete(&self, name: &str) -> Result<(), QuickError> {
        if self.get(name)?.builtin {
            return Err(QuickError::Builtin);
        }
        self.store.with_conn(|c| {
            Ok(c.execute("DELETE FROM quick_commands WHERE name = ?1", params![name])?)
        })?;
        Ok(())
    }

    /// Resolves and expands `/name tail` typed in the input bar.
    pub fn run(&self, input: &str, ctx: &QuickContext) -> Result<Expansion, QuickError> {
        let (name, tail) =
            parse_invocation(input).ok_or_else(|| QuickError::Unknown(input.into()))?;
        expand(&self.get(name)?, tail, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(name: &str) -> QuickCommand {
        builtins().into_iter().find(|c| c.name == name).unwrap()
    }

    #[test]
    fn translate_with_selection_matches_the_plan_oracle() {
        let ctx = QuickContext {
            selection: Some("olá".into()),
            ..Default::default()
        };
        let e = expand(&cmd("traduzir"), "inglês", &ctx).unwrap();
        assert_eq!(e.prompt_text, "Traduza para inglês:\n\nolá");
        assert_eq!(e.display, "/traduzir inglês");
        let e = expand(&cmd("traduzir"), "", &ctx).unwrap();
        assert_eq!(e.prompt_text, "Traduza para inglês:\n\nolá");
        let e = expand(
            &cmd("traduzir"),
            "espanhol bom dia",
            &QuickContext::default(),
        )
        .unwrap();
        assert_eq!(e.prompt_text, "Traduza para espanhol:\n\nbom dia");
    }

    #[test]
    fn custom_template_uses_selection_then_typed_text() {
        let c = QuickCommand {
            name: "email-formal".into(),
            template: "Reescreva de forma formal: {selecao}".into(),
            builtin: false,
            enabled: true,
        };
        let with_sel = QuickContext {
            selection: Some("oi chefe".into()),
            typed: "ignorado".into(),
            has_chips: false,
            ..Default::default()
        };
        assert_eq!(
            expand(&c, "", &with_sel).unwrap().prompt_text,
            "Reescreva de forma formal: oi chefe"
        );
        let typed = QuickContext {
            typed: "manda o relatório".into(),
            ..Default::default()
        };
        assert_eq!(
            expand(&c, "", &typed).unwrap().prompt_text,
            "Reescreva de forma formal: manda o relatório"
        );
        assert_eq!(
            expand(&c, "", &QuickContext::default()),
            Err(QuickError::NoInput)
        );
        let chips = QuickContext {
            has_chips: true,
            ..Default::default()
        };
        assert_eq!(
            expand(&c, "", &chips).unwrap().prompt_text,
            "Reescreva de forma formal:"
        );
    }

    #[test]
    fn screen_summary_needs_screen_but_no_text() {
        let e = expand(&cmd("resumir-tela"), "", &QuickContext::default()).unwrap();
        assert!(e.needs_screen);
        assert!(e.prompt_text.starts_with("Resuma o que está na tela"));
        assert_eq!(
            parse_invocation("  /tldr   texto longo "),
            Some(("tldr", "texto longo"))
        );
        assert_eq!(parse_invocation("sem barra"), None);
    }

    #[test]
    fn everyday_commands_expand_with_the_selection() {
        let ctx = QuickContext {
            selection: Some("Seu Pix foi bloqueado, clique aqui".into()),
            ..Default::default()
        };
        let scam = expand(&cmd("golpe"), "", &ctx).unwrap();
        assert!(scam.prompt_text.contains("golpe"));
        assert!(
            scam.prompt_text
                .ends_with("Seu Pix foi bloqueado, clique aqui")
        );
        let formal = expand(&cmd("formal"), "", &ctx).unwrap();
        assert!(formal.prompt_text.starts_with("Reescreva em tom formal"));
        // No text anywhere: the rewrite commands ask for input.
        assert_eq!(
            expand(&cmd("curto"), "", &QuickContext::default()),
            Err(QuickError::NoInput)
        );
    }

    #[test]
    fn reply_attaches_the_screen_and_takes_a_tone() {
        let e = expand(&cmd("responder"), "informal", &QuickContext::default()).unwrap();
        assert!(e.needs_screen);
        assert!(e.prompt_text.contains("Tom: informal"));
        let d = expand(&cmd("responder"), "", &QuickContext::default()).unwrap();
        assert!(d.prompt_text.contains("Tom: cordial"));
    }

    #[test]
    fn resume_points_the_agent_at_the_recent_screen() {
        let e = expand(&cmd("parei"), "", &QuickContext::default()).unwrap();
        assert!(!e.needs_screen);
        assert!(e.prompt_text.contains("screen_recent"));
    }

    #[test]
    fn paste_as_reads_the_clipboard_only_when_asked() {
        let ctx = QuickContext {
            clipboard: Some("a, b e c\n".into()),
            ..Default::default()
        };
        let e = expand(&cmd("colar"), "tabela", &ctx).unwrap();
        assert!(e.prompt_text.contains("formato: tabela"));
        assert!(e.prompt_text.ends_with("a, b e c"));
        assert_eq!(
            expand(&cmd("colar"), "", &QuickContext::default()),
            Err(QuickError::EmptyClipboard)
        );
        // Commands without {area} never need or use the clipboard.
        let c = QuickContext {
            clipboard: Some("segredo".into()),
            selection: Some("texto".into()),
            ..Default::default()
        };
        assert!(
            !expand(&cmd("curto"), "", &c)
                .unwrap()
                .prompt_text
                .contains("segredo")
        );
    }

    #[test]
    fn repo_seeds_builtins_and_protects_them() {
        let repo = QuickCommandsRepo::new(Store::open_in_memory().unwrap()).unwrap();
        assert_eq!(repo.list().unwrap().len(), 18);
        assert_eq!(repo.save("tldr", "x", true), Err(QuickError::Builtin));
        assert_eq!(repo.delete("tldr"), Err(QuickError::Builtin));
        repo.save("email-formal", "Formal: {selecao}", false)
            .unwrap();
        assert_eq!(
            repo.save("email-formal", "y", false),
            Err(QuickError::Exists("email-formal".into()))
        );
        repo.set_enabled("tldr", false).unwrap();
        let ctx = QuickContext {
            typed: "a".into(),
            ..Default::default()
        };
        assert_eq!(
            repo.run("/tldr", &ctx),
            Err(QuickError::Disabled("tldr".into()))
        );
        assert_eq!(
            repo.run("/email-formal", &ctx).unwrap().prompt_text,
            "Formal: a"
        );
        assert_eq!(repo.save("Maiúscula", "x", false), Err(QuickError::BadName));
        repo.delete("email-formal").unwrap();
        assert!(matches!(
            repo.get("email-formal"),
            Err(QuickError::Unknown(_))
        ));
    }
}
