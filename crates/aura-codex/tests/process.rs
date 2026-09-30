//! Exercises the production launcher path: a real child process over stdio.

use aura_codex::events::{ConversationEvent as E, TurnStatus};
use aura_codex::launcher::ProcessLauncher;
use aura_codex::service::{CodexService, StartOptions, TurnOptions};
use aura_codex::supervisor::{AppServerSupervisor, SupervisorConfig};
use aura_core::context::TurnInput;
use aura_store::Store;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[tokio::test]
async fn real_process_streams_a_turn_and_stops_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let spawned = Arc::new(AtomicU32::new(0));
    let seen = spawned.clone();
    let mut launcher = ProcessLauncher::app_server(
        env!("CARGO_BIN_EXE_aura-fake-codex").into(),
        dir.path().join("codex-home"),
        vec![("AURA_FAKE_CODEX_DELAY_MS".into(), "0".into())],
    );
    launcher.on_spawn = Some(Arc::new(move |pid| seen.store(pid, Ordering::SeqCst)));
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let sup = AppServerSupervisor::new(Arc::new(launcher), SupervisorConfig::default(), tx);
    let svc = CodexService::new(
        sup,
        rx,
        Store::open_in_memory().unwrap(),
        dir.path().join("ws"),
    );
    let mut events = svc.events();

    let conv = svc.start(StartOptions::default()).await.unwrap();
    assert!(
        spawned.load(Ordering::SeqCst) > 0,
        "on_spawn hook receives the PID"
    );
    svc.send(
        &conv.thread_id,
        &[TurnInput::Text { text: "oi".into() }],
        TurnOptions::default(),
    )
    .await
    .unwrap();
    let status = loop {
        match tokio::time::timeout(Duration::from_secs(20), events.recv())
            .await
            .unwrap()
            .unwrap()
        {
            E::TurnCompleted { status, .. } => break status,
            _ => continue,
        }
    };
    assert_eq!(status, TurnStatus::Completed);
    svc.shutdown().await;
}
