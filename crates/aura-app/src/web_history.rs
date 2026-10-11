//! Citation metadata only. Page bodies and search snippets are never written here.
use aura_store::{Result, Store};
use aura_web::WebSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptMessage {
    pub role: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<WebSource>,
}

pub(crate) struct WebHistory(Store);

#[cfg(test)]
#[path = "web_history_tests.rs"]
mod tests;

impl WebHistory {
    pub fn new(store: Store) -> Result<Self> {
        store.with_conn(|connection| {
            connection.execute_batch(
                "CREATE TABLE IF NOT EXISTS web_citations (
                thread_id TEXT NOT NULL REFERENCES conversations_meta(thread_id) ON DELETE CASCADE,
                turn_id TEXT NOT NULL, item_id TEXT NOT NULL, sources_json TEXT NOT NULL,
                PRIMARY KEY(thread_id, turn_id, item_id)
            );",
            )?;
            connection.execute_batch("CREATE TABLE IF NOT EXISTS web_source_sequence (
                thread_id TEXT PRIMARY KEY REFERENCES conversations_meta(thread_id) ON DELETE CASCADE,
                last_id INTEGER NOT NULL
            );")?;
            Ok(())
        })?;
        Ok(Self(store))
    }
    pub fn record(
        &self,
        thread: &str,
        turn: &str,
        item: &str,
        mut sources: Vec<WebSource>,
    ) -> Result<()> {
        if sources.is_empty() {
            return Ok(());
        }
        for source in &mut sources {
            source.snippet.clear();
        }
        let json = serde_json::to_string(&sources)?;
        self.0.with_conn(|connection| {
            // Ephemeral threads have no durable metadata. Do not create a row,
            // even if a completed message races with closing that thread.
            connection.execute("INSERT INTO web_citations(thread_id,turn_id,item_id,sources_json)
                SELECT ?1,?2,?3,?4 WHERE EXISTS(SELECT 1 FROM conversations_meta WHERE thread_id=?1)
                ON CONFLICT(thread_id,turn_id,item_id) DO UPDATE SET sources_json=excluded.sources_json",
                [thread, turn, item, &json])?;
            Ok(())
        })
    }
    pub fn sources(&self, thread: &str, turn: &str, item: &str) -> Result<Vec<WebSource>> {
        self.0.with_conn(|connection| {
            let mut statement = connection.prepare(
                "SELECT sources_json FROM web_citations
                WHERE thread_id=?1 AND turn_id=?2 AND item_id=?3",
            )?;
            let mut rows = statement.query([thread, turn, item])?;
            match rows.next()? {
                Some(row) => Ok(serde_json::from_str(&row.get::<_, String>(0)?)?),
                None => Ok(vec![]),
            }
        })
    }
    /// Reserve identifiers without retaining an uncited URL, title or snippet.
    pub fn observe_id(&self, thread: &str, source_id: &str) -> Result<()> {
        let Some(number) = source_id
            .strip_prefix('W')
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|number| *number > 0 && *number < i64::MAX)
        else {
            return Ok(());
        };
        self.0.with_conn(|connection| {
            connection.execute(
                "INSERT INTO web_source_sequence(thread_id,last_id)
                SELECT ?1,?2 WHERE EXISTS(SELECT 1 FROM conversations_meta WHERE thread_id=?1)
                ON CONFLICT(thread_id) DO UPDATE SET last_id=MAX(last_id,excluded.last_id)",
                (thread, number),
            )?;
            Ok(())
        })
    }
    /// Release the database lock before applying a bounded row of metadata.
    /// Avoid accumulating the entire conversation history in a second vector.
    pub fn restore(
        &self,
        thread: &str,
        mut apply: impl FnMut(usize, Vec<WebSource>) -> bool,
    ) -> Result<()> {
        let last = self.0.with_conn(|connection| {
            let mut statement =
                connection.prepare("SELECT last_id FROM web_source_sequence WHERE thread_id=?1")?;
            let mut rows = statement.query([thread])?;
            Ok(match rows.next()? {
                Some(row) => row.get::<_, i64>(0)?,
                None => 0,
            })
        })?;
        if !apply(last as usize, vec![]) {
            return Ok(());
        }
        let mut cursor = 0_i64;
        loop {
            let row = self.0.with_conn(|connection| {
                let mut statement = connection.prepare(
                    "SELECT rowid,sources_json FROM web_citations
                    WHERE thread_id=?1 AND rowid>?2 ORDER BY rowid LIMIT 1",
                )?;
                let mut rows = statement.query((thread, cursor))?;
                Ok(match rows.next()? {
                    Some(row) => Some((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
                    None => None,
                })
            })?;
            let Some((next, json)) = row else {
                break;
            };
            cursor = next;
            if !apply(last as usize, serde_json::from_str(&json)?) {
                break;
            }
        }
        Ok(())
    }
}
