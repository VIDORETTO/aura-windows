//! 045 TK-003: retention applied by the segment store (A-04).

use aura_capture::retention::{RetentionPolicy, SegmentMeta, plan};
use aura_capture::segments::{SegmentError, SegmentStore};
use aura_store::Store;
use std::collections::HashSet;
use std::sync::Arc;

/// Identity sealer: the payload size is the stored size.
struct Plain;
impl aura_capture::encoder::Sealer for Plain {
    fn seal(&self, p: &[u8]) -> Result<Vec<u8>, SegmentError> {
        Ok(p.to_vec())
    }
    fn open(&self, s: &[u8]) -> Result<Vec<u8>, SegmentError> {
        Ok(s.to_vec())
    }
}

fn store_in(dir: &std::path::Path, store: Store) -> SegmentStore {
    SegmentStore::new(store, dir.to_path_buf(), Arc::new(Plain))
}

fn meta(
    id: &str,
    source: &str,
    kind: &str,
    rec: Option<&str>,
    start: i64,
    end: i64,
) -> SegmentMeta {
    SegmentMeta {
        id: id.into(),
        source: source.into(),
        kind: kind.into(),
        recording_id: rec.map(Into::into),
        start_ms: start,
        end_ms: end,
        bytes: 0,
        manual: false,
    }
}

/// Small deterministic generator (no new dependency).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

const MIN: i64 = 60_000;
const DAY: i64 = 24 * 60 * MIN;

// 045 AC-007: the store deletes exactly what `plan` says, for any set.
#[test]
fn store_retention_matches_the_pure_plan() {
    for seed in [1u64, 7, 42] {
        for case in 0..4 {
            let dir = tempfile::tempdir().unwrap();
            let store = Store::open_in_memory().unwrap();
            store
                .with_conn(|c| {
                    c.execute_batch(
                        "INSERT INTO recordings(id, source, title, started_at, ended_at, manual) VALUES ('rec-manual','screen','m',0,NULL,1);
                         INSERT INTO recordings(id, source, title, started_at, ended_at, manual) VALUES ('rec-auto','screen','a',0,NULL,0);",
                    )?;
                    Ok(())
                })
                .unwrap();
            let segs = store_in(dir.path(), store);
            let mut rng = Lcg(seed);
            let mut eligible_bytes = 0u64;
            for i in 0..2000 {
                let source = ["screen", "mic", "system"][rng.below(3) as usize];
                let kind = ["buffer", "continuous", "recording"][rng.below(3) as usize];
                let rec = (kind == "recording").then_some(if i % 2 == 0 {
                    "rec-manual"
                } else {
                    "rec-auto"
                });
                let start = rng.below(30 * DAY as u64) as i64;
                let end = start + 10_000 + rng.below(DAY as u64) as i64;
                let size = 100 + rng.below(4900) as usize;
                if kind != "buffer" {
                    eligible_bytes += size as u64;
                }
                segs.write(
                    meta(&format!("s{i}"), source, kind, rec, start, end),
                    &vec![0u8; size],
                )
                .unwrap();
            }
            let policy = match case {
                0 => RetentionPolicy::default(),
                1 => RetentionPolicy {
                    buffer_ms: 60 * MIN,
                    max_age_ms: 3 * DAY,
                    max_bytes: eligible_bytes * 2 / 5,
                    apply_to_manual: false,
                },
                2 => RetentionPolicy {
                    buffer_ms: 5 * MIN,
                    max_age_ms: 20 * DAY,
                    max_bytes: eligible_bytes / 4,
                    apply_to_manual: true,
                },
                _ => RetentionPolicy {
                    buffer_ms: 30 * DAY,
                    max_age_ms: 365 * DAY,
                    max_bytes: 5_000,
                    apply_to_manual: true,
                },
            };
            let now = 30 * DAY;
            let before = segs.list(None).unwrap();
            let expected: HashSet<String> = plan(&before, &policy, now).into_iter().collect();
            let removed = segs.apply_retention(&policy, now).unwrap();
            let after: HashSet<String> =
                segs.list(None).unwrap().into_iter().map(|s| s.id).collect();
            let deleted: HashSet<String> = before
                .iter()
                .map(|s| s.id.clone())
                .filter(|id| !after.contains(id))
                .collect();
            assert_eq!(deleted, expected, "seed {seed} case {case}");
            assert_eq!(removed, expected.len(), "seed {seed} case {case}");
        }
    }
}

// 045 AC-008: orphans are left to the sweep; the sweep removes all kinds.
#[test]
fn orphans_wait_for_the_sweep() {
    let dir = tempfile::tempdir().unwrap();
    let segs = store_in(dir.path(), Store::open_in_memory().unwrap());
    segs.write(meta("kept", "screen", "buffer", None, 0, 10_000), b"x")
        .unwrap();
    segs.write(
        meta("lost-file", "screen", "buffer", None, 10_000, 20_000),
        b"x",
    )
    .unwrap();
    let screen = dir.path().join("screen");
    std::fs::write(screen.join("stray.seg"), b"x").unwrap();
    std::fs::write(screen.join("half.seg.tmp"), b"x").unwrap();
    std::fs::remove_file(screen.join("lost-file.seg")).unwrap();

    let policy = RetentionPolicy::default();
    assert_eq!(segs.apply_retention(&policy, 20_000).unwrap(), 0);
    assert!(screen.join("stray.seg").exists());
    assert!(screen.join("half.seg.tmp").exists());
    assert_eq!(segs.list(None).unwrap().len(), 2);

    assert_eq!(segs.sweep_orphans().unwrap(), 3);
    assert!(!screen.join("stray.seg").exists());
    assert!(!screen.join("half.seg.tmp").exists());
    let ids: Vec<String> = segs.list(None).unwrap().into_iter().map(|s| s.id).collect();
    assert_eq!(ids, vec!["kept"]);
}

// 045 AC-009: range keeps its result.
#[test]
fn range_returns_overlapping_segments() {
    let dir = tempfile::tempdir().unwrap();
    let segs = store_in(dir.path(), Store::open_in_memory().unwrap());
    for i in 0..10i64 {
        segs.write(
            meta(
                &format!("t{i}"),
                "screen",
                "buffer",
                None,
                i * 10_000,
                (i + 1) * 10_000,
            ),
            b"x",
        )
        .unwrap();
        segs.write(
            meta(
                &format!("m{i}"),
                "mic",
                "buffer",
                None,
                i * 10_000,
                (i + 1) * 10_000,
            ),
            b"x",
        )
        .unwrap();
    }
    let starts: Vec<i64> = segs
        .range("screen", 25_000, 55_000)
        .unwrap()
        .into_iter()
        .map(|s| s.start_ms / 1000)
        .collect();
    assert_eq!(starts, vec![20, 30, 40, 50]);
}
