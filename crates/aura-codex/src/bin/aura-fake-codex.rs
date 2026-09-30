//! Fake Codex app-server over stdio for UI development and E2E tests.
//!
//! Usage: set `AURA_CODEX_BIN` to this executable. Optional environment:
//! `AURA_FAKE_CODEX_RECORD=<file>` appends every received message as JSONL;
//! `AURA_FAKE_CODEX_DELAY_MS=<n>` delays streamed deltas;
//! `AURA_FAKE_CODEX_STATE=<file>` keeps conversations across restarts.

use aura_codex::fake::{FakeConfig, serve};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Codex is invoked as `<bin> app-server`; accept and ignore arguments.
    let record = std::env::var("AURA_FAKE_CODEX_RECORD").ok();
    let sink = Arc::new(Mutex::new(Vec::new()));
    let state = std::env::var("AURA_FAKE_CODEX_STATE")
        .map(|p| aura_codex::fake::FakeState::with_file(p.into()))
        .unwrap_or_default();
    let cfg = FakeConfig {
        state,
        crash_after_turn_start: std::env::var("AURA_FAKE_CODEX_CRASH").is_ok(),
        record: record.as_ref().map(|_| sink.clone()),
        delta_delay: Duration::from_millis(
            std::env::var("AURA_FAKE_CODEX_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        ),
    };
    serve(tokio::io::stdin(), tokio::io::stdout(), cfg).await;
    if let Some(path) = record
        && let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    {
        for line in sink.lock().unwrap().iter() {
            let _ = writeln!(f, "{line}");
        }
    }
}
