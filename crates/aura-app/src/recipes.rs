//! Recipes (025): a reusable shape for a kind of meeting — how the notes are
//! organized and how much help the user wants while it runs. Eight come with
//! Aura; the user (or the agent, with approval) adds more.

use aura_store::{Store, StoreError, now_secs};
use rusqlite::params;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    /// How the agent organizes the notes of this kind of meeting.
    pub notes_template: String,
    /// `silent`, `onDemand`, `balanced` or `active`.
    pub help_level: String,
    pub builtin: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecipeError {
    #[error("nome inválido: use letras minúsculas, números e hífens (máx. 40)")]
    BadId,
    #[error("informe o nome e o modelo de notas")]
    Empty,
    #[error("nível de ajuda inválido: silent, onDemand, balanced ou active")]
    BadHelp,
    #[error("não é possível alterar uma Receita embutida")]
    Builtin,
    #[error("já existe a Receita {0}")]
    Exists(String),
    #[error("Receita não encontrada")]
    NotFound,
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for RecipeError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

fn b(id: &str, name: &str, description: &str, template: &str, help: &str) -> Recipe {
    Recipe {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        notes_template: template.into(),
        help_level: help.into(),
        builtin: true,
    }
}

/// The eight Recipes that ship with Aura.
pub fn builtins() -> Vec<Recipe> {
    vec![
        b(
            "um-a-um",
            "1:1",
            "Conversa individual com chefe, par ou liderado.",
            "Como foi (humor e temas) · Feedback dado e recebido · Compromissos de cada lado (dono e prazo) · Assuntos para a próxima 1:1",
            "onDemand",
        ),
        b(
            "daily",
            "Stand-up / Daily",
            "Alinhamento rápido do time.",
            "Por pessoa: ontem, hoje, bloqueios · Bloqueios que precisam de ajuda (dono) · Decisões rápidas",
            "silent",
        ),
        b(
            "cliente",
            "Cliente / Discovery",
            "Conversa para entender a necessidade de um cliente.",
            "Contexto e dor do cliente · Objetivos e métricas · Restrições (prazo, orçamento, decisores) · Objeções · Próximos passos combinados",
            "onDemand",
        ),
        b(
            "vendas",
            "Chamada de vendas",
            "Chamada comercial com proposta ou negociação.",
            "Necessidade · Orçamento e prazo · Decisores · Objeções e respostas · Compromissos de cada lado · Próximo passo e data",
            "balanced",
        ),
        b(
            "entrevistado",
            "Entrevista (como entrevistado)",
            "Você está sendo entrevistado.",
            "Perguntas feitas e como respondi · O que o entrevistador valorizou · Pontos a reforçar ou corrigir · Próximos passos do processo",
            "onDemand",
        ),
        b(
            "entrevistador",
            "Entrevista (como entrevistador)",
            "Você conduz a entrevista.",
            "Perguntas feitas · Evidências por competência · Pontos fortes e dúvidas · Recomendação e justificativa",
            "silent",
        ),
        b(
            "decisao",
            "Reunião de decisão",
            "Reunião para escolher entre opções.",
            "Decisão a tomar · Opções e argumentos · Decisão (ou o que falta para decidir) · Responsáveis e prazos · Riscos",
            "balanced",
        ),
        b(
            "aula",
            "Aula / Treinamento",
            "Conteúdo para aprender.",
            "Resumo por tópico · Conceitos e definições · Exemplos · Dúvidas para pesquisar · Revisão em 5 perguntas",
            "silent",
        ),
    ]
}

const HELP: [&str; 4] = ["silent", "onDemand", "balanced", "active"];

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[derive(Clone)]
pub struct RecipesRepo {
    store: Store,
}

impl RecipesRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Built-ins first, then the user's, by name.
    pub fn list(&self) -> Result<Vec<Recipe>, RecipeError> {
        let mut all = builtins();
        let user = self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT id, name, description, notes_template, help_level FROM recipes ORDER BY name")?;
            let rows = st
                .query_map([], |r| {
                    Ok(Recipe { id: r.get(0)?, name: r.get(1)?, description: r.get(2)?, notes_template: r.get(3)?, help_level: r.get(4)?, builtin: false })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;
        all.extend(user);
        Ok(all)
    }

    pub fn get(&self, id: &str) -> Result<Recipe, RecipeError> {
        self.list()?
            .into_iter()
            .find(|r| r.id == id)
            .ok_or(RecipeError::NotFound)
    }

    pub fn save(&self, recipe: &Recipe, replace: bool) -> Result<(), RecipeError> {
        if !valid_id(&recipe.id) {
            return Err(RecipeError::BadId);
        }
        if recipe.name.trim().is_empty() || recipe.notes_template.trim().is_empty() {
            return Err(RecipeError::Empty);
        }
        if !HELP.contains(&recipe.help_level.as_str()) {
            return Err(RecipeError::BadHelp);
        }
        match self.get(&recipe.id) {
            Ok(r) if r.builtin => return Err(RecipeError::Builtin),
            Ok(_) if !replace => return Err(RecipeError::Exists(recipe.id.clone())),
            _ => {}
        }
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO recipes(id, name, description, notes_template, help_level, updated_at) VALUES (?1,?2,?3,?4,?5,?6)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, description=excluded.description, notes_template=excluded.notes_template, help_level=excluded.help_level, updated_at=excluded.updated_at",
                params![recipe.id, recipe.name.trim(), recipe.description, recipe.notes_template.trim(), recipe.help_level, now_secs()],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), RecipeError> {
        if builtins().iter().any(|r| r.id == id) {
            return Err(RecipeError::Builtin);
        }
        let n = self
            .store
            .with_conn(|c| Ok(c.execute("DELETE FROM recipes WHERE id = ?1", params![id])?))?;
        if n == 0 {
            Err(RecipeError::NotFound)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> RecipesRepo {
        RecipesRepo::new(Store::open_in_memory().unwrap())
    }

    fn mine(id: &str) -> Recipe {
        Recipe {
            id: id.into(),
            name: "Fornecedores".into(),
            description: "Reuniões com fornecedores".into(),
            notes_template: "Preço · Prazo · Condições".into(),
            help_level: "onDemand".into(),
            builtin: false,
        }
    }

    #[test]
    fn eight_builtins_with_valid_ids_and_help_levels() {
        let all = builtins();
        assert_eq!(all.len(), 8);
        for r in &all {
            assert!(valid_id(&r.id), "{}", r.id);
            assert!(HELP.contains(&r.help_level.as_str()), "{}", r.id);
            assert!(r.notes_template.contains('·'), "{} has no template", r.id);
        }
        let mut ids: Vec<_> = all.iter().map(|r| r.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn user_recipes_are_saved_replaced_and_deleted_but_builtins_are_protected() {
        let r = repo();
        r.save(&mine("fornecedores"), false).unwrap();
        assert_eq!(r.list().unwrap().len(), 9);
        assert_eq!(
            r.save(&mine("fornecedores"), false),
            Err(RecipeError::Exists("fornecedores".into()))
        );
        let mut changed = mine("fornecedores");
        changed.notes_template = "Preço · Prazo".into();
        r.save(&changed, true).unwrap();
        assert_eq!(
            r.get("fornecedores").unwrap().notes_template,
            "Preço · Prazo"
        );
        assert_eq!(r.save(&mine("daily"), true), Err(RecipeError::Builtin));
        assert_eq!(r.delete("daily"), Err(RecipeError::Builtin));
        r.delete("fornecedores").unwrap();
        assert_eq!(r.delete("fornecedores"), Err(RecipeError::NotFound));
        assert_eq!(r.get("fornecedores"), Err(RecipeError::NotFound));
    }

    #[test]
    fn invalid_recipes_are_explained() {
        let r = repo();
        assert_eq!(r.save(&mine("Com Espaço"), false), Err(RecipeError::BadId));
        let mut empty = mine("x");
        empty.notes_template = "  ".into();
        assert_eq!(r.save(&empty, false), Err(RecipeError::Empty));
        let mut help = mine("y");
        help.help_level = "loud".into();
        assert_eq!(r.save(&help, false), Err(RecipeError::BadHelp));
    }
}
