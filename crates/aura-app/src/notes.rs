//! Quick notes and saved answers (020): "anota: renovar o seguro em março".

use aura_store::{Store, StoreError};
use rusqlite::params;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoteError {
    #[error("a nota está vazia")]
    Empty,
    #[error("nota longa demais (máx. 20000 caracteres)")]
    TooLong,
    #[error("tipo inválido: use note ou saved")]
    BadKind,
    #[error("nota não encontrada")]
    NotFound,
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for NoteError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub kind: String,
    pub text: String,
    pub created_at: i64,
}

#[derive(Clone)]
pub struct NotesRepo {
    store: Store,
}

fn kind_ok(kind: &str) -> Result<(), NoteError> {
    matches!(kind, "note" | "saved")
        .then_some(())
        .ok_or(NoteError::BadKind)
}

/// Lowercase without accents, so "março" matches "marco".
fn fold(s: &str) -> String {
    s.chars()
        .map(|c| match c.to_ascii_lowercase() {
            c if c.is_ascii() => c,
            _ => match c.to_lowercase().next().unwrap_or(c) {
                'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
                'é' | 'è' | 'ê' | 'ë' => 'e',
                'í' | 'ì' | 'î' | 'ï' => 'i',
                'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
                'ú' | 'ù' | 'û' | 'ü' => 'u',
                'ç' => 'c',
                other => other,
            },
        })
        .collect()
}

impl NotesRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn add(&self, kind: &str, text: &str, now: i64) -> Result<Note, NoteError> {
        kind_ok(kind)?;
        let text = text.trim();
        if text.is_empty() {
            return Err(NoteError::Empty);
        }
        if text.chars().count() > 20_000 {
            return Err(NoteError::TooLong);
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO notes(id, kind, text, created_at) VALUES (?1,?2,?3,?4)",
                params![id, kind, text, now],
            )?;
            Ok(())
        })?;
        Ok(Note {
            id,
            kind: kind.into(),
            text: text.into(),
            created_at: now,
        })
    }

    /// Newest first; every word of `query` must appear (accent-insensitive).
    pub fn search(&self, kind: &str, query: &str, limit: usize) -> Result<Vec<Note>, NoteError> {
        kind_ok(kind)?;
        let words: Vec<String> = fold(query).split_whitespace().map(str::to_string).collect();
        let all = self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT id, kind, text, created_at FROM notes WHERE kind = ?1 ORDER BY created_at DESC, rowid DESC")?;
            let rows = st
                .query_map(params![kind], |r| Ok(Note { id: r.get(0)?, kind: r.get(1)?, text: r.get(2)?, created_at: r.get(3)? }))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;
        Ok(all
            .into_iter()
            .filter(|n| {
                let t = fold(&n.text);
                words.iter().all(|w| t.contains(w))
            })
            .take(limit)
            .collect())
    }

    pub fn delete(&self, id: &str) -> Result<(), NoteError> {
        let n = self
            .store
            .with_conn(|c| Ok(c.execute("DELETE FROM notes WHERE id = ?1", params![id])?))?;
        if n == 0 {
            Err(NoteError::NotFound)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> NotesRepo {
        NotesRepo::new(Store::open_in_memory().unwrap())
    }

    #[test]
    fn notes_are_found_by_words_without_accents_newest_first() {
        let r = repo();
        r.add("note", "renovar o seguro em março", 10).unwrap();
        r.add("note", "comprar pão", 20).unwrap();
        r.add("note", "seguro do carro: ligar na corretora", 30)
            .unwrap();
        let hits = r.search("note", "seguro marco", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].text.contains("março"));
        let all = r.search("note", "seguro", 10).unwrap();
        assert_eq!(all[0].created_at, 30);
        assert_eq!(r.search("note", "", 1).unwrap().len(), 1);
    }

    #[test]
    fn kinds_are_separate_and_input_is_validated() {
        let r = repo();
        r.add("saved", "texto pronto", 1).unwrap();
        assert!(r.search("note", "", 10).unwrap().is_empty());
        assert_eq!(r.add("note", "   ", 1), Err(NoteError::Empty));
        assert_eq!(r.add("outro", "x", 1), Err(NoteError::BadKind));
        assert_eq!(
            r.add("note", &"x".repeat(20_001), 1),
            Err(NoteError::TooLong)
        );
        let n = r.add("note", "apagar", 2).unwrap();
        r.delete(&n.id).unwrap();
        assert_eq!(r.delete(&n.id), Err(NoteError::NotFound));
    }
}
