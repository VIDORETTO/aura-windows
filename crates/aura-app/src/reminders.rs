//! Reminders by conversation (020): "me lembra às 15h de ligar para o João".
//! The agent creates them with its tools; the host checks `fire_due` on a
//! timer and tells the Overlay to show a Windows notification. Time goes
//! through `Clock` so tests control "now" and the UTC offset.

use aura_store::{Store, StoreError, now_secs};
use rusqlite::params;
use serde::Serialize;
use thiserror::Error;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

/// Source of "now" and of the user's UTC offset.
pub trait Clock: Send + Sync {
    fn now(&self) -> i64;
    /// Seconds east of UTC at `now`.
    fn utc_offset_secs(&self) -> i32;
}

/// The real clock; the offset is the Windows time zone (UTC if unknown).
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> i64 {
        now_secs()
    }
    fn utc_offset_secs(&self) -> i32 {
        UtcOffset::current_local_offset()
            .map(|o| o.whole_seconds())
            .unwrap_or(0)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReminderError {
    #[error("informe o texto do lembrete")]
    EmptyText,
    #[error("data e hora inválidas: use AAAA-MM-DDTHH:MM (hora local) ou delay_minutes")]
    BadTime,
    #[error("esse horário já passou")]
    InThePast,
    #[error("repetição inválida: use none, daily, weekdays ou weekly")]
    BadRepeat,
    #[error("lembrete não encontrado")]
    NotFound,
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for ReminderError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    None,
    Daily,
    Weekdays,
    Weekly,
}

impl Repeat {
    pub fn parse(s: &str) -> Result<Self, ReminderError> {
        match s {
            "" | "none" => Ok(Self::None),
            "daily" => Ok(Self::Daily),
            "weekdays" => Ok(Self::Weekdays),
            "weekly" => Ok(Self::Weekly),
            _ => Err(ReminderError::BadRepeat),
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Daily => "daily",
            Self::Weekdays => "weekdays",
            Self::Weekly => "weekly",
        }
    }
    /// Next occurrence after `due`, strictly after `now` (missed ones are skipped).
    fn next(self, due: i64, now: i64, offset: i32) -> Option<i64> {
        const DAY: i64 = 86_400;
        let mut next = due;
        loop {
            next += match self {
                Self::None => return None,
                Self::Daily | Self::Weekdays => DAY,
                Self::Weekly => 7 * DAY,
            };
            if next > now && (self != Self::Weekdays || !is_weekend(next, offset)) {
                return Some(next);
            }
        }
    }
}

fn is_weekend(ts: i64, offset: i32) -> bool {
    let Ok(o) = UtcOffset::from_whole_seconds(offset) else {
        return false;
    };
    OffsetDateTime::from_unix_timestamp(ts)
        .map(|d| {
            matches!(
                d.to_offset(o).weekday(),
                time::Weekday::Saturday | time::Weekday::Sunday
            )
        })
        .unwrap_or(false)
}

fn bad<E>(_: E) -> ReminderError {
    ReminderError::BadTime
}

/// `2026-10-05T15:00` (local time, seconds optional) or RFC 3339 with an offset.
pub fn parse_time(input: &str, offset: i32) -> Result<i64, ReminderError> {
    let input = input.trim();
    if let Ok(d) = OffsetDateTime::parse(input, &time::format_description::well_known::Rfc3339) {
        return Ok(d.unix_timestamp());
    }
    let (date, clock) = input.split_once(['T', ' ']).ok_or(ReminderError::BadTime)?;
    let mut d = date.split('-').map(|p| p.parse::<i32>());
    let (Some(Ok(y)), Some(Ok(m)), Some(Ok(day)), None) = (d.next(), d.next(), d.next(), d.next())
    else {
        return Err(ReminderError::BadTime);
    };
    let mut t = clock.split(':').map(|p| p.parse::<u8>());
    let (Some(Ok(h)), Some(Ok(min))) = (t.next(), t.next()) else {
        return Err(ReminderError::BadTime);
    };
    let sec = t.next().map(|s| s.map_err(bad)).transpose()?.unwrap_or(0);
    let date = Date::from_calendar_date(y, Month::try_from(m as u8).map_err(bad)?, day as u8)
        .map_err(bad)?;
    let time = Time::from_hms(h, min, sec).map_err(bad)?;
    let off = UtcOffset::from_whole_seconds(offset).map_err(bad)?;
    Ok(PrimitiveDateTime::new(date, time)
        .assume_offset(off)
        .unix_timestamp())
}

/// Local `YYYY-MM-DDTHH:MM:SS±HH:MM` for the model to read the clock.
pub fn format_local(ts: i64, offset: i32) -> String {
    let o = UtcOffset::from_whole_seconds(offset).unwrap_or(UtcOffset::UTC);
    OffsetDateTime::from_unix_timestamp(ts)
        .map(|d| d.to_offset(o))
        .ok()
        .and_then(|d| {
            d.format(&time::format_description::well_known::Rfc3339)
                .ok()
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: String,
    pub text: String,
    pub due_at: i64,
    pub repeat: String,
    /// An instruction the agent runs in a new Chat conversation when it is
    /// due ("every weekday at 8, summarize my open commitments").
    pub prompt: Option<String>,
}

#[derive(Clone)]
pub struct RemindersRepo {
    store: Store,
}

impl RemindersRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn create(
        &self,
        text: &str,
        due_at: i64,
        repeat: Repeat,
        now: i64,
        prompt: Option<&str>,
    ) -> Result<Reminder, ReminderError> {
        let prompt = prompt.map(str::trim).filter(|p| !p.is_empty());
        let text = text.trim();
        if text.is_empty() {
            return Err(ReminderError::EmptyText);
        }
        if due_at <= now {
            return Err(ReminderError::InThePast);
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO reminders(id, text, due_at, repeat, done, created_at, prompt) VALUES (?1,?2,?3,?4,0,?5,?6)",
                params![id, text, due_at, repeat.key(), now, prompt],
            )?;
            Ok(())
        })?;
        Ok(Reminder {
            id,
            text: text.into(),
            due_at,
            repeat: repeat.key().into(),
            prompt: prompt.map(str::to_string),
        })
    }

    pub fn list(&self) -> Result<Vec<Reminder>, ReminderError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare(
                "SELECT id, text, due_at, repeat, prompt FROM reminders WHERE done = 0 ORDER BY due_at",
            )?;
            let rows = st
                .query_map([], |r| {
                    Ok(Reminder {
                        id: r.get(0)?,
                        text: r.get(1)?,
                        due_at: r.get(2)?,
                        repeat: r.get(3)?,
                        prompt: r.get(4)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    pub fn delete(&self, id: &str) -> Result<(), ReminderError> {
        let n = self.store.with_conn(|c| {
            Ok(c.execute(
                "DELETE FROM reminders WHERE id = ?1 AND done = 0",
                params![id],
            )?)
        })?;
        if n == 0 {
            Err(ReminderError::NotFound)
        } else {
            Ok(())
        }
    }

    /// Reminders due at `now`: one-off ones are marked done, repeating ones
    /// move to their next occurrence (missed occurrences are not replayed).
    pub fn fire_due(&self, now: i64, offset: i32) -> Result<Vec<Reminder>, ReminderError> {
        let due: Vec<(Reminder, String)> = self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT id, text, due_at, repeat, prompt FROM reminders WHERE done = 0 AND due_at <= ?1 ORDER BY due_at")?;
            let rows = st
                .query_map(params![now], |r| Ok((Reminder { id: r.get(0)?, text: r.get(1)?, due_at: r.get(2)?, repeat: r.get::<_, String>(3)?, prompt: r.get(4)? }, r.get::<_, String>(3)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;
        for (r, rep) in &due {
            let next = Repeat::parse(rep)
                .ok()
                .and_then(|p| p.next(r.due_at, now, offset));
            self.store.with_conn(|c| {
                match next {
                    Some(n) => c.execute(
                        "UPDATE reminders SET due_at = ?2 WHERE id = ?1",
                        params![r.id, n],
                    )?,
                    None => {
                        c.execute("UPDATE reminders SET done = 1 WHERE id = ?1", params![r.id])?
                    }
                };
                Ok(())
            })?;
        }
        Ok(due.into_iter().map(|(r, _)| r).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRT: i32 = -3 * 3600;
    // 2026-10-05 12:00:00 UTC (a Monday).
    const NOON_UTC: i64 = 1_791_201_600;

    fn repo() -> RemindersRepo {
        RemindersRepo::new(Store::open_in_memory().unwrap())
    }

    #[test]
    fn local_time_uses_the_offset_and_rfc3339_wins() {
        // 15:00 in UTC-3 is 18:00 UTC.
        assert_eq!(
            parse_time("2026-10-05T15:00", BRT).unwrap(),
            NOON_UTC + 6 * 3600
        );
        assert_eq!(
            parse_time("2026-10-05 15:00:30", BRT).unwrap(),
            NOON_UTC + 6 * 3600 + 30
        );
        assert_eq!(
            parse_time("2026-10-05T18:00:00Z", BRT).unwrap(),
            NOON_UTC + 6 * 3600
        );
        for bad in [
            "amanhã",
            "2026-13-01T10:00",
            "2026-10-05T25:00",
            "2026-10-05",
        ] {
            assert_eq!(parse_time(bad, BRT), Err(ReminderError::BadTime), "{bad}");
        }
        assert_eq!(format_local(NOON_UTC, BRT), "2026-10-05T09:00:00-03:00");
    }

    #[test]
    fn create_validates_and_lists_by_time() {
        let r = repo();
        assert_eq!(
            r.create("  ", NOON_UTC + 60, Repeat::None, NOON_UTC, None),
            Err(ReminderError::EmptyText)
        );
        assert_eq!(
            r.create("ligar", NOON_UTC, Repeat::None, NOON_UTC, None),
            Err(ReminderError::InThePast)
        );
        r.create("depois", NOON_UTC + 7200, Repeat::None, NOON_UTC, None)
            .unwrap();
        r.create("antes", NOON_UTC + 60, Repeat::None, NOON_UTC, None)
            .unwrap();
        let texts: Vec<_> = r.list().unwrap().into_iter().map(|x| x.text).collect();
        assert_eq!(texts, ["antes", "depois"]);
    }

    #[test]
    fn due_reminders_fire_once() {
        let r = repo();
        let a = r
            .create(
                "ligar para o João",
                NOON_UTC + 60,
                Repeat::None,
                NOON_UTC,
                None,
            )
            .unwrap();
        assert!(r.fire_due(NOON_UTC + 59, BRT).unwrap().is_empty());
        let fired = r.fire_due(NOON_UTC + 61, BRT).unwrap();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].id, a.id);
        assert!(r.fire_due(NOON_UTC + 9999, BRT).unwrap().is_empty());
        assert!(r.list().unwrap().is_empty());
    }

    #[test]
    fn repeating_reminders_move_to_the_next_day_and_skip_missed_ones() {
        let r = repo();
        let d = r
            .create("daily", NOON_UTC + 60, Repeat::Daily, NOON_UTC, None)
            .unwrap();
        // Aura was closed for three days: fires once, next one is in the future.
        let now = NOON_UTC + 3 * 86_400 + 120;
        assert_eq!(r.fire_due(now, BRT).unwrap().len(), 1);
        let next = r
            .list()
            .unwrap()
            .into_iter()
            .find(|x| x.id == d.id)
            .unwrap();
        assert_eq!(next.due_at, NOON_UTC + 60 + 4 * 86_400);
    }

    #[test]
    fn weekdays_skip_the_weekend() {
        // Friday 2026-10-09 15:00 local; the next weekday reminder is Monday.
        let friday = parse_time("2026-10-09T15:00", BRT).unwrap();
        let monday = parse_time("2026-10-12T15:00", BRT).unwrap();
        assert_eq!(Repeat::Weekdays.next(friday, friday + 1, BRT), Some(monday));
        assert_eq!(
            Repeat::Weekly.next(friday, friday + 1, BRT),
            Some(friday + 7 * 86_400)
        );
        assert_eq!(Repeat::None.next(friday, friday + 1, BRT), None);
        assert_eq!(Repeat::parse("monthly"), Err(ReminderError::BadRepeat));
    }

    #[test]
    fn delete_removes_only_active_reminders() {
        let r = repo();
        let a = r
            .create("x", NOON_UTC + 60, Repeat::None, NOON_UTC, None)
            .unwrap();
        r.delete(&a.id).unwrap();
        assert_eq!(r.delete(&a.id), Err(ReminderError::NotFound));
    }
}
