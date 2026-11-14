//! Capture pipeline with synthetic sources: policy, redaction, encrypted
//! segments, retention and keyframes.

use aura_capture::clips::sample_indices;
use aura_capture::frame::Frame;
use aura_capture::retention::RetentionPolicy;
use aura_capture::segments::{
    NullEncoder, RecorderConfig, SegmentRecorder, SegmentStore, VaultSealer,
};
use aura_capture::source::{FrameSource, StaticInventory, SyntheticSource, Target};
use aura_capture::{CaptureOutcome, capture_with_policy};
use aura_core::placement::Rect;
use aura_policy::{CaptureMode, Grants, Policy, Requester, WindowInfo};
use aura_store::{StaticKeyProtector, Store, Vault};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn win(id: u64, process: &str, rect: Rect) -> WindowInfo {
    WindowInfo {
        id,
        pid: 1,
        process: process.into(),
        title: String::new(),
        class: String::new(),
        rect,
    }
}

#[test]
fn concurrent_sources_keep_every_retained_segment_readable() {
    use aura_capture::retention::SegmentMeta;
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
    let segments = SegmentStore::new(
        store,
        dir.path().join("audio"),
        Arc::new(VaultSealer(vault)),
    );
    std::thread::scope(|scope| {
        for source in ["mic", "system"] {
            let segments = segments.clone();
            scope.spawn(move || {
                for index in 0..200 {
                    let id = format!("{source}-{index}");
                    segments
                        .write(
                            SegmentMeta {
                                id,
                                source: source.into(),
                                kind: "buffer".into(),
                                recording_id: None,
                                start_ms: 0,
                                end_ms: 1000,
                                bytes: 0,
                                manual: false,
                            },
                            b"retained synthetic audio",
                        )
                        .expect("concurrent write succeeds");
                    segments
                        .apply_retention(&RetentionPolicy::default(), 1000)
                        .unwrap();
                }
            });
        }
    });
    assert_eq!(segments.list(None).unwrap().len(), 400);
    for source in ["mic", "system"] {
        for index in 0..200 {
            assert_eq!(
                segments.read(&format!("{source}-{index}")).unwrap(),
                b"retained synthetic audio"
            );
        }
    }
}

struct Solid;
impl FrameSource for Solid {
    fn capture(&self, _t: &Target) -> Result<Frame, aura_capture::source::CaptureError> {
        Ok(Frame::solid(200, 100, [0, 255, 0, 255]))
    }
}

#[test]
fn explicit_capture_redacts_excluded_windows_and_respects_pause() {
    let inv = StaticInventory(vec![
        win(1, "chrome.exe", Rect::new(0, 0, 100, 100)),
        win(2, "KeePassXC.exe", Rect::new(100, 0, 100, 100)),
    ]);
    let target = Target::Monitor {
        id: "A".into(),
        area: Rect::new(0, 0, 200, 100),
    };
    let out = capture_with_policy(
        &Policy::default(),
        &Grants::default(),
        Requester::User,
        &target,
        &Solid,
        &inv,
    )
    .unwrap();
    let CaptureOutcome::Captured { frame, redacted } = out else {
        panic!("{out:?}")
    };
    assert!(redacted);
    assert_eq!(frame.pixel(10, 10), [0, 255, 0, 255]);
    assert_ne!(frame.pixel(150, 10), [0, 255, 0, 255]);

    let paused = Policy {
        paused: true,
        ..Default::default()
    };
    assert!(matches!(
        capture_with_policy(
            &paused,
            &Grants::default(),
            Requester::User,
            &target,
            &Solid,
            &inv
        )
        .unwrap(),
        CaptureOutcome::Denied { .. }
    ));
    let agent = Requester::Agent {
        tool: "screen_capture".into(),
        conversation: "c".into(),
    };
    assert_eq!(
        capture_with_policy(
            &Policy::default(),
            &Grants::default(),
            agent,
            &target,
            &Solid,
            &inv
        )
        .unwrap(),
        CaptureOutcome::NeedsPermission
    );
}

#[tokio::test(start_paused = true)]
async fn recent_buffer_keeps_last_minutes_encrypted_and_keyframes_are_sampled() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
    let segments = SegmentStore::new(
        store.clone(),
        dir.path().join("captures"),
        Arc::new(VaultSealer(vault)),
    );
    let mut policy = Policy::default();
    policy.screen.mode = CaptureMode::RecentBuffer { minutes: 5 };
    let clock_ms = Arc::new(AtomicI64::new(0));
    let clock = {
        let c = clock_ms.clone();
        Arc::new(move || c.load(Ordering::SeqCst)) as Arc<dyn Fn() -> i64 + Send + Sync>
    };
    let cfg = RecorderConfig {
        policy: Arc::new(Mutex::new(policy)),
        retention: Arc::new(Mutex::new(RetentionPolicy::default())),
        target: Target::Monitor {
            id: "A".into(),
            area: Rect::new(0, 0, 8, 8),
        },
        interval: Duration::from_secs(1),
        kind: "buffer".into(),
        recording_id: None,
    };
    let rec = SegmentRecorder::start(
        cfg,
        Arc::new(SyntheticSource::new(8, 8)),
        Arc::new(StaticInventory(vec![])),
        Box::new(NullEncoder::new(10)),
        segments.clone(),
        clock,
    );
    // 20 simulated minutes at 1 fps.
    for _ in 0..1200 {
        clock_ms.fetch_add(1000, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    rec.stop().await.unwrap();

    let segs = segments.list(Some("screen")).unwrap();
    let span_min = (segs.last().unwrap().end_ms - segs.first().unwrap().start_ms) as f64 / 60_000.0;
    assert!(
        (5.0..=5.5).contains(&span_min),
        "kept {span_min} minutes in {} segments",
        segs.len()
    );
    // Files are sealed: each file starts with the vault magic and does not
    // contain its own plaintext.
    for s in &segs {
        let plain = segments.read(&s.id).unwrap();
        let file = dir
            .path()
            .join("captures/screen")
            .join(format!("{}.seg", s.id));
        let bytes = std::fs::read(file).unwrap();
        assert!(bytes.starts_with(b"AVS1"));
        assert!(!bytes.windows(plain.len()).any(|w| w == plain.as_slice()));
    }
    // Last 2 minutes → 8 keyframes, oldest first.
    let end = segs.last().unwrap().end_ms;
    let mut frames: Vec<i64> = Vec::new();
    for s in segments.range("screen", end - 120_000, end).unwrap() {
        frames.extend(serde_json::from_slice::<Vec<i64>>(&segments.read(&s.id).unwrap()).unwrap());
    }
    let picks: Vec<i64> = sample_indices(frames.len(), 8)
        .into_iter()
        .map(|i| frames[i])
        .collect();
    assert_eq!(picks.len(), 8);
    assert!(picks.windows(2).all(|w| w[0] < w[1]));
}

#[tokio::test(start_paused = true)]
async fn pause_closes_the_segment_and_stops_recording() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
    let segments = SegmentStore::new(
        store,
        dir.path().to_path_buf(),
        Arc::new(VaultSealer(vault)),
    );
    let mut p = Policy::default();
    p.screen.mode = CaptureMode::RecentBuffer { minutes: 5 };
    let policy = Arc::new(Mutex::new(p));
    let t = Arc::new(AtomicI64::new(0));
    let t2 = t.clone();
    let rec = SegmentRecorder::start(
        RecorderConfig {
            policy: policy.clone(),
            retention: Arc::new(Mutex::new(RetentionPolicy::default())),
            target: Target::Monitor {
                id: "A".into(),
                area: Rect::new(0, 0, 4, 4),
            },
            interval: Duration::from_secs(1),
            kind: "buffer".into(),
            recording_id: None,
        },
        Arc::new(SyntheticSource::new(4, 4)),
        Arc::new(StaticInventory(vec![])),
        Box::new(NullEncoder::new(100)),
        segments.clone(),
        Arc::new(move || t2.load(Ordering::SeqCst)),
    );
    for _ in 0..5 {
        t.fetch_add(1000, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    policy.lock().unwrap().paused = true;
    for _ in 0..30 {
        t.fetch_add(1000, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let segs = segments.list(None).unwrap();
    assert_eq!(segs.len(), 1, "open segment flushed on pause");
    let frames: Vec<i64> = serde_json::from_slice(&segments.read(&segs[0].id).unwrap()).unwrap();
    assert!(frames.len() <= 6);
    rec.stop().await.unwrap();
    assert_eq!(
        segments.list(None).unwrap().len(),
        1,
        "nothing recorded while paused"
    );
}

// 045 AC-008: the recorder sweeps orphans when it starts and then hourly.
#[tokio::test(start_paused = true)]
async fn recorder_sweeps_orphans_at_start_and_hourly() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
    let segments = SegmentStore::new(
        store,
        dir.path().to_path_buf(),
        Arc::new(VaultSealer(vault)),
    );
    let screen = dir.path().join("screen");
    std::fs::create_dir_all(&screen).unwrap();
    std::fs::write(screen.join("before.seg"), b"x").unwrap();
    let mut p = Policy::default();
    p.screen.mode = CaptureMode::RecentBuffer { minutes: 5 };
    let t = Arc::new(AtomicI64::new(0));
    let t2 = t.clone();
    let rec = SegmentRecorder::start(
        RecorderConfig {
            policy: Arc::new(Mutex::new(p)),
            retention: Arc::new(Mutex::new(RetentionPolicy::default())),
            target: Target::Monitor {
                id: "A".into(),
                area: Rect::new(0, 0, 4, 4),
            },
            interval: Duration::from_secs(1),
            kind: "buffer".into(),
            recording_id: None,
        },
        Arc::new(SyntheticSource::new(4, 4)),
        Arc::new(StaticInventory(vec![])),
        Box::new(NullEncoder::new(100)),
        segments.clone(),
        Arc::new(move || t2.load(Ordering::SeqCst)),
    );
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert!(!screen.join("before.seg").exists(), "swept at start");
    std::fs::write(screen.join("later.seg"), b"x").unwrap();
    t.store(30 * 60_000, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(screen.join("later.seg").exists(), "no sweep before an hour");
    t.store(60 * 60_000 + 1, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(!screen.join("later.seg").exists(), "swept after an hour");
    rec.stop().await.unwrap();
}

/// Sealer that fails on demand (storage errors vary for real: disk, vault).
struct FlakySealer {
    fail_next: std::sync::atomic::AtomicUsize,
    always: bool,
}
impl aura_capture::encoder::Sealer for FlakySealer {
    fn seal(&self, p: &[u8]) -> Result<Vec<u8>, aura_capture::segments::SegmentError> {
        let fail = self.always
            || self
                .fail_next
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
                .is_ok();
        if fail {
            Err(aura_capture::segments::SegmentError::Seal(
                "injected".into(),
            ))
        } else {
            Ok(p.to_vec())
        }
    }
    fn open(&self, s: &[u8]) -> Result<Vec<u8>, aura_capture::segments::SegmentError> {
        Ok(s.to_vec())
    }
}

fn flaky_recorder(
    dir: &std::path::Path,
    sealer: FlakySealer,
) -> (SegmentRecorder, SegmentStore, Arc<AtomicI64>) {
    let segments = SegmentStore::new(
        Store::open_in_memory().unwrap(),
        dir.to_path_buf(),
        Arc::new(sealer),
    );
    let mut p = Policy::default();
    p.screen.mode = CaptureMode::RecentBuffer { minutes: 5 };
    let t = Arc::new(AtomicI64::new(0));
    let t2 = t.clone();
    let rec = SegmentRecorder::start(
        RecorderConfig {
            policy: Arc::new(Mutex::new(p)),
            retention: Arc::new(Mutex::new(RetentionPolicy::default())),
            target: Target::Monitor {
                id: "A".into(),
                area: Rect::new(0, 0, 4, 4),
            },
            interval: Duration::from_secs(1),
            kind: "buffer".into(),
            recording_id: None,
        },
        Arc::new(SyntheticSource::new(4, 4)),
        Arc::new(StaticInventory(vec![])),
        Box::new(NullEncoder::new(2)),
        segments.clone(),
        Arc::new(move || t2.load(Ordering::SeqCst)),
    );
    (rec, segments, t)
}

// 045 AC-005: one failed segment is logged and recording goes on.
#[tokio::test(start_paused = true)]
async fn one_failed_segment_does_not_stop_the_recorder() {
    let dir = tempfile::tempdir().unwrap();
    let sealer = FlakySealer {
        fail_next: std::sync::atomic::AtomicUsize::new(1),
        always: false,
    };
    let (rec, segments, t) = flaky_recorder(dir.path(), sealer);
    for _ in 0..10 {
        t.fetch_add(1000, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert!(!rec.is_finished(), "recorder still running");
    assert!(
        !segments.list(None).unwrap().is_empty(),
        "later segments stored"
    );
    rec.stop().await.unwrap();
}

// 045 AC-006: five failed segments in a row end the recorder.
#[tokio::test(start_paused = true)]
async fn five_failures_in_a_row_end_the_recorder() {
    let dir = tempfile::tempdir().unwrap();
    let sealer = FlakySealer {
        fail_next: std::sync::atomic::AtomicUsize::new(0),
        always: true,
    };
    let (rec, segments, t) = flaky_recorder(dir.path(), sealer);
    // 2 frames per segment at 1 fps: the 4th failure happens near 8 s.
    for _ in 0..8 {
        t.fetch_add(1000, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert!(!rec.is_finished(), "four failures are tolerated");
    for _ in 0..4 {
        t.fetch_add(1000, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert!(rec.is_finished(), "the fifth failure in a row ends it");
    assert!(segments.list(None).unwrap().is_empty());
    assert!(rec.stop().await.is_err());
}
