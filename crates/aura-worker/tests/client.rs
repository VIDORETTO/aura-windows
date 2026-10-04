//! `aura-asr::WorkerClient` against the real worker process (fake engine).

use aura_asr::transcriber::{AsrOptions, Transcriber};
use aura_asr::worker_client::{WorkerClient, WorkerTranscriber};
use serde_json::json;
use std::time::Duration;

fn worker() -> std::path::PathBuf {
    env!("CARGO_BIN_EXE_aura-worker").into()
}

#[tokio::test]
async fn transcribes_applies_vocabulary_and_stops_when_idle() {
    let dir = tempfile::tempdir().unwrap();
    let client = WorkerClient::new(worker(), Duration::from_millis(400), None);
    let t = WorkerTranscriber {
        client: client.clone(),
        engine: "fake".into(),
        model_dir: dir.path().into(),
        model_name: "fake".into(),
    };
    let opts = AsrOptions {
        language: Some("pt".into()),
        vocabulary: vec!["Worker".into()],
    };
    let out = t.transcribe(&vec![0.1; 16_000], &opts).await.unwrap();
    assert_eq!(out.text, "olá do Worker");
    assert_eq!(out.segments[0].end_ms, 1000);
    assert!(client.is_running().await);
    // Second call reuses the process and the loaded model.
    t.transcribe(&vec![0.1; 1600], &opts).await.unwrap();
    assert_eq!(client.spawn_count(), 1);
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(!client.is_running().await, "stopped after idle");
    t.transcribe(&vec![0.1; 1600], &opts).await.unwrap();
    assert_eq!(client.spawn_count(), 2);
}

#[tokio::test]
async fn keep_warm_prevents_idle_stop_and_crash_is_retried() {
    let dir = tempfile::tempdir().unwrap();
    let client = WorkerClient::new(worker(), Duration::from_millis(300), None);
    client.set_keep_warm(true);
    client
        .call("health", json!({}), Duration::from_secs(10))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert!(client.is_running().await);
    // The worker exits mid-call; the client restarts it for the next call.
    let _ = client
        .call("debug.exit", json!({}), Duration::from_secs(5))
        .await;
    let t = WorkerTranscriber {
        client: client.clone(),
        engine: "fake".into(),
        model_dir: dir.path().into(),
        model_name: "fake".into(),
    };
    assert!(
        t.transcribe(&vec![0.1; 1600], &AsrOptions::default())
            .await
            .is_ok()
    );
    assert!(client.spawn_count() >= 2);
}

#[test]
fn capabilities_report_cpu_only_without_the_vulkan_feature() {
    assert_eq!(
        aura_asr::worker_client::gpu_inference(&worker()),
        cfg!(feature = "vulkan")
    );
    let missing = std::path::Path::new("does-not-exist/aura-worker.exe");
    assert!(!aura_asr::worker_client::gpu_inference(missing));
}

#[tokio::test]
async fn unknown_engine_reports_a_clear_error() {
    let dir = tempfile::tempdir().unwrap();
    let client = WorkerClient::new(worker(), Duration::from_secs(60), None);
    let t = WorkerTranscriber {
        client,
        engine: "ggml-whisper-x".into(),
        model_dir: dir.path().into(),
        model_name: "x".into(),
    };
    let err = t
        .transcribe(&vec![0.0; 160], &AsrOptions::default())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not available"), "{err}");
}
