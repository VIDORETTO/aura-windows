//! An app-server that dies during its handshake (e.g. the previous instance is
//! still releasing CODEX_HOME after an idle stop) must not fail the user's
//! message: one more launch is tried (042 round 2).
use aura_codex::fake::{FakeConfig, serve};
use aura_codex::launcher::InMemoryLauncher;
use aura_codex::supervisor::{AppServerSupervisor, SupervisorConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;

#[tokio::test]
async fn a_handshake_that_dies_once_is_retried() {
    let launches = Arc::new(AtomicUsize::new(0));
    let counter = launches.clone();
    let launcher = InMemoryLauncher::new(move |stream| {
        if counter.fetch_add(1, Ordering::SeqCst) == 0 {
            drop(stream); // exits before answering `initialize`
            return;
        }
        tokio::spawn(async move {
            let (r, w) = tokio::io::split(stream);
            serve(r, w, FakeConfig::default()).await;
        });
    });
    let (tx, _rx) = mpsc::unbounded_channel();
    let sup = AppServerSupervisor::new(Arc::new(launcher), SupervisorConfig::default(), tx);
    sup.ensure_running()
        .await
        .expect("second launch answers the handshake");
    assert_eq!(launches.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn a_handshake_that_keeps_dying_still_fails() {
    let launcher = InMemoryLauncher::new(drop);
    let (tx, _rx) = mpsc::unbounded_channel();
    let sup = AppServerSupervisor::new(Arc::new(launcher), SupervisorConfig::default(), tx);
    assert!(sup.ensure_running().await.is_err());
}
