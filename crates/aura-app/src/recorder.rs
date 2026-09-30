//! Background capture (004 TK-005/006, 005 TK-002/004): the privacy policy
//! decides which sources record, into encrypted segments with retention; the
//! agent reads them back through `screen_recent` / `audio_recent`.
//!
//! - Recent buffer / Continuous → recorder always on (kind `buffer`/`continuous`).
//! - Manual → only between `start_manual` and `stop_manual` (kind `recording`).
//! - On demand / Off / paused → nothing recorded (pause closes open segments).

use crate::platform::Platform;
use crate::tools::{LabelledFrames, RecentMedia};
use crate::voice::Voice;
use aura_asr::transcriber::AsrOptions;
use aura_audio::clips::{TimedText, format_transcript, interleave};
use aura_audio::hub::AudioHub;
use aura_audio::recorder::{AudioSegmentWriter, PcmZstd, read_range};
use aura_audio::{AudioSourceKind, DeviceSel};
use aura_capture::clips::{relative_label, sample_indices};
use aura_capture::frame::Frame;
use aura_capture::retention::RetentionPolicy;
use aura_capture::segments::{RecorderConfig, SegmentRecorder, SegmentStore};
use aura_capture::source::Target;
use aura_mcp::BoxFut;
use aura_policy::{CaptureMode, Policy};
use aura_store::{Store, now_secs};
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use aura_capture::encoder::{FRAMES_PER_SEGMENT, NullCodec, VideoCodec};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn background_kind(mode: CaptureMode) -> Option<&'static str> {
    match mode {
        CaptureMode::RecentBuffer { .. } => Some("buffer"),
        CaptureMode::Continuous => Some("continuous"),
        _ => None,
    }
}

struct AudioRun {
    hub: AudioHub,
    stop: Arc<AtomicBool>,
    task: tokio::task::JoinHandle<()>,
    kind: String,
    recording_id: Option<String>,
}

struct ScreenRun {
    recorder: SegmentRecorder,
    kind: String,
    recording_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recording {
    pub id: String,
    pub source: String,
    pub title: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub bytes: u64,
}

pub struct CaptureService {
    platform: Platform,
    codec: Arc<dyn VideoCodec>,
    voice: Arc<Voice>,
    store: Store,
    screen_store: SegmentStore,
    audio_store: SegmentStore,
    policy: Arc<Mutex<Policy>>,
    retention: Arc<Mutex<RetentionPolicy>>,
    screen: tokio::sync::Mutex<Option<ScreenRun>>,
    audio: tokio::sync::Mutex<HashMap<AudioSourceKind, AudioRun>>,
    manual: Mutex<Option<String>>,
    interval: Duration,
}

impl CaptureService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        platform: Platform,
        codec: Arc<dyn VideoCodec>,
        voice: Arc<Voice>,
        store: Store,
        vault: aura_store::vault::Vault,
        screen_root: PathBuf,
        audio_root: PathBuf,
        policy: Policy,
        interval: Duration,
    ) -> Arc<Self> {
        let sealer: Arc<dyn aura_capture::encoder::Sealer> =
            Arc::new(aura_capture::segments::VaultSealer(vault));
        Arc::new(Self {
            platform,
            codec,
            voice,
            screen_store: SegmentStore::new(store.clone(), screen_root, sealer.clone()),
            audio_store: SegmentStore::new(store.clone(), audio_root, sealer),
            store,
            retention: Arc::new(Mutex::new(retention_for(&policy))),
            policy: Arc::new(Mutex::new(policy)),
            screen: tokio::sync::Mutex::new(None),
            audio: tokio::sync::Mutex::new(HashMap::new()),
            manual: Mutex::new(None),
            interval,
        })
    }

    /// Sources currently recording (`screen`, `mic`, `system`).
    pub async fn active(&self) -> Vec<String> {
        let mut v = Vec::new();
        if self.screen.lock().await.is_some() {
            v.push("screen".to_string());
        }
        for k in self.audio.lock().await.keys() {
            v.push(k.key().to_string());
        }
        v.sort();
        v
    }

    fn target(&self) -> Option<Target> {
        let app = self.platform.foreground.current();
        let id = app
            .as_ref()
            .map(|a| a.monitor_id.clone())
            .unwrap_or_default();
        let area = self.platform.foreground.monitor_area(&id)?;
        Some(Target::Monitor { id, area })
    }

    /// Reconciles running recorders with `policy` (+ manual recording).
    pub async fn apply(&self, policy: &Policy) -> Vec<String> {
        *self.policy.lock().unwrap() = policy.clone();
        *self.retention.lock().unwrap() = retention_for(policy);
        let manual = self.manual.lock().unwrap().clone();

        // Screen.
        let want: Option<(String, Option<String>)> = if policy.paused {
            None
        } else if let Some(kind) = background_kind(policy.screen.mode) {
            Some((kind.to_string(), None))
        } else if policy.screen.mode == CaptureMode::Manual {
            manual.clone().map(|id| ("recording".to_string(), Some(id)))
        } else {
            None
        };
        {
            let mut screen = self.screen.lock().await;
            let same = matches!((&*screen, &want), (Some(r), Some((k, id))) if &r.kind == k && &r.recording_id == id);
            if !same {
                if let Some(run) = screen.take()
                    && let Err(e) = run.recorder.stop().await
                {
                    tracing::warn!("screen recorder stop: {e}");
                }
                if let Some((kind, recording_id)) = want {
                    match (self.codec.encoder(), self.target()) {
                        (Ok(encoder), Some(target)) => {
                            let recorder = SegmentRecorder::start(
                                RecorderConfig {
                                    policy: self.policy.clone(),
                                    retention: self.retention.clone(),
                                    target,
                                    interval: self.interval,
                                    kind: kind.clone(),
                                    recording_id: recording_id.clone(),
                                },
                                self.platform.frames.clone(),
                                self.platform.inventory.clone(),
                                encoder,
                                self.screen_store.clone(),
                                Arc::new(now_ms),
                            );
                            *screen = Some(ScreenRun {
                                recorder,
                                kind,
                                recording_id,
                            });
                        }
                        (Err(e), _) => tracing::warn!("screen encoder unavailable: {e}"),
                        (_, None) => tracing::warn!("no monitor to record"),
                    }
                }
            }
        }

        // Audio (mic and system).
        for (kind, sp) in [
            (AudioSourceKind::Mic, policy.mic),
            (AudioSourceKind::SystemAudio, policy.system_audio),
        ] {
            let want: Option<(String, Option<String>)> = if policy.paused {
                None
            } else if let Some(k) = background_kind(sp.mode) {
                Some((k.to_string(), None))
            } else if sp.mode == CaptureMode::Manual {
                manual.clone().map(|id| ("recording".to_string(), Some(id)))
            } else {
                None
            };
            let mut audio = self.audio.lock().await;
            let same = matches!((audio.get(&kind), &want), (Some(r), Some((k, id))) if &r.kind == k && &r.recording_id == id);
            if same {
                continue;
            }
            if let Some(run) = audio.remove(&kind) {
                stop_audio(run).await;
            }
            if let Some((k, recording_id)) = want {
                match self.start_audio(kind, &k, recording_id) {
                    Ok(run) => {
                        audio.insert(kind, run);
                    }
                    Err(e) => tracing::warn!(source = kind.key(), "audio recorder: {e}"),
                }
            }
        }
        self.active().await
    }

    fn start_audio(
        &self,
        kind: AudioSourceKind,
        seg_kind: &str,
        recording_id: Option<String>,
    ) -> Result<AudioRun, String> {
        let hub = AudioHub::start(self.platform.audio.clone(), kind, DeviceSel::Default)
            .map_err(|e| e.to_string())?;
        let mut rx = hub.subscribe();
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let retention = self.retention.lock().unwrap().clone();
        let mut writer = AudioSegmentWriter::new(
            kind.key(),
            seg_kind,
            recording_id.clone(),
            Box::new(PcmZstd),
            self.audio_store.clone(),
            retention,
        );
        let policy = self.policy.clone();
        let task = tokio::task::spawn_blocking(move || {
            loop {
                if stop2.load(Ordering::SeqCst) {
                    break;
                }
                match rx.blocking_recv() {
                    Ok(chunk) => {
                        if policy.lock().unwrap().paused {
                            let _ = writer.flush();
                            continue;
                        }
                        if let Err(e) = writer.push(&chunk) {
                            tracing::warn!("audio segment: {e}");
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            let _ = writer.flush();
        });
        Ok(AudioRun {
            hub,
            stop,
            task,
            kind: seg_kind.into(),
            recording_id,
        })
    }

    /// Starts a manual recording for every source in Manual mode.
    pub async fn start_manual(&self, title: &str, policy: &Policy) -> Result<String, String> {
        if self.manual.lock().unwrap().is_some() {
            return Err("já existe uma gravação em andamento".into());
        }
        let policy = policy.clone();
        let sources: Vec<&str> = [
            ("screen", policy.screen.mode),
            ("mic", policy.mic.mode),
            ("system", policy.system_audio.mode),
        ]
        .into_iter()
        .filter(|(_, m)| *m == CaptureMode::Manual)
        .map(|(s, _)| s)
        .collect();
        if sources.is_empty() {
            return Err("nenhuma fonte está no modo \"Gravação manual\"".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.store
            .with_conn(|c| {
                c.execute(
                    "INSERT INTO recordings(id, source, title, started_at, ended_at, manual) VALUES (?1, ?2, ?3, ?4, NULL, 1)",
                    params![id, sources.join(","), title, now_secs()],
                )?;
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        *self.manual.lock().unwrap() = Some(id.clone());
        self.apply(&policy).await;
        Ok(id)
    }

    pub async fn stop_manual(&self) -> Option<String> {
        let id = self.manual.lock().unwrap().take()?;
        let policy = self.policy.lock().unwrap().clone();
        self.apply(&policy).await;
        let _ = self.store.with_conn(|c| {
            c.execute(
                "UPDATE recordings SET ended_at = ?2 WHERE id = ?1",
                params![id, now_secs()],
            )?;
            Ok(())
        });
        Some(id)
    }

    pub fn manual_active(&self) -> Option<String> {
        self.manual.lock().unwrap().clone()
    }

    pub fn recordings(&self) -> Vec<Recording> {
        let mut rows: Vec<Recording> = self
            .store
            .with_conn(|c| {
                let mut st = c.prepare("SELECT id, source, title, started_at, ended_at FROM recordings ORDER BY started_at DESC")?;
                let rows = st
                    .query_map([], |r| {
                        Ok(Recording { id: r.get(0)?, source: r.get(1)?, title: r.get(2)?, started_at: r.get(3)?, ended_at: r.get(4)?, bytes: 0 })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .unwrap_or_default();
        let all: Vec<_> = self
            .screen_store
            .list(None)
            .unwrap_or_default()
            .into_iter()
            .chain(self.audio_store.list(None).unwrap_or_default())
            .collect();
        for r in &mut rows {
            r.bytes = all
                .iter()
                .filter(|s| s.recording_id.as_deref() == Some(&r.id))
                .map(|s| s.bytes)
                .sum();
        }
        rows
    }

    pub fn delete_recording(&self, id: &str) -> Result<(), String> {
        for store in [&self.screen_store, &self.audio_store] {
            let ids: Vec<String> = store
                .list(None)
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|s| s.recording_id.as_deref() == Some(id))
                .map(|s| s.id)
                .collect();
            store.delete(&ids).map_err(|e| e.to_string())?;
        }
        self.store
            .with_conn(|c| {
                c.execute("DELETE FROM recordings WHERE id = ?1", params![id])?;
                Ok(())
            })
            .map_err(|e| e.to_string())
    }

    /// Decrypts a recording into `dir`: audio as WAV (16 kHz mono), screen
    /// segments as numbered video files.
    pub fn export_recording(&self, id: &str, dir: &Path) -> Result<Vec<PathBuf>, String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for source in ["mic", "system"] {
            let segs: Vec<_> = self
                .audio_store
                .list(Some(source))
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|s| s.recording_id.as_deref() == Some(id))
                .collect();
            let (Some(first), Some(last)) = (segs.first(), segs.last()) else {
                continue;
            };
            let pcm = read_range(
                &self.audio_store,
                &PcmZstd,
                source,
                first.start_ms,
                last.end_ms,
            )
            .map_err(|e| e.to_string())?;
            let path = dir.join(format!("{source}.wav"));
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: 16_000,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            let mut w = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
            for s in pcm {
                w.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                    .map_err(|e| e.to_string())?;
            }
            w.finalize().map_err(|e| e.to_string())?;
            out.push(path);
        }
        let screen: Vec<_> = self
            .screen_store
            .list(Some("screen"))
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|s| s.recording_id.as_deref() == Some(id))
            .collect();
        for (i, seg) in screen.iter().enumerate() {
            let bytes = self.screen_store.read(&seg.id).map_err(|e| e.to_string())?;
            let path = dir.join(format!("screen-{:04}.mp4", i + 1));
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
            out.push(path);
        }
        Ok(out)
    }

    pub async fn shutdown(&self) {
        if let Some(run) = self.screen.lock().await.take() {
            let _ = run.recorder.stop().await;
        }
        for (_, run) in self.audio.lock().await.drain() {
            stop_audio(run).await;
        }
    }
}

async fn stop_audio(run: AudioRun) {
    run.stop.store(true, Ordering::SeqCst);
    let mut hub = run.hub;
    // Joining the device thread blocks; dropping the hub closes the channel
    // so the writer flushes its last segment and exits.
    let _ = tokio::task::spawn_blocking(move || {
        hub.stop();
        drop(hub);
    })
    .await;
    let _ = run.task.await;
}

fn retention_for(p: &Policy) -> RetentionPolicy {
    let minutes = [p.screen.mode, p.mic.mode, p.system_audio.mode]
        .into_iter()
        .filter_map(|m| {
            if let CaptureMode::RecentBuffer { minutes } = m {
                Some(minutes)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(5);
    RetentionPolicy {
        buffer_ms: minutes as i64 * 60_000,
        ..RetentionPolicy::default()
    }
}

impl RecentMedia for CaptureService {
    fn screen_frames<'a>(
        &'a self,
        minutes: f64,
        max_frames: usize,
    ) -> BoxFut<'a, Result<LabelledFrames, String>> {
        Box::pin(async move {
            let now = now_ms();
            let from = now - (minutes * 60_000.0) as i64;
            let segs = self
                .screen_store
                .range("screen", from, now)
                .map_err(|e| e.to_string())?;
            if segs.is_empty() {
                return Err("nada gravado nesse período (o buffer de tela está ligado?)".into());
            }
            let mut frames: Vec<(i64, Frame)> = Vec::new();
            for seg in segs {
                let bytes = self.screen_store.read(&seg.id).map_err(|e| e.to_string())?;
                for (off, f) in self.codec.keyframes(&bytes, max_frames)? {
                    let at = seg.start_ms + off;
                    if at >= from && at <= now {
                        frames.push((at, f));
                    }
                }
            }
            let picked = sample_indices(frames.len(), max_frames);
            Ok(picked
                .into_iter()
                .map(|i| {
                    let (at, f) = &frames[i];
                    (
                        relative_label(*at, now),
                        f.downscale(crate::tools::MAX_IMAGE_SIDE).to_png(),
                    )
                })
                .collect())
        })
    }

    fn audio_transcript<'a>(
        &'a self,
        minutes: f64,
        source: &'a str,
    ) -> BoxFut<'a, Result<String, String>> {
        Box::pin(async move {
            let now = now_ms();
            let from = now - (minutes * 60_000.0) as i64;
            let transcriber = self.voice.transcriber_for_files()?;
            let mut per_source: HashMap<&str, Vec<TimedText>> = HashMap::new();
            let wanted: &[&str] = match source {
                "mic" => &["mic"],
                "system" => &["system"],
                _ => &["mic", "system"],
            };
            for s in wanted {
                let segs = self
                    .audio_store
                    .range(s, from, now)
                    .map_err(|e| e.to_string())?;
                let Some(first) = segs.first() else { continue };
                let start = first.start_ms.max(from);
                let pcm = read_range(&self.audio_store, &PcmZstd, s, from, now)
                    .map_err(|e| e.to_string())?;
                if pcm.is_empty() {
                    continue;
                }
                let t = transcriber
                    .transcribe(&pcm, &AsrOptions::default())
                    .await
                    .map_err(|e| e.to_string())?;
                per_source.insert(
                    s,
                    t.segments
                        .into_iter()
                        .map(|x| TimedText {
                            start_ms: start - from + x.start_ms,
                            end_ms: start - from + x.end_ms,
                            text: x.text,
                        })
                        .collect(),
                );
            }
            if per_source.is_empty() {
                return Err("nada gravado nesse período (o buffer de áudio está ligado?)".into());
            }
            let mic = per_source.remove("mic").unwrap_or_default();
            let sys = per_source.remove("system").unwrap_or_default();
            let lines = interleave(&mic, &sys);
            let label = match source {
                "mic" => "microfone",
                "system" => "áudio do sistema",
                _ => "microfone + áudio do sistema",
            };
            Ok(format_transcript(&lines, now - from, label))
        })
    }
}
