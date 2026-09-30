//! Scenarios that run on any OS against the real host with the fake
//! app-server: time to first streamed delta and the capture pipeline cost.

use crate::baseline::percentile;
use aura_app::events::HostEvent;
use aura_app::host::SendRequest;
use aura_app::paths::AppPaths;
use aura_app::platform::Platform;
use aura_app::{Host, HostConfig};
use aura_codex::events::ConversationEvent;
use aura_codex::service::StartOptions;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

pub async fn first_delta(turns: usize) -> BTreeMap<String, f64> {
    let dir = std::env::temp_dir().join(format!("aura-bench-{}", std::process::id()));
    let (platform, _fg) = Platform::fake();
    let mut cfg = HostConfig::demo(AppPaths::new(&dir), platform);
    cfg.in_memory_store = true;
    let host = Host::start(cfg).await.expect("host");
    let conv = host
        .start_conversation(StartOptions::default())
        .await
        .expect("conversation");
    let mut rx = host.subscribe();
    let mut samples = Vec::new();
    for i in 0..turns {
        let t0 = Instant::now();
        host.send(SendRequest {
            thread_id: conv.thread_id.clone(),
            text: format!("pergunta {i}"),
            tray: conv.thread_id.clone(),
            accepts_images: true,
            options: Default::default(),
        })
        .await
        .expect("send");
        let mut first = None;
        loop {
            match tokio::time::timeout(Duration::from_secs(20), rx.recv()).await {
                Ok(Ok(HostEvent::Conversation(ConversationEvent::MessageDelta { .. })))
                    if first.is_none() =>
                {
                    first = Some(t0.elapsed().as_secs_f64() * 1000.0);
                }
                Ok(Ok(HostEvent::Conversation(ConversationEvent::TurnCompleted { .. }))) => break,
                Ok(Ok(_)) | Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => {}
                _ => break,
            }
        }
        if let Some(ms) = first {
            samples.push(ms);
        }
    }
    host.shutdown().await;
    let _ = std::fs::remove_dir_all(&dir);
    BTreeMap::from([
        ("first_delta.p50_ms".to_string(), percentile(&samples, 50.0)),
        ("first_delta.p95_ms".to_string(), percentile(&samples, 95.0)),
    ])
}

/// Redaction + downscale + PNG of a 1920×1080 frame (the explicit capture path).
pub fn capture_pipeline(iterations: usize) -> BTreeMap<String, f64> {
    use aura_capture::frame::{Frame, redact};
    use aura_core::placement::Rect;
    let frame = Frame::solid(1920, 1080, [40, 80, 120, 255]);
    let mut samples = Vec::new();
    for _ in 0..iterations {
        let t0 = Instant::now();
        let mut f = frame.clone();
        redact(&mut f, &[Rect::new(100, 100, 600, 400)]);
        let small = f.downscale(1568);
        let png = small.to_png();
        std::hint::black_box(png);
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    BTreeMap::from([("capture.p95_ms".to_string(), percentile(&samples, 95.0))])
}
