//! Downloader against a local server with Range support and failures.

use aura_asr::catalog::{ModelEntry, ModelFile};
use aura_asr::download::{CancelToken, DiskSpace, DownloadError, Downloader};
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Srv {
    data: Vec<u8>,
    /// Drop the connection after this many bytes on the first request.
    cut_first_at: Option<usize>,
    requests: AtomicUsize,
    ranges: Mutex<Vec<String>>,
}

async fn file(State(s): State<Arc<Srv>>, headers: HeaderMap) -> impl IntoResponse {
    let n = s.requests.fetch_add(1, Ordering::SeqCst);
    let range = headers
        .get("range")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    if let Some(r) = &range {
        s.ranges.lock().unwrap().push(r.clone());
    }
    let start = range
        .as_deref()
        .and_then(|r| r.strip_prefix("bytes="))
        .and_then(|r| r.trim_end_matches('-').parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = s.data[start..].to_vec();
    if n == 0
        && let Some(cut) = s.cut_first_at
    {
        body.truncate(cut);
    }
    let status = if start > 0 {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    (status, body)
}

async fn serve(srv: Arc<Srv>) -> String {
    let app = Router::new().route("/m.bin", get(file)).with_state(srv);
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/m.bin", l.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    url
}

fn model(url: &str, data: &[u8], sha: Option<String>) -> ModelEntry {
    ModelEntry {
        id: "test-model".into(),
        name: "Test".into(),
        family: "whisper".into(),
        engine: "fake".into(),
        description: String::new(),
        languages: vec!["*".into()],
        speed: 1.0,
        accuracy: 1.0,
        streaming: false,
        min_ram_mb: 0,
        gpu_recommended: false,
        license: "MIT".into(),
        source_url: String::new(),
        files: vec![ModelFile {
            path: "m.bin".into(),
            url: url.into(),
            size: data.len() as u64,
            sha256: sha.unwrap_or_else(|| hex::encode(Sha256::digest(data))),
        }],
    }
}

fn data() -> Vec<u8> {
    (0..10 * 1024 * 1024).map(|i| (i % 251) as u8).collect()
}

fn quick(dir: &std::path::Path) -> Downloader {
    let mut d = Downloader::new(dir.to_path_buf());
    d.retry_delay = Duration::from_millis(10);
    d
}

#[tokio::test]
async fn resumes_with_range_after_a_dropped_connection_and_installs_atomically() {
    let srv = Arc::new(Srv {
        data: data(),
        cut_first_at: Some(4 * 1024 * 1024),
        requests: AtomicUsize::new(0),
        ranges: Mutex::new(vec![]),
    });
    let url = serve(srv.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let d = quick(dir.path());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    let out = d
        .install(
            &model(&url, &srv.data, None),
            &CancelToken::default(),
            &move |p| s2.lock().unwrap().push(p.bytes_done),
        )
        .await
        .unwrap();
    assert_eq!(std::fs::read(out.join("m.bin")).unwrap(), srv.data);
    assert_eq!(
        srv.ranges.lock().unwrap().as_slice(),
        &[format!("bytes={}-", 4 * 1024 * 1024)]
    );
    assert!(d.is_installed("test-model"));
    assert!(!dir.path().join(".downloads/test-model").exists());
    assert!(*seen.lock().unwrap().last().unwrap() == srv.data.len() as u64);
    d.remove("test-model").unwrap();
    assert!(!d.is_installed("test-model"));
}

#[tokio::test]
async fn wrong_checksum_installs_nothing() {
    let srv = Arc::new(Srv {
        data: data(),
        cut_first_at: None,
        requests: AtomicUsize::new(0),
        ranges: Mutex::new(vec![]),
    });
    let url = serve(srv.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let err = quick(dir.path())
        .install(
            &model(&url, &srv.data, Some("00".repeat(32))),
            &CancelToken::default(),
            &|_| {},
        )
        .await
        .unwrap_err();
    assert_eq!(
        err,
        DownloadError::Checksum {
            file: "m.bin".into()
        }
    );
    assert!(!dir.path().join("test-model").exists());
}

#[tokio::test]
async fn cancel_removes_partial_files() {
    let srv = Arc::new(Srv {
        data: data(),
        cut_first_at: None,
        requests: AtomicUsize::new(0),
        ranges: Mutex::new(vec![]),
    });
    let url = serve(srv.clone()).await;
    let dir = tempfile::tempdir().unwrap();
    let cancel = CancelToken::default();
    let c2 = cancel.clone();
    let err = quick(dir.path())
        .install(&model(&url, &srv.data, None), &cancel, &move |p| {
            if p.bytes_done > 1024 * 1024 {
                c2.cancel()
            }
        })
        .await
        .unwrap_err();
    assert_eq!(err, DownloadError::Cancelled);
    assert!(!dir.path().join(".downloads/test-model").exists());
}

struct Tight;
impl DiskSpace for Tight {
    fn free_bytes(&self, _p: &std::path::Path) -> Option<u64> {
        Some(500 * 1024 * 1024)
    }
}

#[tokio::test]
async fn insufficient_space_is_reported_with_the_needed_amount() {
    let dir = tempfile::tempdir().unwrap();
    let mut d = quick(dir.path());
    d.space = Arc::new(Tight);
    let mut m = model("http://127.0.0.1:9/x", b"", None);
    m.files[0].size = 456 * 1024 * 1024;
    let err = d
        .install(&m, &CancelToken::default(), &|_| {})
        .await
        .unwrap_err();
    let DownloadError::InsufficientSpace { needed } = err else {
        panic!()
    };
    assert_eq!(needed, (456.0 * 1024.0 * 1024.0 * 1.2) as u64);
}
