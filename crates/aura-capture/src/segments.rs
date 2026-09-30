//! Encrypted segment storage and the background recorder (ADR 0004).
//!
//! Frames → privacy decision + redaction → [`VideoEncoder`] (H.264 fMP4 in
//! memory on Windows) → [`Sealer`] (vault, key `capture-segment`) → `.seg`
//! file + index row. Plaintext never touches the disk (OT-003).

use crate::frame::redact;
use crate::retention::{RetentionPolicy, SegmentMeta, plan};
use crate::source::{FrameSource, Target, WindowInventory};
use aura_policy::{AccessRequest, Decision, Grants, Policy, Requester, Source, decide};
use aura_store::Store;
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use crate::encoder::{NullEncoder, Sealer, SegmentError, VideoEncoder};

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
}

impl SegmentStore {
    pub fn new(store: Store, root: PathBuf, sealer: Arc<dyn Sealer>) -> Self {
        Self {
            store,
            root,
            sealer,
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
            let sql = "SELECT s.id, s.source, s.kind, s.recording_id, s.start_ms, s.end_ms, s.bytes, COALESCE(r.manual, 0)
                       FROM capture_segments s LEFT JOIN recordings r ON r.id = s.recording_id
                       WHERE (?1 IS NULL OR s.source = ?1) ORDER BY s.start_ms";
            let mut stmt = c.prepare(sql)?;
            let rows = stmt.query_map([source], |r| {
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
            })?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })?)
    }

    pub fn range(
        &self,
        source: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<SegmentMeta>, SegmentError> {
        Ok(self
            .list(Some(source))?
            .into_iter()
            .filter(|s| s.end_ms > from_ms && s.start_ms < to_ms)
            .collect())
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

    pub fn delete(&self, ids: &[String]) -> Result<(), SegmentError> {
        for id in ids {
            let path: Option<String> = self.store.with_conn(|c| {
                Ok(rusqlite::OptionalExtension::optional(c.query_row(
                    "SELECT path FROM capture_segments WHERE id = ?1",
                    [id],
                    |r| r.get(0),
                ))?)
            })?;
            if let Some(p) = path {
                let _ = std::fs::remove_file(p);
            }
            self.store.with_conn(|c| {
                c.execute("DELETE FROM capture_segments WHERE id = ?1", [id])?;
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Applies retention and removes orphans (files without rows and rows
    /// without files). Run on start and after each closed segment.
    pub fn apply_retention(
        &self,
        policy: &RetentionPolicy,
        now_ms: i64,
    ) -> Result<usize, SegmentError> {
        let all = self.list(None)?;
        let doomed = plan(&all, policy, now_ms);
        self.delete(&doomed)?;
        let mut removed = doomed.len();
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
