//! Non-secret account metadata (`chatgpt_accounts` table) and the host id.

use aura_store::{Result, Store, now_secs};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatGptAccount {
    pub client_id: String,
    pub subject: String,
    pub email: Option<String>,
    pub scopes: Vec<String>,
    pub plan_usage_enabled: bool,
    pub expires_at: Option<i64>,
    pub active: bool,
    /// Whether the "Você está usando seu plano ChatGPT" modal was shown.
    pub welcomed: bool,
    /// False after sign-out (registration kept, tokens cleared).
    pub signed_in: bool,
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ChatGptAccount> {
    let scopes: String = r.get(3)?;
    Ok(ChatGptAccount {
        client_id: r.get(0)?,
        subject: r.get(1)?,
        email: r.get(2)?,
        scopes: scopes.split_whitespace().map(str::to_string).collect(),
        plan_usage_enabled: r.get::<_, i64>(4)? != 0,
        expires_at: r.get(5)?,
        active: r.get::<_, i64>(6)? != 0,
        welcomed: r.get::<_, i64>(7)? != 0,
        signed_in: r.get::<_, Option<i64>>(5)?.is_some(),
    })
}

const COLS: &str =
    "client_id, subject, email, scopes, plan_usage_enabled, expires_at, active, welcomed";

pub struct AccountsRepo<'a> {
    store: &'a Store,
}

impl<'a> AccountsRepo<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<ChatGptAccount>> {
        self.store.with_conn(|c| {
            let mut stmt = c.prepare(&format!(
                "SELECT {COLS} FROM chatgpt_accounts ORDER BY created_at"
            ))?;
            let rows = stmt.query_map([], row)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })
    }

    pub fn get(&self, client_id: &str) -> Result<Option<ChatGptAccount>> {
        self.store.with_conn(|c| {
            Ok(c.query_row(
                &format!("SELECT {COLS} FROM chatgpt_accounts WHERE client_id = ?1"),
                [client_id],
                row,
            )
            .optional()?)
        })
    }

    pub fn active(&self) -> Result<Option<ChatGptAccount>> {
        Ok(self.list()?.into_iter().find(|a| a.active))
    }

    pub fn upsert(&self, a: &ChatGptAccount) -> Result<()> {
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO chatgpt_accounts(client_id, subject, email, scopes, plan_usage_enabled, expires_at, active, welcomed, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                 ON CONFLICT(client_id) DO UPDATE SET subject=excluded.subject, email=excluded.email, scopes=excluded.scopes,
                   plan_usage_enabled=excluded.plan_usage_enabled, expires_at=excluded.expires_at,
                   active=excluded.active, welcomed=excluded.welcomed",
                params![
                    a.client_id,
                    a.subject,
                    a.email,
                    a.scopes.join(" "),
                    a.plan_usage_enabled as i64,
                    if a.signed_in { a.expires_at.or(Some(0)) } else { None },
                    a.active as i64,
                    a.welcomed as i64,
                    now_secs()
                ],
            )?;
            Ok(())
        })
    }

    pub fn set_active(&self, client_id: &str) -> Result<()> {
        self.store.with_conn(|c| {
            let tx = c.transaction()?;
            tx.execute("UPDATE chatgpt_accounts SET active = 0", [])?;
            tx.execute(
                "UPDATE chatgpt_accounts SET active = 1 WHERE client_id = ?1",
                [client_id],
            )?;
            tx.commit()?;
            Ok(())
        })
    }
}

pub const HOST_ID_KEY: &str = "siwc.extAgentHostId";

/// Stable per-installation host id (`urn:uuid:<v4>`, a format accepted by
/// SIWC). Created before the first sign-in and reused forever.
pub fn host_id(store: &Store) -> Result<String> {
    let repo = aura_store::settings_repo::SettingsRepo::new(store);
    if let Some(v) = repo
        .get_raw(HOST_ID_KEY)?
        .and_then(|v| v.as_str().map(str::to_string))
    {
        return Ok(v);
    }
    let id = format!("urn:uuid:{}", uuid::Uuid::new_v4());
    repo.set_raw(HOST_ID_KEY, &serde_json::Value::String(id.clone()))?;
    Ok(id)
}
