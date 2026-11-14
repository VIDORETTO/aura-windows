//! 045 TK-003 / SC-002: cost of the retention after one segment (A-04).
//! Run with `cargo test -p aura-capture --release --test retention_cost -- --ignored --nocapture`.

use aura_capture::retention::{RetentionPolicy, SegmentMeta};
use aura_capture::segments::{SegmentError, SegmentStore};
use aura_store::Store;
use std::sync::Arc;

struct Plain;
impl aura_capture::encoder::Sealer for Plain {
    fn seal(&self, p: &[u8]) -> Result<Vec<u8>, SegmentError> {
        Ok(p.to_vec())
    }
    fn open(&self, s: &[u8]) -> Result<Vec<u8>, SegmentError> {
        Ok(s.to_vec())
    }
}

fn meta(id: &str, start: i64, end: i64) -> SegmentMeta {
    SegmentMeta {
        id: id.into(),
        source: "screen".into(),
        kind: "continuous".into(),
        recording_id: None,
        start_ms: start,
        end_ms: end,
        bytes: 0,
        manual: false,
    }
}

// 045 SC-002: cost of the retention after one segment with 60 000 Continuous
// segments on disk. Run with `--ignored --nocapture` before and after.
#[test]
#[ignore]
fn retention_cost() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("aura.db")).unwrap();
    let segs = SegmentStore::new(store, dir.path().join("segments"), Arc::new(Plain));
    let t = std::time::Instant::now();
    for i in 0..60_000i64 {
        segs.write(
            meta(&format!("c{i}"), i * 10_000, (i + 1) * 10_000),
            b"frames",
        )
        .unwrap();
    }
    eprintln!("carga: {:.1} s", t.elapsed().as_secs_f64());
    let policy = RetentionPolicy::default();
    let now = 60_000 * 10_000;
    let mut ms = Vec::new();
    for _ in 0..7 {
        let t = std::time::Instant::now();
        segs.apply_retention(&policy, now).unwrap();
        ms.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    eprintln!(
        "retenção após um Segmento (7 execuções, ms): {ms:?}; p50 {:.1}",
        ms[3]
    );
}
