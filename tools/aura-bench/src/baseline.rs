//! Baseline comparison: every metric is "lower is better"; a regression is a
//! value worse than `baseline × tolerance` (default 1.2, AC-008 of 010).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const TOLERANCE: f64 = 1.2;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub version: String,
    pub machine: String,
    /// `scenario.metric` → value (ms, MB, %).
    pub metrics: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Regression {
    pub metric: String,
    pub baseline: f64,
    pub latest: f64,
    pub limit: f64,
}

pub fn compare(latest: &Report, baseline: &Report, tolerance: f64) -> Vec<Regression> {
    let mut out = Vec::new();
    for (k, &b) in &baseline.metrics {
        if let Some(&l) = latest.metrics.get(k) {
            let limit = b * tolerance;
            if l > limit {
                out.push(Regression {
                    metric: k.clone(),
                    baseline: b,
                    latest: l,
                    limit,
                });
            }
        }
    }
    out
}

/// Absolute budgets from `docs/architecture/overview.md`.
pub fn budgets() -> BTreeMap<&'static str, f64> {
    BTreeMap::from([
        ("idle.private_ws_mb", 150.0),
        ("idle.cpu_avg_pct", 0.5),
        ("open.p95_ms", 100.0),
        ("first_delta.p95_ms", 50.0),
    ])
}

pub fn over_budget(latest: &Report) -> Vec<(String, f64, f64)> {
    budgets()
        .into_iter()
        .filter_map(|(k, max)| {
            latest
                .metrics
                .get(k)
                .filter(|v| **v > max)
                .map(|v| (k.to_string(), *v, max))
        })
        .collect()
}

pub fn markdown(latest: &Report, baseline: Option<&Report>, regressions: &[Regression]) -> String {
    let mut s = format!(
        "# aura-bench\n\nVersion {} · {}\n\n| Metric | Latest | Baseline | Budget |\n| --- | ---: | ---: | ---: |\n",
        latest.version, latest.machine
    );
    let budgets = budgets();
    for (k, v) in &latest.metrics {
        let b = baseline
            .and_then(|b| b.metrics.get(k))
            .map(|b| format!("{b:.2}"))
            .unwrap_or_else(|| "—".into());
        let budget = budgets
            .get(k.as_str())
            .map(|b| format!("{b:.2}"))
            .unwrap_or_else(|| "—".into());
        s.push_str(&format!("| `{k}` | {v:.2} | {b} | {budget} |\n"));
    }
    if regressions.is_empty() {
        s.push_str("\nNo regressions.\n");
    } else {
        s.push_str("\n## Regressions\n\n");
        for r in regressions {
            s.push_str(&format!(
                "- `{}`: {:.2} > {:.2} (baseline {:.2} × {TOLERANCE})\n",
                r.metric, r.latest, r.limit, r.baseline
            ));
        }
    }
    s
}

/// Nearest-rank percentile of `samples` (unsorted).
pub fn percentile(samples: &[f64], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut v = samples.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let rank = ((p / 100.0) * v.len() as f64).ceil().max(1.0) as usize;
    v[rank.min(v.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(pairs: &[(&str, f64)]) -> Report {
        Report {
            metrics: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn regression_is_worse_than_baseline_times_tolerance() {
        // AC-008 example: baseline 80 → limit 96.
        let base = report(&[("open.p95_ms", 80.0)]);
        let r = compare(&report(&[("open.p95_ms", 100.0)]), &base, TOLERANCE);
        assert_eq!(r.len(), 1);
        assert!((r[0].limit - 96.0).abs() < 1e-9);
        assert!(compare(&report(&[("open.p95_ms", 90.0)]), &base, TOLERANCE).is_empty());
        // Metrics missing on either side are ignored.
        assert!(compare(&report(&[("other", 1e9)]), &base, TOLERANCE).is_empty());
    }

    #[test]
    fn budgets_and_percentiles() {
        let latest = report(&[("idle.private_ws_mb", 151.0), ("open.p95_ms", 99.0)]);
        assert_eq!(
            over_budget(&latest),
            vec![("idle.private_ws_mb".to_string(), 151.0, 150.0)]
        );
        let s: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile(&s, 95.0), 95.0);
        assert_eq!(percentile(&s, 50.0), 50.0);
        assert_eq!(percentile(&[3.0], 95.0), 3.0);
        let md = markdown(&latest, None, &[]);
        assert!(md.contains("| `open.p95_ms` | 99.00 | — | 100.00 |"));
    }
}
