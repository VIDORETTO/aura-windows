//! Recortes: uniform keyframe sampling (AC-012/AC-014 of 004).

/// Picks `k = min(max, n)` indices spread uniformly over `0..n`, always
/// including the first and last frame. `max == 1` returns the last frame.
pub fn sample_indices(n: usize, max: usize) -> Vec<usize> {
    if n == 0 || max == 0 {
        return vec![];
    }
    let k = max.min(n);
    if k == 1 {
        return vec![n - 1];
    }
    (0..k)
        .map(|i| ((i as f64) * ((n - 1) as f64) / ((k - 1) as f64)).round() as usize)
        .collect()
}

/// Label for a keyframe relative to the end of the range, e.g. `t−02:00`.
pub fn relative_label(frame_ms: i64, end_ms: i64) -> String {
    let secs = ((end_ms - frame_ms).max(0) / 1000) as u64;
    format!("t−{:02}:{:02}", secs / 60, secs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_examples() {
        assert_eq!(
            sample_indices(120, 8),
            vec![0, 17, 34, 51, 68, 85, 102, 119]
        );
        assert_eq!(sample_indices(5, 8), vec![0, 1, 2, 3, 4]);
        assert_eq!(sample_indices(10, 1), vec![9]);
        assert_eq!(
            sample_indices(60, 12),
            vec![0, 5, 11, 16, 21, 27, 32, 38, 43, 48, 54, 59]
        );
        assert!(sample_indices(0, 8).is_empty());
    }

    #[test]
    fn labels() {
        assert_eq!(relative_label(0, 120_000), "t−02:00");
        assert_eq!(relative_label(119_000, 120_000), "t−00:01");
    }
}
