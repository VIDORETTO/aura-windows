//! Audio buffer: 1 kHz tone survives encode → seal → store → decode.

use aura_audio::dsp::{sine, tone_power};
use aura_audio::hub::Chunk;
use aura_audio::recorder::{AudioSegmentWriter, PcmZstd, read_range};
use aura_capture::retention::RetentionPolicy;
use aura_capture::segments::{SegmentStore, VaultSealer};
use aura_store::{StaticKeyProtector, Store, Vault};
use std::sync::{Arc, Mutex};

#[test]
fn tone_round_trips_through_sealed_segments_and_retention_applies() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
    let segs = SegmentStore::new(
        store,
        dir.path().to_path_buf(),
        Arc::new(VaultSealer(vault)),
    );
    let retention = RetentionPolicy {
        buffer_ms: 60_000,
        ..Default::default()
    };
    let mut w = AudioSegmentWriter::new(
        "mic",
        "buffer",
        None,
        Box::new(PcmZstd),
        segs.clone(),
        Arc::new(Mutex::new(retention)),
    );
    let tone = sine(1000.0, 16_000, 1.0, 0.5);
    // 3 simulated minutes in 20 ms chunks.
    for i in 0..9000i64 {
        let start = (i as usize * 320) % 16_000;
        let samples: Vec<f32> = (0..320).map(|k| tone[(start + k) % 16_000]).collect();
        w.push(&Chunk {
            samples: samples.into(),
            at_ms: i * 20,
            level_dbfs: -9.0,
        })
        .unwrap();
    }
    w.flush().unwrap();
    let all = segs.list(Some("mic")).unwrap();
    let span = all.last().unwrap().end_ms - all.first().unwrap().start_ms;
    assert!((60_000..=70_000).contains(&span), "kept {span} ms");
    let end = all.last().unwrap().end_ms;
    let audio = read_range(&segs, &PcmZstd, "mic", end - 30_000, end).unwrap();
    assert_eq!(audio.len(), 30 * 16_000);
    assert!(tone_power(&audio, 16_000, 1000.0) > 20.0 * tone_power(&audio, 16_000, 2500.0));
}

// 045 AC-004: a live change of the recent buffer reaches the audio writer and
// the old value no longer deletes screen segments.
#[test]
fn live_retention_change_reaches_audio() {
    use aura_capture::retention::SegmentMeta;
    const MIN: i64 = 60_000;
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open_in_memory().unwrap();
    let vault = Vault::open(&store, &StaticKeyProtector::default()).unwrap();
    let segs = SegmentStore::new(
        store,
        dir.path().to_path_buf(),
        Arc::new(VaultSealer(vault)),
    );
    for m in 0..20 {
        segs.write(
            SegmentMeta {
                id: format!("screen-{m}"),
                source: "screen".into(),
                kind: "buffer".into(),
                recording_id: None,
                start_ms: m * MIN,
                end_ms: (m + 1) * MIN,
                bytes: 0,
                manual: false,
            },
            b"frames",
        )
        .unwrap();
    }
    let policy = Arc::new(Mutex::new(RetentionPolicy {
        buffer_ms: 5 * MIN,
        ..Default::default()
    }));
    let mut w = AudioSegmentWriter::new(
        "mic",
        "buffer",
        None,
        Box::new(PcmZstd),
        segs.clone(),
        policy.clone(),
    );
    let ten_seconds = |w: &mut AudioSegmentWriter, from: i64| {
        for i in 0..500i64 {
            w.push(&Chunk {
                samples: vec![0.0f32; 320].into(),
                at_ms: from + i * 20,
                level_dbfs: -60.0,
            })
            .unwrap();
        }
    };
    policy.lock().unwrap().buffer_ms = 30 * MIN;
    // Closes an audio segment at 20 min: a 30 min buffer keeps every screen one.
    ten_seconds(&mut w, 20 * MIN - 10_000);
    assert_eq!(segs.list(Some("screen")).unwrap().len(), 20);
    policy.lock().unwrap().buffer_ms = 5 * MIN;
    // Closes at 20 min 10 s: screen segments ending before 15 min 10 s go.
    ten_seconds(&mut w, 20 * MIN);
    let kept: Vec<i64> = segs
        .list(Some("screen"))
        .unwrap()
        .into_iter()
        .map(|s| s.end_ms / MIN)
        .collect();
    assert_eq!(kept, vec![16, 17, 18, 19, 20]);
}
