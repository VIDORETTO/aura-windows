//! Aura-side metadata for Codex threads (workspace, mode, provider) and the
//! queue of folders that could not be deleted yet (AC-016, 002).

use crate::{Result, Store, now_secs};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMeta {
    pub thread_id: String,
    /// Aura-generated id sent to the MCP server in `X-Aura-Conversation`.
    pub conversation_uuid: String,
    pub workspace_path: PathBuf,
    /// `chat` or `task`.
    pub mode: String,
    pub provider_id: String,
    pub granted_folders: Vec<PathBuf>,
    pub extra_instructions: String,
}

pub struct ConversationsRepo<'a> {
    store: &'a Store,
}

impl<'a> ConversationsRepo<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn upsert(&self, m: &ConversationMeta) -> Result<()> {
        let folders = serde_json::to_string(&m.granted_folders)?;
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO conversations_meta(thread_id, conversation_uuid, workspace_path, mode, provider_id, granted_folders_json, extra_instructions, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
                 ON CONFLICT(thread_id) DO UPDATE SET workspace_path=excluded.workspace_path, mode=excluded.mode,
                   provider_id=excluded.provider_id, granted_folders_json=excluded.granted_folders_json,
                   extra_instructions=excluded.extra_instructions",
                params![
                    m.thread_id,
                    m.conversation_uuid,
                    m.workspace_path.to_string_lossy(),
                    m.mode,
                    m.provider_id,
                    folders,
                    m.extra_instructions,
                    now_secs()
                ],
            )?;
            Ok(())
        })
    }

    pub fn get(&self, thread_id: &str) -> Result<Option<ConversationMeta>> {
        self.query_one("thread_id", thread_id)
    }

    pub fn get_by_uuid(&self, uuid: &str) -> Result<Option<ConversationMeta>> {
        self.query_one("conversation_uuid", uuid)
    }

    fn query_one(&self, column: &str, value: &str) -> Result<Option<ConversationMeta>> {
        let sql = format!(
            "SELECT thread_id, conversation_uuid, workspace_path, mode, provider_id, granted_folders_json, extra_instructions
             FROM conversations_meta WHERE {column} = ?1"
        );
        self.store.with_conn(|c| {
            let row = c
                .query_row(&sql, [value], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, String>(6)?,
                    ))
                })
                .optional()?;
            let Some((thread_id, conversation_uuid, ws, mode, provider_id, folders, extra)) = row
            else {
                return Ok(None);
            };
            Ok(Some(ConversationMeta {
                thread_id,
                conversation_uuid,
                workspace_path: PathBuf::from(ws),
                mode,
                provider_id,
                granted_folders: serde_json::from_str(&folders).unwrap_or_default(),
                extra_instructions: extra,
            }))
        })
    }

    pub fn remove(&self, thread_id: &str) -> Result<()> {
        self.store.with_conn(|c| {
            c.execute(
                "DELETE FROM conversations_meta WHERE thread_id = ?1",
                [thread_id],
            )?;
            Ok(())
        })
    }

    pub fn queue_deletion(&self, path: &std::path::Path) -> Result<()> {
        self.store.with_conn(|c| {
            c.execute(
                "INSERT OR IGNORE INTO pending_deletions(path, queued_at) VALUES (?1, ?2)",
                params![path.to_string_lossy(), now_secs()],
            )?;
            Ok(())
        })
    }

    pub fn pending_deletions(&self) -> Result<Vec<PathBuf>> {
        self.store.with_conn(|c| {
            let mut stmt = c.prepare("SELECT path FROM pending_deletions ORDER BY queued_at")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            Ok(rows.filter_map(|r| r.ok()).map(PathBuf::from).collect())
        })
    }

    pub fn clear_deletion(&self, path: &std::path::Path) -> Result<()> {
        self.store.with_conn(|c| {
            c.execute(
                "DELETE FROM pending_deletions WHERE path = ?1",
                [path.to_string_lossy()],
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> ConversationMeta {
        ConversationMeta {
            thread_id: "thr_1".into(),
            conversation_uuid: "u1".into(),
            workspace_path: "/w/thr_1".into(),
            mode: "chat".into(),
            provider_id: "chatgpt-plan".into(),
            granted_folders: vec!["/docs".into()],
            extra_instructions: "Foque em Excel".into(),
        }
    }

    #[test]
    fn upsert_get_remove() {
        let store = Store::open_in_memory().unwrap();
        let repo = ConversationsRepo::new(&store);
        repo.upsert(&meta()).unwrap();
        assert_eq!(repo.get("thr_1").unwrap(), Some(meta()));
        assert_eq!(repo.get_by_uuid("u1").unwrap(), Some(meta()));
        let mut m = meta();
        m.mode = "task".into();
        repo.upsert(&m).unwrap();
        assert_eq!(repo.get("thr_1").unwrap().unwrap().mode, "task");
        repo.remove("thr_1").unwrap();
        assert_eq!(repo.get("thr_1").unwrap(), None);
    }

    #[test]
    fn pending_deletions_queue() {
        let store = Store::open_in_memory().unwrap();
        let repo = ConversationsRepo::new(&store);
        repo.queue_deletion(std::path::Path::new("/w/a")).unwrap();
        repo.queue_deletion(std::path::Path::new("/w/a")).unwrap();
        assert_eq!(
            repo.pending_deletions().unwrap(),
            vec![PathBuf::from("/w/a")]
        );
        repo.clear_deletion(std::path::Path::new("/w/a")).unwrap();
        assert!(repo.pending_deletions().unwrap().is_empty());
    }
}
