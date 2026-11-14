//! Pure retention planning (AC-013/AC-016 of 004; reused by 005).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentMeta {
    pub id: String,
    pub source: String,
    /// `buffer`, `continuous` or `recording`.
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

/// Whether a segment follows the age and size limits: Continuous segments and
/// recordings (manual ones only when the policy says so).
pub fn is_limited(s: &SegmentMeta, policy: &RetentionPolicy) -> bool {
    s.kind == "continuous" || (s.kind == "recording" && (!s.manual || policy.apply_to_manual))
}

/// Returns ids of segments to delete, oldest first.
pub fn plan(segments: &[SegmentMeta], policy: &RetentionPolicy, now_ms: i64) -> Vec<String> {
    let mut delete: Vec<&SegmentMeta> = segments
        .iter()
        .filter(|s| s.kind == "buffer" && s.end_ms < now_ms - policy.buffer_ms)
        .collect();

    // Continuous segments and limited recordings share one age/size budget.
    let mut recordings: Vec<&SegmentMeta> =
        segments.iter().filter(|s| is_limited(s, policy)).collect();
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

    const DAY: i64 = 24 * 3_600_000;
    const GB: u64 = 1024 * 1024 * 1024;

    fn day(source: &str, kind: &str, d: i64, gb: u64, manual: bool) -> SegmentMeta {
        SegmentMeta {
            id: format!("{source}-{kind}-d{d}"),
            source: source.into(),
            kind: kind.into(),
            recording_id: None,
            start_ms: d * DAY,
            end_ms: (d + 1) * DAY,
            bytes: gb * GB,
            manual,
        }
    }

    // 045 AC-001: the Continuous mode follows the age and size limits.
    #[test]
    fn continuous_segments_follow_age_and_size_limits() {
        let policy = RetentionPolicy::default();
        let two_gb: Vec<_> = (0..10)
            .map(|d| day("screen", "continuous", d, 2, false))
            .collect();
        assert_eq!(
            plan(&two_gb, &policy, 10 * DAY),
            vec![
                "screen-continuous-d0",
                "screen-continuous-d1",
                "screen-continuous-d2"
            ]
        );
        let three_gb: Vec<_> = (0..10)
            .map(|d| day("screen", "continuous", d, 3, false))
            .collect();
        assert_eq!(
            plan(&three_gb, &policy, 10 * DAY),
            vec![
                "screen-continuous-d0",
                "screen-continuous-d1",
                "screen-continuous-d2",
                "screen-continuous-d3"
            ]
        );
    }

    // 045 AC-002: manual recordings (production rows have manual = 1) stay out
    // of the limits unless the policy says so; they do not use the budget.
    #[test]
    fn manual_recordings_stay_outside_the_continuous_budget() {
        let mut policy = RetentionPolicy::default();
        let mut segs: Vec<_> = (0..10)
            .map(|d| day("screen", "recording", d, 3, true))
            .collect();
        segs.extend((5..10).map(|d| day("mic", "continuous", d, 1, false)));
        assert!(plan(&segs, &policy, 10 * DAY).is_empty());
        policy.apply_to_manual = true;
        // 35 GB: age drops recording d0–d2 (26 GB), size drops d3–d4 (20 GB).
        assert_eq!(
            plan(&segs, &policy, 10 * DAY),
            vec![
                "screen-recording-d0",
                "screen-recording-d1",
                "screen-recording-d2",
                "screen-recording-d3",
                "screen-recording-d4"
            ]
        );
    }

    // 045 AC-003: one global budget over every source, oldest start first.
    #[test]
    fn continuous_budget_is_shared_by_all_sources() {
        let policy = RetentionPolicy::default();
        let mut segs: Vec<_> = (0..10)
            .map(|d| day("screen", "continuous", d, 3, false))
            .collect();
        segs.extend((5..10).map(|d| day("mic", "continuous", d, 1, false)));
        // Worked by hand: 35 GB; age drops screen d0–d2 (26 GB); size drops
        // screen d3 (23 GB) and d4 (20 GB); the microphone stays.
        assert_eq!(
            plan(&segs, &policy, 10 * DAY),
            vec![
                "screen-continuous-d0",
                "screen-continuous-d1",
                "screen-continuous-d2",
                "screen-continuous-d3",
                "screen-continuous-d4"
            ]
        );
    }
}
