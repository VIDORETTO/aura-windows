//! Pure retention planning (AC-013/AC-016 of 004; reused by 005).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentMeta {
    pub id: String,
    pub source: String,
    /// `buffer` or `recording`.
    pub kind: String,
    pub recording_id: Option<String>,
    pub start_ms: i64,
    pub end_ms: i64,
    pub bytes: u64,
    pub manual: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Recent buffer length; buffer segments ending before `now - buffer` go.
    pub buffer_ms: i64,
    /// Continuous recordings: maximum age.
    pub max_age_ms: i64,
    /// Continuous recordings: maximum total bytes.
    pub max_bytes: u64,
    /// Whether manual recordings also follow age/size limits.
    pub apply_to_manual: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            buffer_ms: 5 * 60_000,
            max_age_ms: 7 * 24 * 3_600_000,
            max_bytes: 20 * 1024 * 1024 * 1024,
            apply_to_manual: false,
        }
    }
}

/// Returns ids of segments to delete, oldest first.
pub fn plan(segments: &[SegmentMeta], policy: &RetentionPolicy, now_ms: i64) -> Vec<String> {
    let mut delete: Vec<&SegmentMeta> = segments
        .iter()
        .filter(|s| s.kind == "buffer" && s.end_ms < now_ms - policy.buffer_ms)
        .collect();

    let mut recordings: Vec<&SegmentMeta> = segments
        .iter()
        .filter(|s| s.kind == "recording" && (!s.manual || policy.apply_to_manual))
        .collect();
    recordings.sort_by_key(|s| s.start_ms);
    let mut kept_bytes: u64 = recordings.iter().map(|s| s.bytes).sum();
    for s in recordings {
        let too_old = s.end_ms <= now_ms - policy.max_age_ms;
        if too_old || kept_bytes > policy.max_bytes {
            kept_bytes -= s.bytes;
            delete.push(s);
        }
    }
    delete.sort_by_key(|s| s.start_ms);
    delete.into_iter().map(|s| s.id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(i: i64, kind: &str, len_ms: i64, bytes: u64, manual: bool) -> SegmentMeta {
        SegmentMeta {
            id: format!("{kind}-{i}"),
            source: "screen".into(),
            kind: kind.into(),
            recording_id: None,
            start_ms: i * len_ms,
            end_ms: (i + 1) * len_ms,
            bytes,
            manual,
        }
    }

    #[test]
    fn buffer_keeps_only_the_last_n_minutes() {
        // 10 s segments covering 0..1200 s, now = 1200 s, N = 5 min.
        let segs: Vec<_> = (0..120)
            .map(|i| seg(i, "buffer", 10_000, 1, false))
            .collect();
        let deleted = plan(&segs, &RetentionPolicy::default(), 1_200_000);
        // Segments ending before 900 s go: i in 0..=88 (end = (i+1)*10 s < 900 s).
        assert_eq!(deleted.len(), 89);
        assert_eq!(deleted.first().unwrap(), "buffer-0");
        assert_eq!(deleted.last().unwrap(), "buffer-88");
    }

    #[test]
    fn continuous_keeps_seven_days_and_twenty_gigabytes() {
        const DAY: i64 = 24 * 3_600_000;
        const GB: u64 = 1024 * 1024 * 1024;
        let mut policy = RetentionPolicy::default();
        let two_gb: Vec<_> = (0..10)
            .map(|d| seg(d, "recording", DAY, 2 * GB, false))
            .collect();
        // now = end of day 10: days 0..2 are older than 7 days.
        assert_eq!(
            plan(&two_gb, &policy, 10 * DAY),
            vec!["recording-0", "recording-1", "recording-2"]
        );
        let three_gb: Vec<_> = (0..10)
            .map(|d| seg(d, "recording", DAY, 3 * GB, false))
            .collect();
        // Size: 30 GB → drop oldest until ≤ 20 GB (4 days); age drops 0..2 anyway.
        assert_eq!(
            plan(&three_gb, &policy, 10 * DAY),
            vec!["recording-0", "recording-1", "recording-2", "recording-3"]
        );
        let manual: Vec<_> = (0..10)
            .map(|d| seg(d, "recording", DAY, 3 * GB, true))
            .collect();
        assert!(plan(&manual, &policy, 10 * DAY).is_empty());
        policy.apply_to_manual = true;
        assert_eq!(plan(&manual, &policy, 10 * DAY).len(), 4);
    }
}
