//! Persists [`aura_core::settings::Settings`] key by key (AC-014, 001).

use crate::{Result, Store, now_secs};
use aura_core::settings::Settings;
use rusqlite::params;
use serde_json::{Map, Value};

pub struct SettingsRepo<'a> {
    store: &'a Store,
}

impl<'a> SettingsRepo<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    /// Loads settings; missing keys use defaults, unknown keys are preserved
    /// in the database but ignored.
    pub fn load(&self) -> Result<Settings> {
        let map = self.store.with_conn(|c| {
            let mut stmt = c.prepare("SELECT key, value_json FROM settings")?;
            let rows =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            let mut map = Map::new();
            for row in rows {
                let (k, v) = row?;
                if let Ok(value) = serde_json::from_str::<Value>(&v) {
                    map.insert(k, value);
                }
            }
            Ok(map)
        })?;
        Ok(serde_json::from_value(Value::Object(map)).unwrap_or_default())
    }

    /// Saves every field of `settings` in one transaction.
    pub fn save(&self, settings: &Settings) -> Result<()> {
        let Value::Object(map) = serde_json::to_value(settings)? else {
            return Ok(());
        };
        self.store.with_conn(|c| {
            let tx = c.transaction()?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO settings(key, value_json, updated_at) VALUES (?1, ?2, ?3)
                     ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
                )?;
                let now = now_secs();
                for (k, v) in map {
                    stmt.execute(params![k, v.to_string(), now])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    /// Generic JSON value storage for feature flags such as onboarding state.
    pub fn get_raw(&self, key: &str) -> Result<Option<Value>> {
        self.store.with_conn(|c| {
            let v: Option<String> = c
                .query_row(
                    "SELECT value_json FROM settings WHERE key = ?1",
                    [key],
                    |r| r.get(0),
                )
                .map(Some)
                .or_else(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => Ok(None),
                    e => Err(e),
                })?;
            Ok(v.and_then(|s| serde_json::from_str(&s).ok()))
        })
    }

    pub fn set_raw(&self, key: &str, value: &Value) -> Result<()> {
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO settings(key, value_json, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
                params![key, value.to_string(), now_secs()],
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_core::settings::{FocusLossBehavior, SettingsPatch, Theme};

    #[test]
    fn saved_settings_survive_reopening_the_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aura.db");
        {
            let store = Store::open(&path).unwrap();
            let s = Settings::default()
                .apply(&SettingsPatch {
                    theme: Some(Theme::Dark),
                    focus_loss: Some(FocusLossBehavior::KeepOpen),
                    opacity: Some(0.8),
                    ..Default::default()
                })
                .unwrap();
            SettingsRepo::new(&store).save(&s).unwrap();
        }
        let store = Store::open(&path).unwrap();
        let loaded = SettingsRepo::new(&store).load().unwrap();
        assert_eq!(loaded.theme, Theme::Dark);
        assert_eq!(loaded.focus_loss, FocusLossBehavior::KeepOpen);
        assert_eq!(loaded.opacity, 0.8);
    }

    #[test]
    fn empty_database_yields_defaults() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(
            SettingsRepo::new(&store).load().unwrap(),
            Settings::default()
        );
    }

    #[test]
    fn raw_values_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let repo = SettingsRepo::new(&store);
        assert_eq!(repo.get_raw("onboarding").unwrap(), None);
        repo.set_raw("onboarding", &serde_json::json!({"step": 3}))
            .unwrap();
        assert_eq!(
            repo.get_raw("onboarding").unwrap(),
            Some(serde_json::json!({"step": 3}))
        );
    }
}
