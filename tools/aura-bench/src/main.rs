//! `aura-bench` — performance budgets (010 TK-004).
//!
//! ```text
//! aura-bench all [--pid <aura.exe pid>] [--idle-secs 300] [--opens 50]
//!                [--out bench/latest.json] [--baseline bench/baseline.json]
//! aura-bench compare <latest.json> <baseline.json>
//! ```
//! Exit code 1 on a regression (> baseline × 1.2) or a blown absolute budget.

mod baseline;
#[cfg(feature = "host")]
mod portable;
#[cfg(windows)]
mod windows_probe;

use baseline::{Report, TOLERANCE, compare, markdown, over_budget};
use std::path::PathBuf;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn load(path: &str) -> Report {
    serde_json::from_slice(&std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}")))
        .expect("valid report")
}

fn finish(latest: &Report, baseline: Option<&Report>, out: Option<PathBuf>) -> i32 {
    let regressions = baseline
        .map(|b| compare(latest, b, TOLERANCE))
        .unwrap_or_default();
    let md = markdown(latest, baseline, &regressions);
    println!("{md}");
    if let Some(out) = out {
        if let Some(dir) = out.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        std::fs::write(&out, serde_json::to_vec_pretty(latest).expect("json"))
            .expect("write report");
        std::fs::write(out.with_extension("md"), &md).expect("write markdown");
    }
    let blown = over_budget(latest);
    for (k, v, max) in &blown {
        eprintln!("over budget: {k} = {v:.2} > {max:.2}");
    }
    i32::from(!regressions.is_empty() || !blown.is_empty())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("compare") if args.len() >= 3 => {
            let latest = load(&args[1]);
            let base = load(&args[2]);
            finish(&latest, Some(&base), None)
        }
        Some("all") => {
            let mut report = Report {
                version: env!("CARGO_PKG_VERSION").into(),
                machine: format!(
                    "{} {} · {} threads",
                    std::env::consts::OS,
                    std::env::consts::ARCH,
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(1)
                ),
                metrics: Default::default(),
            };
            #[cfg(feature = "host")]
            {
                let rt = tokio::runtime::Runtime::new().expect("runtime");
                report
                    .metrics
                    .extend(rt.block_on(portable::first_delta(30)));
                report.metrics.extend(portable::capture_pipeline(20));
            }
            #[cfg(windows)]
            if let Some(pid) = arg(&args, "--pid").and_then(|p| p.parse().ok()) {
                let secs = arg(&args, "--idle-secs")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300);
                report.metrics.extend(windows_probe::idle(pid, secs));
                let opens = arg(&args, "--opens")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(50);
                report.metrics.extend(windows_probe::open_latency(opens));
            }
            let base = arg(&args, "--baseline")
                .filter(|p| std::path::Path::new(p).exists())
                .map(|p| load(&p));
            finish(
                &report,
                base.as_ref(),
                arg(&args, "--out").map(PathBuf::from),
            )
        }
        _ => {
            eprintln!(
                "usage: aura-bench all [--pid N] [--idle-secs S] [--opens N] [--out FILE] [--baseline FILE]\n       aura-bench compare LATEST BASELINE"
            );
            2
        }
    };
    std::process::exit(code);
}
