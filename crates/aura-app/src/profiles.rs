//! App profiles (009 TK-004): instructions, default mode/model and "attach
//! the screen on open" per application (`code.exe`, `*chrome*`…).

use aura_core::events::PreviousApp;
use aura_policy::wildcard::matches;
use aura_store::{Store, StoreError, now_secs};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppProfile {
    #[serde(default)]
    pub id: String,
    pub name: String,
    /// Executable pattern, case-insensitive wildcards (`code.exe`, `*chrome*`).
    pub process_pattern: String,
    #[serde(default)]
    pub title_glob: Option<String>,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub attach_screen: bool,
    /// `chat`, `task` or `plan`.
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

impl AppProfile {
    pub fn applies_to(&self, app: &PreviousApp) -> bool {
        matches(&self.process_pattern, &app.process_name)
            && self
                .title_glob
                .as_deref()
                .is_none_or(|g| g.trim().is_empty() || matches(g, &app.title))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("informe um nome e o processo do app")]
    Invalid,
    #[error("instruções longas demais (máx. 4000 caracteres)")]
    TooLong,
    #[error(transparent)]
    Store(#[from] StoreError),
}

#[derive(Clone)]
pub struct ProfilesRepo {
    store: Store,
}

impl ProfilesRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<AppProfile>, ProfileError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare(
                "SELECT id, name, process_pattern, title_glob, instructions, attach_screen, default_mode, default_model
                 FROM app_profiles ORDER BY name COLLATE NOCASE",
            )?;
            let rows = st
                .query_map([], |r| {
                    Ok(AppProfile {
                        id: r.get(0)?,
                        name: r.get(1)?,
                        process_pattern: r.get(2)?,
                        title_glob: r.get(3)?,
                        instructions: r.get(4)?,
                        attach_screen: r.get::<_, i64>(5)? == 1,
                        default_mode: r.get(6)?,
                        default_model: r.get(7)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    pub fn save(&self, mut p: AppProfile) -> Result<AppProfile, ProfileError> {
        if p.name.trim().is_empty() || p.process_pattern.trim().is_empty() {
            return Err(ProfileError::Invalid);
        }
        if p.instructions.chars().count() > 4000 {
            return Err(ProfileError::TooLong);
        }
        if p.id.is_empty() {
            p.id = uuid::Uuid::new_v4().simple().to_string();
        }
        p.default_mode = p
            .default_mode
            .filter(|m| ["chat", "task", "plan"].contains(&m.as_str()));
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO app_profiles(id, name, process_pattern, title_glob, instructions, attach_screen, default_mode, default_model, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, process_pattern=excluded.process_pattern,
                   title_glob=excluded.title_glob, instructions=excluded.instructions, attach_screen=excluded.attach_screen,
                   default_mode=excluded.default_mode, default_model=excluded.default_model, updated_at=excluded.updated_at",
                params![
                    p.id,
                    p.name.trim(),
                    p.process_pattern.trim(),
                    p.title_glob,
                    p.instructions,
                    p.attach_screen as i64,
                    p.default_mode,
                    p.default_model,
                    now_secs()
                ],
            )?;
            Ok(())
        })?;
        Ok(p)
    }

    pub fn delete(&self, id: &str) -> Result<(), ProfileError> {
        self.store.with_conn(|c| {
            Ok(
                c.execute("DELETE FROM app_profiles WHERE id = ?1", params![id])
                    .map(|_| ())?,
            )
        })?;
        Ok(())
    }

    /// First profile matching the app; title-specific profiles win.
    pub fn for_app(&self, app: &PreviousApp) -> Option<AppProfile> {
        let mut list: Vec<AppProfile> = self
            .list()
            .ok()?
            .into_iter()
            .filter(|p| p.applies_to(app))
            .collect();
        list.sort_by_key(|p| p.title_glob.as_deref().is_none_or(|g| g.trim().is_empty()));
        list.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(process: &str, title: &str) -> PreviousApp {
        PreviousApp {
            window: 1,
            pid: 1,
            process_name: process.into(),
            title: title.into(),
            monitor_id: String::new(),
        }
    }

    #[test]
    fn matching_and_precedence() {
        let repo = ProfilesRepo::new(Store::open_in_memory().unwrap());
        let code = repo
            .save(AppProfile {
                id: String::new(),
                name: "VS Code".into(),
                process_pattern: "code.exe".into(),
                title_glob: None,
                instructions: "Responda com código TypeScript".into(),
                attach_screen: true,
                default_mode: Some("task".into()),
                default_model: None,
            })
            .unwrap();
        repo.save(AppProfile {
            id: String::new(),
            name: "Aura repo".into(),
            title_glob: Some("*aura*".into()),
            ..code.clone()
        })
        .unwrap();
        assert_eq!(
            repo.for_app(&app("Code.exe", "main.rs — outro"))
                .unwrap()
                .name,
            "VS Code"
        );
        assert_eq!(
            repo.for_app(&app("Code.exe", "main.rs — aura"))
                .unwrap()
                .name,
            "Aura repo"
        );
        assert!(repo.for_app(&app("chrome.exe", "x")).is_none());
        assert!(matches!(
            repo.save(AppProfile {
                name: " ".into(),
                ..code.clone()
            }),
            Err(ProfileError::Invalid)
        ));
        let bad_mode = repo
            .save(AppProfile {
                id: String::new(),
                default_mode: Some("x".into()),
                ..code.clone()
            })
            .unwrap();
        assert_eq!(bad_mode.default_mode, None);
        repo.delete(&code.id).unwrap();
        assert_eq!(repo.list().unwrap().len(), 2);
    }
}
