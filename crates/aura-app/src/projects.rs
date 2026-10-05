//! Projects (039/027): a name and instructions that group meetings. The agent
//! reads a meeting's project instructions with `meeting_get`, and searches can
//! be limited to one project.

use aura_store::{Store, StoreError, now_secs};
use rusqlite::params;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectError {
    #[error("informe o nome do projeto")]
    Empty,
    #[error("instruções longas demais (máx. 4000 caracteres)")]
    TooLong,
    #[error("projeto não encontrado")]
    NotFound,
    #[error("já existe um projeto com esse nome")]
    Exists,
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for ProjectError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub instructions: String,
}

#[derive(Clone)]
pub struct ProjectsRepo {
    store: Store,
}

impl ProjectsRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<Project>, ProjectError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare(
                "SELECT id, name, instructions FROM projects ORDER BY name COLLATE NOCASE",
            )?;
            let rows = st
                .query_map([], |r| {
                    Ok(Project {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        instructions: r.get(2)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    pub fn get(&self, id: &str) -> Result<Project, ProjectError> {
        self.list()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or(ProjectError::NotFound)
    }

    /// Creates a project, or updates the one with `id` (names stay unique).
    pub fn save(
        &self,
        id: Option<&str>,
        name: &str,
        instructions: &str,
    ) -> Result<Project, ProjectError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProjectError::Empty);
        }
        if instructions.chars().count() > 4000 {
            return Err(ProjectError::TooLong);
        }
        let all = self.list()?;
        if all
            .iter()
            .any(|p| p.name.to_lowercase() == name.to_lowercase() && Some(p.id.as_str()) != id)
        {
            return Err(ProjectError::Exists);
        }
        let id = match id {
            Some(i) if all.iter().any(|p| p.id == i) => i.to_string(),
            Some(_) => return Err(ProjectError::NotFound),
            None => uuid::Uuid::new_v4().to_string(),
        };
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO projects(id, name, instructions, updated_at) VALUES (?1,?2,?3,?4)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, instructions=excluded.instructions, updated_at=excluded.updated_at",
                params![id, name, instructions, now_secs()],
            )?;
            Ok(())
        })?;
        self.get(&id)
    }

    /// Meetings of the project are kept (they just lose the link).
    pub fn delete(&self, id: &str) -> Result<(), ProjectError> {
        let n = self
            .store
            .with_conn(|c| Ok(c.execute("DELETE FROM projects WHERE id = ?1", params![id])?))?;
        if n == 0 {
            Err(ProjectError::NotFound)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> ProjectsRepo {
        ProjectsRepo::new(Store::open_in_memory().unwrap())
    }

    #[test]
    fn projects_are_created_updated_and_names_stay_unique() {
        let r = repo();
        let a = r
            .save(
                None,
                "  Reforma da loja  ",
                "Tom informal; valores em reais",
            )
            .unwrap();
        assert_eq!(a.name, "Reforma da loja");
        assert_eq!(
            r.save(None, "reforma da LOJA", "").unwrap_err(),
            ProjectError::Exists
        );
        let b = r
            .save(Some(&a.id), "Reforma da loja", "Novo texto")
            .unwrap();
        assert_eq!(
            (b.id.as_str(), b.instructions.as_str()),
            (a.id.as_str(), "Novo texto")
        );
        r.save(None, "Contratação", "").unwrap();
        let names: Vec<_> = r.list().unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["Contratação", "Reforma da loja"]);
        assert_eq!(
            r.save(Some("nope"), "x", "").unwrap_err(),
            ProjectError::NotFound
        );
        assert_eq!(r.save(None, " ", "").unwrap_err(), ProjectError::Empty);
        assert_eq!(
            r.save(None, "y", &"x".repeat(4001)).unwrap_err(),
            ProjectError::TooLong
        );
        r.delete(&a.id).unwrap();
        assert_eq!(r.delete(&a.id), Err(ProjectError::NotFound));
    }
}
