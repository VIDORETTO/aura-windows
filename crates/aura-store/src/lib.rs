//! Local persistence for Aura: one SQLite database (`aura.db`, WAL) with
//! versioned migrations, typed repositories and an encrypted vault.
//!
//! Repositories that belong to a single feature crate (providers, ChatGPT
//! accounts, downloads...) use [`Store::with_conn`] from their own crate so
//! that this crate stays free of feature knowledge.

pub mod conversations;
pub mod placements;
pub mod protect;
pub mod settings_repo;
pub mod vault;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub use protect::{SecretProtector, StaticKeyProtector};
pub use vault::{Vault, VaultError};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, StoreError>;

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("migrations/0001_init.sql")),
    (
        "0002_conversations",
        include_str!("migrations/0002_conversations.sql"),
    ),
    (
        "0003_providers",
        include_str!("migrations/0003_providers.sql"),
    ),
    ("0004_capture", include_str!("migrations/0004_capture.sql")),
    ("0005_asr", include_str!("migrations/0005_asr.sql")),
    (
        "0006_chatgpt_accounts",
        include_str!("migrations/0006_chatgpt_accounts.sql"),
    ),
    (
        "0007_extensions",
        include_str!("migrations/0007_extensions.sql"),
    ),
    (
        "0008_productivity",
        include_str!("migrations/0008_productivity.sql"),
    ),
    (
        "0009_access_thumbs",
        include_str!("migrations/0009_access_thumbs.sql"),
    ),
    (
        "0010_reminders_notes",
        include_str!("migrations/0010_reminders_notes.sql"),
    ),
    (
        "0011_meetings",
        include_str!("migrations/0011_meetings.sql"),
    ),
    ("0012_recipes", include_str!("migrations/0012_recipes.sql")),
    ("0013_actions", include_str!("migrations/0013_actions.sql")),
    (
        "0014_projects",
        include_str!("migrations/0014_projects.sql"),
    ),
    (
        "0015_reminder_prompt",
        include_str!("migrations/0015_reminder_prompt.sql"),
    ),
];

/// Shared handle to the database. Cloning is cheap.
#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    /// Opens (or creates) the database and applies pending migrations.
    pub fn open(path: &Path) -> Result<Store> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// In-memory database for tests.
    pub fn open_in_memory() -> Result<Store> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Store> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrate(&conn)?;
        Ok(Store {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Runs `f` with exclusive access to the connection.
    pub fn with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut guard = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard)
    }

    pub fn schema_version(&self) -> Result<u32> {
        self.with_conn(|c| Ok(c.pragma_query_value(None, "user_version", |r| r.get(0))?))
    }
}

fn migrate(conn: &Connection) -> Result<()> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    for (index, (_name, sql)) in MIGRATIONS.iter().enumerate() {
        let version = index as u32 + 1;
        if version <= current {
            continue;
        }
        conn.execute_batch(&format!(
            "BEGIN;\n{sql}\nPRAGMA user_version = {version};\nCOMMIT;"
        ))?;
    }
    Ok(())
}

/// Unix time in seconds.
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_once_and_are_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("aura.db");
        let s = Store::open(&path).unwrap();
        assert_eq!(s.schema_version().unwrap(), MIGRATIONS.len() as u32);
        drop(s);
        let s = Store::open(&path).unwrap();
        assert_eq!(s.schema_version().unwrap(), MIGRATIONS.len() as u32);
    }
}
