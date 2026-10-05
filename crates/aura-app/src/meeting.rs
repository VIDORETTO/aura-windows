//! The Meeting engine (023): a session the user starts and ends, with the
//! speech of both sides stored as timed utterances. It never starts capture on
//! its own; the host asks for audio only between `start` and `stop`.
//!
//! Speech comes from a `LineSource` (the host's recent-audio buffers), polled
//! every few seconds. Utterances are stored once: the cursor only moves
//! forward, so a poll never repeats what an earlier poll saved.

use aura_mcp::BoxFut;
use aura_store::{Store, StoreError};
use rusqlite::params;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MeetingError {
    #[error("já existe uma reunião em andamento")]
    AlreadyActive,
    #[error("nenhuma reunião em andamento")]
    NotActive,
    #[error("reunião não encontrada")]
    NotFound,
    #[error("não há fala nesse período (o buffer de áudio está ligado?)")]
    NothingSaid,
    #[error("transcrição indisponível: {0}")]
    Source(String),
    #[error("store: {0}")]
    Store(String),
}

impl From<StoreError> for MeetingError {
    fn from(e: StoreError) -> Self {
        Self::Store(e.to_string())
    }
}

/// Who spoke: the microphone is "you", the system audio is "them".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    You,
    Them,
}

impl Speaker {
    pub fn key(self) -> &'static str {
        match self {
            Self::You => "you",
            Self::Them => "them",
        }
    }
}

/// A transcribed line with absolute times (ms since the epoch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLine {
    pub speaker: Speaker,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

/// Where speech comes from (the host's audio buffers).
pub trait LineSource: Send + Sync {
    fn lines<'a>(&'a self, from_ms: i64, to_ms: i64)
    -> BoxFut<'a, Result<Vec<SourceLine>, String>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub briefing: String,
    /// `live` (started by the user) or `buffer` ("esqueci de iniciar").
    pub origin: String,
    /// `active`, `ended` or `interrupted` (the app closed during the meeting).
    pub status: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Utterance {
    pub id: i64,
    pub meeting_id: String,
    /// Milliseconds since the meeting started.
    pub t0: i64,
    pub t1: i64,
    /// `you` or `them`.
    pub speaker: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub meeting_id: String,
    pub title: String,
    pub started_at: i64,
    pub t0: i64,
    pub speaker: String,
    pub text: String,
}

#[derive(Clone)]
pub struct MeetingRepo {
    store: Store,
}

fn meeting_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Meeting> {
    Ok(Meeting {
        id: r.get(0)?,
        title: r.get(1)?,
        kind: r.get(2)?,
        briefing: r.get(3)?,
        origin: r.get(4)?,
        status: r.get(5)?,
        started_at: r.get(6)?,
        ended_at: r.get(7)?,
    })
}

const MEETING_COLS: &str = "id, title, kind, briefing, origin, status, started_at, ended_at";

impl MeetingRepo {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    fn create(
        &self,
        title: &str,
        kind: &str,
        briefing: &str,
        origin: &str,
        status: &str,
        started_at: i64,
    ) -> Result<Meeting, MeetingError> {
        let id = uuid::Uuid::new_v4().to_string();
        let title = if title.trim().is_empty() {
            "Reunião"
        } else {
            title.trim()
        };
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO meetings(id, title, kind, briefing, origin, status, started_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![id, title, kind, briefing, origin, status, started_at],
            )?;
            Ok(())
        })?;
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> Result<Meeting, MeetingError> {
        self.store
            .with_conn(|c| {
                Ok(c.query_row(
                    &format!("SELECT {MEETING_COLS} FROM meetings WHERE id = ?1"),
                    params![id],
                    meeting_row,
                )
                .ok())
            })?
            .ok_or(MeetingError::NotFound)
    }

    /// Newest first.
    pub fn list(&self, limit: usize) -> Result<Vec<Meeting>, MeetingError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare(&format!(
                "SELECT {MEETING_COLS} FROM meetings ORDER BY started_at DESC LIMIT ?1"
            ))?;
            let rows = st
                .query_map(params![limit as i64], meeting_row)?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    fn finish(&self, id: &str, status: &str, ended_at: i64) -> Result<(), MeetingError> {
        self.store.with_conn(|c| {
            c.execute(
                "UPDATE meetings SET status = ?2, ended_at = ?3 WHERE id = ?1",
                params![id, status, ended_at],
            )?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn set_briefing(&self, id: &str, briefing: &str) -> Result<(), MeetingError> {
        let n = self.store.with_conn(|c| {
            Ok(c.execute(
                "UPDATE meetings SET briefing = ?2 WHERE id = ?1",
                params![id, briefing],
            )?)
        })?;
        if n == 0 {
            Err(MeetingError::NotFound)
        } else {
            Ok(())
        }
    }

    pub fn delete(&self, id: &str) -> Result<(), MeetingError> {
        let n = self
            .store
            .with_conn(|c| Ok(c.execute("DELETE FROM meetings WHERE id = ?1", params![id])?))?;
        if n == 0 {
            Err(MeetingError::NotFound)
        } else {
            Ok(())
        }
    }

    fn add_utterances(
        &self,
        id: &str,
        lines: &[(i64, i64, Speaker, String)],
    ) -> Result<Vec<Utterance>, MeetingError> {
        let mut out = Vec::new();
        for (t0, t1, who, text) in lines {
            let row = self.store.with_conn(|c| {
                c.execute(
                    "INSERT INTO meeting_utterances(meeting_id, t0, t1, speaker, text) VALUES (?1,?2,?3,?4,?5)",
                    params![id, t0, t1, who.key(), text],
                )?;
                Ok(c.last_insert_rowid())
            })?;
            out.push(Utterance {
                id: row,
                meeting_id: id.into(),
                t0: *t0,
                t1: *t1,
                speaker: who.key().into(),
                text: text.clone(),
            });
        }
        Ok(out)
    }

    /// Test helper: stores `(t0, t1, speaker, text)` rows as they are.
    #[doc(hidden)]
    pub fn store_utterances_for_tests(
        &self,
        id: &str,
        rows: &[(i64, i64, &str, &str)],
    ) -> Result<(), MeetingError> {
        for (t0, t1, who, text) in rows {
            self.store.with_conn(|c| {
                c.execute(
                    "INSERT INTO meeting_utterances(meeting_id, t0, t1, speaker, text) VALUES (?1,?2,?3,?4,?5)",
                    params![id, t0, t1, who, text],
                )?;
                Ok(())
            })?;
        }
        Ok(())
    }

    pub fn utterances(&self, id: &str) -> Result<Vec<Utterance>, MeetingError> {
        Ok(self.store.with_conn(|c| {
            let mut st = c.prepare("SELECT id, meeting_id, t0, t1, speaker, text FROM meeting_utterances WHERE meeting_id = ?1 ORDER BY t0, id")?;
            let rows = st
                .query_map(params![id], |r| {
                    Ok(Utterance { id: r.get(0)?, meeting_id: r.get(1)?, t0: r.get(2)?, t1: r.get(3)?, speaker: r.get(4)?, text: r.get(5)? })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?)
    }

    /// Utterances matching every word of `query` (accent-insensitive), newest
    /// meetings first; `meeting` limits the search to one meeting.
    pub fn search(
        &self,
        query: &str,
        meeting: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Hit>, MeetingError> {
        let words: Vec<String> = crate::notes::fold(query)
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let rows: Vec<Hit> = self.store.with_conn(|c| {
            let mut st = c.prepare(
                "SELECT m.id, m.title, m.started_at, u.t0, u.speaker, u.text FROM meeting_utterances u JOIN meetings m ON m.id = u.meeting_id WHERE (?1 IS NULL OR m.id = ?1) ORDER BY m.started_at DESC, u.t0",
            )?;
            let rows = st
                .query_map(params![meeting], |r| {
                    Ok(Hit { meeting_id: r.get(0)?, title: r.get(1)?, started_at: r.get(2)?, t0: r.get(3)?, speaker: r.get(4)?, text: r.get(5)? })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;
        Ok(rows
            .into_iter()
            .filter(|h| {
                let t = crate::notes::fold(&h.text);
                words.iter().all(|w| t.contains(w))
            })
            .take(limit)
            .collect())
    }

    /// Meetings left `active` by a crash or a forced close become `interrupted`.
    pub fn recover(&self, now_ms: i64) -> Result<usize, MeetingError> {
        Ok(self.store.with_conn(|c| {
            Ok(c.execute(
                "UPDATE meetings SET status = 'interrupted', ended_at = ?1 WHERE status = 'active'",
                params![now_ms],
            )?)
        })?)
    }
}

struct Active {
    id: String,
    started_ms: i64,
    /// Everything before this instant has been stored.
    cursor_ms: i64,
    paused: bool,
}

/// How far behind "now" a poll reads, so a line still being spoken is not cut.
const LAG_MS: i64 = 3_000;

pub struct MeetingService {
    repo: MeetingRepo,
    source: Arc<dyn LineSource>,
    active: Mutex<Option<Active>>,
}

fn to_rows(
    lines: Vec<SourceLine>,
    started_ms: i64,
    from_ms: i64,
) -> Vec<(i64, i64, Speaker, String)> {
    let mut lines: Vec<_> = lines
        .into_iter()
        .filter(|l| l.start_ms >= from_ms && !l.text.trim().is_empty())
        .collect();
    lines.sort_by_key(|l| (l.start_ms, l.speaker == Speaker::You));
    lines
        .into_iter()
        .map(|l| {
            (
                (l.start_ms - started_ms).max(0),
                (l.end_ms - started_ms).max(0),
                l.speaker,
                l.text.trim().to_string(),
            )
        })
        .collect()
}

impl MeetingService {
    pub fn new(repo: MeetingRepo, source: Arc<dyn LineSource>) -> Self {
        Self {
            repo,
            source,
            active: Mutex::new(None),
        }
    }

    pub fn repo(&self) -> &MeetingRepo {
        &self.repo
    }

    /// The meeting in progress, if any.
    pub fn active(&self) -> Option<Meeting> {
        let id = self.active.lock().unwrap().as_ref()?.id.clone();
        self.repo.get(&id).ok()
    }

    pub fn is_paused(&self) -> bool {
        self.active
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|a| a.paused)
    }

    pub fn start(
        &self,
        title: &str,
        kind: &str,
        briefing: &str,
        now_ms: i64,
    ) -> Result<Meeting, MeetingError> {
        let mut active = self.active.lock().unwrap();
        if active.is_some() {
            return Err(MeetingError::AlreadyActive);
        }
        let m = self
            .repo
            .create(title, kind, briefing, "live", "active", now_ms)?;
        *active = Some(Active {
            id: m.id.clone(),
            started_ms: now_ms,
            cursor_ms: now_ms,
            paused: false,
        });
        Ok(m)
    }

    /// Pausing skips the time in between: it is never transcribed.
    pub fn set_paused(&self, paused: bool, now_ms: i64) -> Result<(), MeetingError> {
        let mut active = self.active.lock().unwrap();
        let a = active.as_mut().ok_or(MeetingError::NotActive)?;
        if a.paused && !paused {
            a.cursor_ms = now_ms;
        }
        a.paused = paused;
        Ok(())
    }

    /// Stores what was said since the last poll; returns the new utterances.
    pub async fn poll(&self, now_ms: i64) -> Result<Vec<Utterance>, MeetingError> {
        let (id, started, cursor) = {
            let g = self.active.lock().unwrap();
            let Some(a) = g.as_ref().filter(|a| !a.paused) else {
                return Ok(Vec::new());
            };
            (a.id.clone(), a.started_ms, a.cursor_ms)
        };
        let to = now_ms - LAG_MS;
        if to <= cursor {
            return Ok(Vec::new());
        }
        let lines = self
            .source
            .lines(cursor, to)
            .await
            .map_err(MeetingError::Source)?;
        let rows = to_rows(lines, started, cursor);
        let stored = self.repo.add_utterances(&id, &rows)?;
        if let Some(a) = self.active.lock().unwrap().as_mut().filter(|a| a.id == id) {
            a.cursor_ms = to;
        }
        Ok(stored)
    }

    /// Ends the meeting after one last poll.
    pub async fn stop(&self, now_ms: i64) -> Result<Meeting, MeetingError> {
        if self.active.lock().unwrap().is_none() {
            return Err(MeetingError::NotActive);
        }
        // Read up to "now": nothing is spoken after the user pressed stop.
        let (id, started, cursor, paused) = {
            let g = self.active.lock().unwrap();
            let a = g.as_ref().ok_or(MeetingError::NotActive)?;
            (a.id.clone(), a.started_ms, a.cursor_ms, a.paused)
        };
        if !paused && now_ms > cursor {
            match self.source.lines(cursor, now_ms).await {
                Ok(lines) => {
                    self.repo
                        .add_utterances(&id, &to_rows(lines, started, cursor))?;
                }
                Err(e) => tracing::warn!("last meeting poll: {e}"),
            }
        }
        *self.active.lock().unwrap() = None;
        self.repo.finish(&id, "ended", now_ms)?;
        self.repo.get(&id)
    }

    /// "Esqueci de iniciar": a meeting made from the last `minutes` of the
    /// audio buffer.
    pub async fn from_buffer(
        &self,
        title: &str,
        minutes: u32,
        now_ms: i64,
    ) -> Result<Meeting, MeetingError> {
        let from = now_ms - minutes.max(1) as i64 * 60_000;
        let lines = self
            .source
            .lines(from, now_ms)
            .await
            .map_err(MeetingError::Source)?;
        let first = lines
            .iter()
            .map(|l| l.start_ms)
            .min()
            .ok_or(MeetingError::NothingSaid)?;
        let m = self
            .repo
            .create(title, "other", "", "buffer", "ended", first)?;
        self.repo
            .add_utterances(&m.id, &to_rows(lines, first, from))?;
        self.repo.finish(&m.id, "ended", now_ms)?;
        self.repo.get(&m.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lines the "microphone" and "system audio" produce, by absolute time.
    struct Fake(Mutex<Vec<SourceLine>>);

    impl LineSource for Fake {
        fn lines<'a>(&'a self, from: i64, to: i64) -> BoxFut<'a, Result<Vec<SourceLine>, String>> {
            Box::pin(async move {
                Ok(self
                    .0
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|l| l.start_ms >= from && l.start_ms < to)
                    .cloned()
                    .collect())
            })
        }
    }

    fn line(who: Speaker, t: i64, text: &str) -> SourceLine {
        SourceLine {
            speaker: who,
            start_ms: t,
            end_ms: t + 1500,
            text: text.into(),
        }
    }

    fn service(lines: Vec<SourceLine>) -> (MeetingService, Arc<Fake>) {
        let fake = Arc::new(Fake(Mutex::new(lines)));
        let svc = MeetingService::new(
            MeetingRepo::new(Store::open_in_memory().unwrap()),
            fake.clone(),
        );
        (svc, fake)
    }

    const T0: i64 = 1_000_000;

    #[tokio::test]
    async fn live_meeting_stores_each_line_once_with_speaker_and_time() {
        let (svc, fake) = service(vec![
            line(Speaker::Them, T0 + 5_000, "Bom dia, vamos começar"),
            line(Speaker::You, T0 + 7_000, "Bom dia"),
        ]);
        let m = svc.start("Revisão Q3", "decision", "", T0).unwrap();
        assert_eq!(
            svc.start("outra", "x", "", T0).unwrap_err(),
            MeetingError::AlreadyActive
        );
        // Too early: nothing is old enough to read.
        assert!(svc.poll(T0 + 2_000).await.unwrap().is_empty());
        let first = svc.poll(T0 + 20_000).await.unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!((first[0].speaker.as_str(), first[0].t0), ("them", 5_000));
        assert_eq!(first[1].speaker, "you");
        // A second poll over the same window repeats nothing.
        assert!(svc.poll(T0 + 21_000).await.unwrap().is_empty());
        fake.0
            .lock()
            .unwrap()
            .push(line(Speaker::Them, T0 + 25_000, "Próximo assunto"));
        let again = svc.poll(T0 + 40_000).await.unwrap();
        assert_eq!(again.len(), 1);
        let ended = svc.stop(T0 + 41_000).await.unwrap();
        assert_eq!(ended.status, "ended");
        assert_eq!(ended.ended_at, Some(T0 + 41_000));
        assert_eq!(svc.repo().utterances(&m.id).unwrap().len(), 3);
        assert!(svc.active().is_none());
        assert_eq!(
            svc.stop(T0 + 50_000).await.unwrap_err(),
            MeetingError::NotActive
        );
    }

    #[tokio::test]
    async fn stopping_reads_the_last_seconds_too() {
        let (svc, _) = service(vec![line(Speaker::Them, T0 + 9_000, "Obrigado a todos")]);
        let m = svc.start("x", "other", "", T0).unwrap();
        svc.stop(T0 + 10_000).await.unwrap();
        assert_eq!(svc.repo().utterances(&m.id).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn pausing_skips_the_time_in_between() {
        let (svc, _) = service(vec![
            line(Speaker::Them, T0 + 5_000, "antes"),
            line(Speaker::Them, T0 + 15_000, "durante a pausa"),
            line(Speaker::Them, T0 + 35_000, "depois"),
        ]);
        let m = svc.start("x", "other", "", T0).unwrap();
        svc.poll(T0 + 10_000).await.unwrap();
        svc.set_paused(true, T0 + 10_000).unwrap();
        assert!(svc.is_paused());
        assert!(svc.poll(T0 + 30_000).await.unwrap().is_empty());
        svc.set_paused(false, T0 + 30_000).unwrap();
        svc.stop(T0 + 40_000).await.unwrap();
        let texts: Vec<_> = svc
            .repo()
            .utterances(&m.id)
            .unwrap()
            .into_iter()
            .map(|u| u.text)
            .collect();
        assert_eq!(texts, ["antes", "depois"]);
    }

    #[tokio::test]
    async fn forgot_to_start_makes_a_meeting_from_the_buffer() {
        let now = T0 + 20 * 60_000;
        let (svc, _) = service(vec![
            line(Speaker::Them, now - 600_000, "Podemos fechar o orçamento?"),
            line(Speaker::You, now - 590_000, "Sim, com corte de 10%"),
            line(Speaker::Them, now - 3_600_000, "fora da janela"),
        ]);
        let m = svc.from_buffer("Chamada com a Ana", 15, now).await.unwrap();
        assert_eq!((m.origin.as_str(), m.status.as_str()), ("buffer", "ended"));
        assert_eq!(m.started_at, now - 600_000);
        let u = svc.repo().utterances(&m.id).unwrap();
        assert_eq!(u.len(), 2);
        assert_eq!(u[0].t0, 0);
        let (empty, _) = service(vec![]);
        assert_eq!(
            empty.from_buffer("x", 5, now).await.unwrap_err(),
            MeetingError::NothingSaid
        );
    }

    #[tokio::test]
    async fn search_lists_meetings_and_finds_words_without_accents() {
        let (svc, _) = service(vec![line(
            Speaker::Them,
            T0 + 5_000,
            "Vamos cortar o orçamento de mídia",
        )]);
        let m = svc.start("Revisão", "other", "", T0).unwrap();
        svc.stop(T0 + 10_000).await.unwrap();
        let hits = svc.repo().search("orcamento midia", None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].title.as_str(), hits[0].t0), ("Revisão", 5_000));
        assert!(
            svc.repo()
                .search("orcamento", Some("outra"), 10)
                .unwrap()
                .is_empty()
        );
        assert_eq!(svc.repo().list(10).unwrap()[0].id, m.id);
        svc.repo().delete(&m.id).unwrap();
        assert!(svc.repo().utterances(&m.id).unwrap().is_empty(), "cascade");
    }

    #[test]
    fn crashed_meetings_are_marked_interrupted() {
        let (svc, _) = service(vec![]);
        let m = svc.start("x", "other", "", T0).unwrap();
        assert_eq!(svc.repo().recover(T0 + 5).unwrap(), 1);
        assert_eq!(svc.repo().get(&m.id).unwrap().status, "interrupted");
    }
}
