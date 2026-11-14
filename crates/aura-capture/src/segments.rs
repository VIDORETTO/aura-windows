//! Encrypted segment storage and the background recorder (ADR 0004).
//!
//! Frames → privacy decision + redaction → [`VideoEncoder`] (H.264 fMP4 in
//! memory on Windows) → [`Sealer`] (vault, key `capture-segment`) → `.seg`
//! file + index row. Plaintext never touches the disk (OT-003).

use crate::frame::redact;
use crate::retention::{RetentionPolicy, SegmentMeta};
use crate::source::{FrameSource, Target, WindowInventory};
use aura_policy::{AccessRequest, Decision, Grants, Policy, Requester, Source, decide};
use aura_store::Store;
use rusqlite::{OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use crate::encoder::{NullEncoder, Sealer, SegmentError, VideoEncoder};

/// Columns of [`SegmentMeta`], with `manual` from the recording.
const SELECT_META: &str = "SELECT s.id, s.source, s.kind, s.recording_id, s.start_ms, s.end_ms, s.bytes, COALESCE(r.manual, 0)
     FROM capture_segments s LEFT JOIN recordings r ON r.id = s.recording_id";

/// Segments under the age and size limits (same rule as
/// [`crate::retention::is_limited`]); `?1` = apply to manual recordings.
const LIMITED: &str = "FROM capture_segments s LEFT JOIN recordings r ON r.id = s.recording_id
     WHERE s.kind = 'continuous' OR (s.kind = 'recording' AND (COALESCE(r.manual, 0) = 0 OR ?1))";

/// Interval between orphan sweeps of a running recorder.
pub const SWEEP_EVERY_MS: i64 = 3_600_000;

fn row_meta(r: &rusqlite::Row<'_>) -> rusqlite::Result<SegmentMeta> {
    Ok(SegmentMeta {
        id: r.get(0)?,
        source: r.get(1)?,
        kind: r.get(2)?,
        recording_id: r.get(3)?,
        start_ms: r.get(4)?,
        end_ms: r.get(5)?,
        bytes: r.get::<_, i64>(6)? as u64,
        manual: r.get::<_, i64>(7)? != 0,
    })
}

impl From<aura_store::StoreError> for SegmentError {
    fn from(e: aura_store::StoreError) -> Self {
        SegmentError::Store(e.to_string())
    }
}

impl From<rusqlite::Error> for SegmentError {
    fn from(e: rusqlite::Error) -> Self {
        SegmentError::Store(e.to_string())
    }
}

/// Vault-backed sealer with the `capture-segment` derived key.
pub struct VaultSealer(pub aura_store::Vault);

pub const SEGMENT_KEY_CONTEXT: &str = "capture-segment";

impl Sealer for VaultSealer {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, SegmentError> {
        self.0
            .seal_bytes(SEGMENT_KEY_CONTEXT, plaintext)
            .map_err(|e| SegmentError::Seal(e.to_string()))
    }
    fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, SegmentError> {
        self.0
            .open_bytes(SEGMENT_KEY_CONTEXT, sealed)
            .map_err(|e| SegmentError::Seal(e.to_string()))
    }
}

#[derive(Clone)]
pub struct SegmentStore {
    store: Store,
    root: PathBuf,
    sealer: Arc<dyn Sealer>,
    // Shared by source writers: orphan cleanup must not see a half-published file.
    publication: Arc<Mutex<()>>,
}

impl SegmentStore {
    pub fn new(store: Store, root: PathBuf, sealer: Arc<dyn Sealer>) -> Self {
        Self {
            store,
            root,
            sealer,
            publication: Arc::new(Mutex::new(())),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn write(
        &self,
        meta_in: SegmentMeta,
        plaintext: &[u8],
    ) -> Result<SegmentMeta, SegmentError> {
        let sealed = self.sealer.seal(plaintext)?;
        let _publication = self.publication.lock().unwrap();
        let dir = self.root.join(&meta_in.source);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.seg", meta_in.id));
        let tmp = path.with_extension("seg.tmp");
        std::fs::write(&tmp, &sealed)?;
        std::fs::rename(&tmp, &path)?;
        let meta = SegmentMeta {
            bytes: sealed.len() as u64,
            ..meta_in
        };
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO capture_segments(id, source, kind, recording_id, start_ms, end_ms, path, bytes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![meta.id, meta.source, meta.kind, meta.recording_id, meta.start_ms, meta.end_ms, path.to_string_lossy(), meta.bytes as i64],
            )?;
            Ok(())
        })?;
        Ok(meta)
    }

    pub fn list(&self, source: Option<&str>) -> Result<Vec<SegmentMeta>, SegmentError> {
        Ok(self.store.with_conn(|c| {
            let sql = format!(
                "{SELECT_META} WHERE (?1 IS NULL OR s.source = ?1) ORDER BY s.start_ms, s.rowid"
            );
            let mut stmt = c.prepare(&sql)?;
            let rows = stmt.query_map([source], row_meta)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })?)
    }

    pub fn range(
        &self,
        source: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<SegmentMeta>, SegmentError> {
        Ok(self.store.with_conn(|c| {
            let sql = format!(
                "{SELECT_META} WHERE s.source = ?1 AND s.end_ms > ?2 AND s.start_ms < ?3 ORDER BY s.start_ms, s.rowid"
            );
            let mut stmt = c.prepare(&sql)?;
            let rows = stmt.query_map(params![source, from_ms, to_ms], row_meta)?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })?)
    }

    pub fn read(&self, id: &str) -> Result<Vec<u8>, SegmentError> {
        let path: String = self.store.with_conn(|c| {
            Ok(c.query_row(
                "SELECT path FROM capture_segments WHERE id = ?1",
                [id],
                |r| r.get(0),
            )?)
        })?;
        self.sealer.open(&std::fs::read(path)?)
    }

    /// Deletes rows in one transaction, then their files. A crash between the
    /// two leaves files without rows, which the orphan sweep removes.
    pub fn delete(&self, ids: &[String]) -> Result<(), SegmentError> {
        if ids.is_empty() {
            return Ok(());
        }
        let paths: Vec<String> = self.store.with_conn(|c| {
            let tx = c.transaction()?;
            let mut paths = Vec::with_capacity(ids.len());
            {
                let mut select =
                    tx.prepare_cached("SELECT path FROM capture_segments WHERE id = ?1")?;
                let mut remove = tx.prepare_cached("DELETE FROM capture_segments WHERE id = ?1")?;
                for id in ids {
                    if let Some(p) = select
                        .query_row([id], |r| r.get::<_, String>(0))
                        .optional()?
                    {
                        paths.push(p);
                    }
                    remove.execute([id])?;
                }
            }
            tx.commit()?;
            Ok(paths)
        })?;
        for p in paths {
            let _ = std::fs::remove_file(p);
        }
        Ok(())
    }

    /// Applies the retention policy after a closed segment (041 TK-003).
    /// Deletes exactly what [`crate::retention::plan`] would, but reads only
    /// candidates: expired buffer rows, aggregates of the limited set and, when
    /// something must go, the limited set oldest first until the budget fits.
    /// Orphans are left to [`Self::sweep_orphans`].
    pub fn apply_retention(
        &self,
        policy: &RetentionPolicy,
        now_ms: i64,
    ) -> Result<usize, SegmentError> {
        let _publication = self.publication.lock().unwrap();
        let manual_ok = policy.apply_to_manual as i64;
        let age_cut = now_ms - policy.max_age_ms;
        let mut doomed: Vec<(i64, i64, String)> = Vec::new();
        self.store.with_conn(|c| {
            {
                let mut stmt = c.prepare_cached(
                    "SELECT start_ms, rowid, id FROM capture_segments WHERE kind = 'buffer' AND end_ms < ?1",
                )?;
                let rows = stmt.query_map([now_ms - policy.buffer_ms], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?))
                })?;
                for r in rows {
                    doomed.push(r?);
                }
            }
            let (total, too_old): (i64, i64) = c.query_row(
                &format!("SELECT COALESCE(SUM(s.bytes), 0), COALESCE(SUM(s.end_ms <= ?2), 0) {LIMITED}"),
                params![manual_ok, age_cut],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let mut kept = total as u64;
            let mut old_left = too_old;
            if kept > policy.max_bytes || old_left > 0 {
                let mut stmt = c.prepare(&format!(
                    "SELECT s.start_ms, s.rowid, s.id, s.end_ms, s.bytes {LIMITED} ORDER BY s.start_ms, s.rowid"
                ))?;
                let mut rows = stmt.query(params![manual_ok])?;
                while let Some(r) = rows.next()? {
                    if kept <= policy.max_bytes && old_left == 0 {
                        break;
                    }
                    let end: i64 = r.get(3)?;
                    let bytes = r.get::<_, i64>(4)? as u64;
                    let expired = end <= age_cut;
                    if expired {
                        old_left -= 1;
                    }
                    if expired || kept > policy.max_bytes {
                        kept -= bytes;
                        doomed.push((r.get(0)?, r.get(1)?, r.get(2)?));
                    }
                }
            }
            Ok(())
        })?;
        doomed.sort();
        let ids: Vec<String> = doomed.into_iter().map(|d| d.2).collect();
        self.delete(&ids)?;
        Ok(ids.len())
    }

    /// Removes orphans: `.seg` files without a row, `.seg.tmp` leftovers and
    /// rows without a file. Run when a recorder starts and at most hourly.
    pub fn sweep_orphans(&self) -> Result<usize, SegmentError> {
        let _publication = self.publication.lock().unwrap();
        self.sweep_locked()
    }

    fn sweep_locked(&self) -> Result<usize, SegmentError> {
        let mut removed = 0;
        let known: std::collections::HashSet<String> = self
            .list(None)?
            .into_iter()
            .map(|s| format!("{}.seg", s.id))
            .collect();
        if self.root.exists() {
            for source_dir in std::fs::read_dir(&self.root)?.flatten() {
                if !source_dir.path().is_dir() {
                    continue;
                }
                for f in std::fs::read_dir(source_dir.path())?.flatten() {
                    let name = f.file_name().to_string_lossy().to_string();
                    if name.ends_with(".seg") && !known.contains(&name)
                        || name.ends_with(".seg.tmp")
                    {
                        let _ = std::fs::remove_file(f.path());
                        removed += 1;
                    }
                }
            }
        }
        let ids: Vec<(String, String)> = self.store.with_conn(|c| {
            let mut stmt = c.prepare("SELECT id, path FROM capture_segments")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })?;
        let orphans: Vec<String> = ids
            .into_iter()
            .filter(|(_, p)| !Path::new(p).exists())
            .map(|(id, _)| id)
            .collect();
        removed += orphans.len();
        self.delete(&orphans)?;
        Ok(removed)
    }
}

/// Shared, live-updatable configuration of a background recorder.
#[derive(Clone)]
pub struct RecorderConfig {
    pub policy: Arc<Mutex<Policy>>,
    pub retention: Arc<Mutex<RetentionPolicy>>,
    pub target: Target,
    pub interval: Duration,
    /// `buffer` or `recording`.
    pub kind: String,
    pub recording_id: Option<String>,
}

/// Records frames until the returned handle is dropped or `stop()`ped.
pub struct SegmentRecorder {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<Result<(), SegmentError>>>,
}

impl SegmentRecorder {
    pub fn start(
        cfg: RecorderConfig,
        source: Arc<dyn FrameSource>,
        inventory: Arc<dyn WindowInventory>,
        mut encoder: Box<dyn VideoEncoder>,
        segments: SegmentStore,
        clock: Arc<dyn Fn() -> i64 + Send + Sync>,
    ) -> Self {
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(cfg.interval);
            let mut seg_start: Option<i64> = None;
            if let Err(e) = segments.sweep_orphans() {
                tracing::warn!("orphan sweep: {e}");
            }
            let mut last_sweep = clock();
            let store_seg = |bytes: Vec<u8>, start: i64, end: i64| -> Result<(), SegmentError> {
                segments.write(
                    SegmentMeta {
                        id: uuid::Uuid::new_v4().to_string(),
                        source: "screen".into(),
                        kind: cfg.kind.clone(),
                        recording_id: cfg.recording_id.clone(),
                        start_ms: start,
                        end_ms: end,
                        bytes: 0,
                        manual: false,
                    },
                    &bytes,
                )?;
                let retention = cfg.retention.lock().unwrap().clone();
                segments.apply_retention(&retention, end)?;
                Ok(())
            };
            loop {
                tokio::select! {
                    _ = &mut rx => break,
                    _ = ticker.tick() => {}
                }
                let now = clock();
                if now - last_sweep >= SWEEP_EVERY_MS {
                    if let Err(e) = segments.sweep_orphans() {
                        tracing::warn!("orphan sweep: {e}");
                    }
                    last_sweep = now;
                }
                let decision = {
                    let policy = cfg.policy.lock().unwrap().clone();
                    let area = match &cfg.target {
                        Target::Monitor { area, .. } => *area,
                        Target::Window { .. } => {
                            aura_core::placement::Rect::new(0, 0, i32::MAX / 2, i32::MAX / 2)
                        }
                    };
                    decide(
                        &policy,
                        &Grants::default(),
                        &AccessRequest {
                            source: Source::Screen,
                            requester: Requester::User,
                            target: aura_policy::Target::Monitor { area },
                            visible_windows: inventory.visible_windows(area),
                            background: true,
                        },
                    )
                };
                match decision {
                    Decision::Deny { .. } | Decision::Ask => {
                        // Pause / off / excluded full screen: close the open segment.
                        if let (Some(bytes), Some(start)) = (encoder.flush()?, seg_start.take()) {
                            store_seg(bytes, start, now)?;
                        }
                        continue;
                    }
                    Decision::Allow | Decision::AllowRedacted { .. } => {}
                }
                let mut frame = match source.capture(&cfg.target) {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::debug!("capture skipped: {e}");
                        continue;
                    }
                };
                if let Decision::AllowRedacted { rects } = &decision {
                    redact(&mut frame, rects);
                }
                seg_start.get_or_insert(now);
                if let Some(bytes) = encoder.push(&frame)? {
                    let start = seg_start.take().unwrap_or(now);
                    store_seg(bytes, start, now + cfg.interval.as_millis() as i64)?;
                }
            }
            if let (Some(bytes), Some(start)) = (encoder.flush()?, seg_start) {
                store_seg(bytes, start, clock())?;
            }
            Ok(())
        });
        Self {
            stop: Some(tx),
            task: Some(task),
        }
    }

    pub async fn stop(mut self) -> Result<(), SegmentError> {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
        match self.task.take() {
            Some(t) => t.await.unwrap_or(Ok(())),
            None => Ok(()),
        }
    }
}

impl Drop for SegmentRecorder {
    fn drop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
    }
}
