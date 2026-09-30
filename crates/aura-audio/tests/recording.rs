//! Audio buffer: 1 kHz tone survives encode → seal → store → decode.

use aura_audio::dsp::{sine, tone_power};
use aura_audio::hub::Chunk;
use aura_audio::recorder::{AudioSegmentWriter, PcmZstd, read_range};
use aura_capture::retention::RetentionPolicy;
use aura_capture::segments::{SegmentStore, VaultSealer};
use aura_store::{StaticKeyProtector, Store, Vault};
use std::sync::Arc;

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
        retention,
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
