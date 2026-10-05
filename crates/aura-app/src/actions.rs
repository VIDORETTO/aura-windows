//! Commitments (047): actions and promises taken from meetings, with owner,
//! due date and the minute they came from. "Mine" are what the user owes;
//! "theirs" are what others promised the user. Open ones age and show as
//! overdue once past their due date.

use aura_store::{Store, StoreError};
use rusqlite::params;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ActionError {
    #[error("informe o texto do compromisso")]
    Empty,
    #[error("dono inválido: use \"you\" (eu) ou \"them\" (outra pessoa)")]
    BadOwner,
    #[error("data inválida: use AAAA-MM-DD")]
    BadDue,
    #[error("compromisso não encontrado")]
    NotFound,
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for ActionError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub meeting_id: Option<String>,
    pub text: String,
    /// `you` (the user owes it) or `them` (someone promised the user).
    pub owner: String,
    /// `YYYY-MM-DD`.
    pub due: Option<String>,
    pub status: String,
    /// Minute of the meeting it came from (ms since it started).
    pub t0: Option<i64>,
    pub created_at: i64,
}

/// `YYYY-MM-DD` is valid (calendar-checked).
pub fn valid_date(s: &str) -> bool {
    let mut p = s.split('-');
    let (Some(y), Some(m), Some(d), None) = (p.next(), p.next(), p.next(), p.next()) else {
        return false;
    };
    let (Ok(y), Ok(m), Ok(d)) = (y.parse::<i32>(), m.parse::<u8>(), d.parse::<u8>()) else {
        return false;
    };
    s.len() == 10
        && time::Month::try_from(m)
            .ok()
            .and_then(|m| time::Date::from_calendar_date(y, m, d).ok())
            .is_some()
}

impl Action {
    /// Open and past its due date (`today` as `YYYY-MM-DD`; ISO dates sort as text).
    pub fn overdue(&self, today: &str) -> bool {
        self.status == "open" && self.due.as_deref().is_some_and(|d| d < today)
    }
}

#[derive(Clone)]
pub struct ActionsRepo {
    store: Store,
}

fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Action> {
    Ok(Action {
        id: r.get(0)?,
        meeting_id: r.get(1)?,
        text: r.get(2)?,
        owner: r.get(3)?,
        due: r.get(4)?,
        status: r.get(5)?,
        t0: r.get(6)?,
        created_at: r.get(7)?,
    })
}

const COLS: &str = "id, meeting_id, text, owner, due, status, t0, created_at";

impl ActionsRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn add(
        &self,
        meeting_id: Option<&str>,
        text: &str,
        owner: &str,
        due: Option<&str>,
        t0: Option<i64>,
        now: i64,
    ) -> Result<Action, ActionError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ActionError::Empty);
        }
        if !matches!(owner, "you" | "them") {
            return Err(ActionError::BadOwner);
        }
        let due = due.map(str::trim).filter(|d| !d.is_empty());
        if due.is_some_and(|d| !valid_date(d)) {
            return Err(ActionError::BadDue);
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO meeting_actions(id, meeting_id, text, owner, due, status, t0, created_at) VALUES (?1,?2,?3,?4,?5,'open',?6,?7)",
                params![id, meeting_id, text, owner, due, t0, now],
            )?;
            Ok(())
        })?;
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> Result<Action, ActionError> {
        self.store
            .with_conn(|c| {
                Ok(c.query_row(
                    &format!("SELECT {COLS} FROM meeting_actions WHERE id = ?1"),
                    params![id],
                    row,
                )
                .ok())
            })?
            .ok_or(ActionError::NotFound)
    }

    /// Open first (by due date, undated last), then the done ones.
    pub fn list(
        &self,
        status: Option<&str>,
        owner: Option<&str>,
    ) -> Result<Vec<Action>, ActionError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare(&format!(
                "SELECT {COLS} FROM meeting_actions WHERE (?1 IS NULL OR status = ?1) AND (?2 IS NULL OR owner = ?2)
                 ORDER BY status DESC, due IS NULL, due, created_at"
            ))?;
            let rows = st
                .query_map(params![status, owner], row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    pub fn set_done(&self, id: &str, done: bool) -> Result<(), ActionError> {
        let n = self.store.with_conn(|c| {
            Ok(c.execute(
                "UPDATE meeting_actions SET status = ?2 WHERE id = ?1",
                params![id, if done { "done" } else { "open" }],
            )?)
        })?;
        if n == 0 {
            Err(ActionError::NotFound)
        } else {
            Ok(())
        }
    }

    pub fn delete(&self, id: &str) -> Result<(), ActionError> {
        let n = self.store.with_conn(|c| {
            Ok(c.execute("DELETE FROM meeting_actions WHERE id = ?1", params![id])?)
        })?;
        if n == 0 {
            Err(ActionError::NotFound)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> ActionsRepo {
        ActionsRepo::new(Store::open_in_memory().unwrap())
    }

    #[test]
    fn dates_are_calendar_checked() {
        assert!(valid_date("2026-10-31"));
        assert!(valid_date("2028-02-29"));
        for bad in [
            "2026-02-30",
            "2026-13-01",
            "26-10-05",
            "2026/10/05",
            "amanhã",
            "2026-10-5",
        ] {
            assert!(!valid_date(bad), "{bad}");
        }
    }

    #[test]
    fn actions_keep_owner_due_and_source_minute_and_validate_input() {
        let r = repo();
        let a = r
            .add(
                None,
                "  enviar a proposta  ",
                "you",
                Some("2026-10-09"),
                Some(125_000),
                100,
            )
            .unwrap();
        assert_eq!(
            (a.owner.as_str(), a.due.as_deref(), a.t0, a.status.as_str()),
            ("you", Some("2026-10-09"), Some(125_000), "open")
        );
        assert_eq!(a.text, "enviar a proposta");
        assert_eq!(
            r.add(None, " ", "you", None, None, 1).unwrap_err(),
            ActionError::Empty
        );
        assert_eq!(
            r.add(None, "x", "ana", None, None, 1).unwrap_err(),
            ActionError::BadOwner
        );
        assert_eq!(
            r.add(None, "x", "you", Some("2026-02-30"), None, 1)
                .unwrap_err(),
            ActionError::BadDue
        );
    }

    #[test]
    fn open_actions_list_by_due_date_and_overdue_ones_are_flagged() {
        let r = repo();
        r.add(None, "sem prazo", "you", None, None, 1).unwrap();
        r.add(None, "depois", "them", Some("2026-11-01"), None, 2)
            .unwrap();
        let early = r
            .add(None, "cedo", "you", Some("2026-10-01"), None, 3)
            .unwrap();
        let texts: Vec<_> = r
            .list(Some("open"), None)
            .unwrap()
            .into_iter()
            .map(|a| a.text)
            .collect();
        assert_eq!(texts, ["cedo", "depois", "sem prazo"]);
        assert!(early.overdue("2026-10-05"));
        assert!(!early.overdue("2026-10-01"), "due today is not overdue yet");
        let mine: Vec<_> = r.list(None, Some("you")).unwrap();
        assert_eq!(mine.len(), 2);
        r.set_done(&early.id, true).unwrap();
        assert!(
            !r.get(&early.id).unwrap().overdue("2026-12-01"),
            "done is never overdue"
        );
        assert_eq!(r.list(Some("open"), None).unwrap().len(), 2);
        r.set_done(&early.id, false).unwrap();
        r.delete(&early.id).unwrap();
        assert_eq!(r.delete(&early.id), Err(ActionError::NotFound));
        assert_eq!(r.set_done("nope", true), Err(ActionError::NotFound));
    }
}
