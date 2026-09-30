//! Client of `aura-worker` (ADR 0005): started on demand, stopped after
//! `idle` without requests unless `keep_warm` is set, restarted once after a
//! crash. OT-001 of 006: this is the only owner of the worker process.

use crate::transcriber::{AsrError, AsrOptions, BoxFut, Segment, Transcriber, Transcript};
use aura_core::jsonrpc::{Peer, RpcError};
use base64::Engine;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::Instant;

pub type SpawnHook = Arc<dyn Fn(u32) + Send + Sync>;

struct Live {
    peer: Peer,
    _child: Child,
    loaded_model: Option<String>,
}

pub struct WorkerClient {
    program: PathBuf,
    idle: Duration,
    live: Mutex<Option<Live>>,
    last_use: Mutex<Instant>,
    keep_warm: AtomicBool,
    spawns: AtomicU64,
    on_spawn: Option<SpawnHook>,
}

impl WorkerClient {
    pub fn new(program: PathBuf, idle: Duration, on_spawn: Option<SpawnHook>) -> Arc<Self> {
        let client = Arc::new(Self {
            program,
            idle,
            live: Mutex::new(None),
            last_use: Mutex::new(Instant::now()),
            keep_warm: AtomicBool::new(false),
            spawns: AtomicU64::new(0),
            on_spawn,
        });
        let weak = Arc::downgrade(&client);
        let tick = (idle / 4).clamp(Duration::from_millis(50), Duration::from_secs(15));
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tick).await;
                let Some(c) = weak.upgrade() else { break };
                if c.keep_warm.load(Ordering::SeqCst) {
                    continue;
                }
                let idle_for = c.last_use.lock().await.elapsed();
                if idle_for >= c.idle {
                    let mut live = c.live.lock().await;
                    if live.is_some() {
                        tracing::info!("stopping idle aura-worker");
                        *live = None; // kill_on_drop
                    }
                }
            }
        });
        client
    }

    pub fn set_keep_warm(&self, warm: bool) {
        self.keep_warm.store(warm, Ordering::SeqCst);
    }

    pub fn spawn_count(&self) -> u64 {
        self.spawns.load(Ordering::SeqCst)
    }

    pub async fn is_running(&self) -> bool {
        self.live
            .lock()
            .await
            .as_ref()
            .is_some_and(|l| !l.peer.is_closed())
    }

    async fn spawn(&self) -> Result<Live, AsrError> {
        let mut cmd = Command::new(&self.program);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true);
        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000);
        let mut child = cmd
            .spawn()
            .map_err(|e| AsrError::LoadFailed(format!("worker: {e}")))?;
        if let (Some(hook), Some(pid)) = (&self.on_spawn, child.id()) {
            hook(pid);
        }
        let stdin = child.stdin.take().ok_or(AsrError::WorkerCrashed)?;
        let stdout = child.stdout.take().ok_or(AsrError::WorkerCrashed)?;
        let (peer, _incoming) = Peer::spawn(stdout, stdin);
        self.spawns.fetch_add(1, Ordering::SeqCst);
        peer.request_timeout("health", json!({}), Duration::from_secs(20))
            .await
            .map_err(|_| AsrError::WorkerCrashed)?;
        Ok(Live {
            peer,
            _child: child,
            loaded_model: None,
        })
    }

    /// Calls a worker method, (re)starting the process as needed. One retry
    /// after a crash.
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, AsrError> {
        *self.last_use.lock().await = Instant::now();
        for attempt in 0..2 {
            let peer = {
                let mut live = self.live.lock().await;
                if live.as_ref().is_none_or(|l| l.peer.is_closed()) {
                    *live = Some(self.spawn().await?);
                }
                live.as_ref().expect("live").peer.clone()
            };
            match peer.request_timeout(method, params.clone(), timeout).await {
                Ok(v) => {
                    *self.last_use.lock().await = Instant::now();
                    return Ok(v);
                }
                Err(RpcError::Closed) if attempt == 0 => {
                    *self.live.lock().await = None;
                    continue;
                }
                Err(RpcError::Timeout) => return Err(AsrError::Timeout),
                Err(RpcError::Remote { message, .. }) => return Err(AsrError::LoadFailed(message)),
                Err(_) => return Err(AsrError::WorkerCrashed),
            }
        }
        Err(AsrError::WorkerCrashed)
    }

    /// Loads a model unless it is already the loaded one.
    pub async fn ensure_model(&self, engine: &str, dir: &std::path::Path) -> Result<(), AsrError> {
        let key = format!("{engine}:{}", dir.display());
        if self
            .live
            .lock()
            .await
            .as_ref()
            .is_some_and(|l| l.loaded_model.as_deref() == Some(key.as_str()) && !l.peer.is_closed())
        {
            return Ok(());
        }
        self.call(
            "asr.load",
            json!({"engine": engine, "modelDir": dir}),
            Duration::from_secs(120),
        )
        .await?;
        if let Some(l) = self.live.lock().await.as_mut() {
            l.loaded_model = Some(key);
        }
        Ok(())
    }

    pub async fn stop(&self) {
        *self.live.lock().await = None;
    }
}

/// Encodes 16 kHz f32 PCM as base64 of little-endian i16 (JSON transport).
pub fn encode_pcm(pcm: &[f32]) -> String {
    let bytes: Vec<u8> = pcm
        .iter()
        .flat_map(|s| ((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes())
        .collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn decode_pcm(b64: &str) -> Option<Vec<f32>> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    Some(
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / i16::MAX as f32)
            .collect(),
    )
}

/// Local transcription through the worker.
pub struct WorkerTranscriber {
    pub client: Arc<WorkerClient>,
    pub engine: String,
    pub model_dir: PathBuf,
    pub model_name: String,
}

impl Transcriber for WorkerTranscriber {
    fn transcribe<'a>(
        &'a self,
        pcm: &'a [f32],
        opts: &'a AsrOptions,
    ) -> BoxFut<'a, Result<Transcript, AsrError>> {
        Box::pin(async move {
            if !self.model_dir.exists() {
                return Err(AsrError::ModelMissing);
            }
            self.client
                .ensure_model(&self.engine, &self.model_dir)
                .await?;
            let timeout = Duration::from_secs(30 + (pcm.len() as u64 / 16_000));
            let v = self
                .client
                .call(
                    "asr.transcribe",
                    json!({"pcm": encode_pcm(pcm), "language": opts.language, "prompt": crate::text::prompt_for_whisper(&opts.vocabulary)}),
                    timeout,
                )
                .await?;
            let mut t: Transcript =
                serde_json::from_value(v).map_err(|e| AsrError::LoadFailed(e.to_string()))?;
            t.text = crate::text::apply_vocabulary(&t.text, &opts.vocabulary);
            for s in &mut t.segments {
                s.text = crate::text::apply_vocabulary(&s.text, &opts.vocabulary);
            }
            Ok(t)
        })
    }
    fn label(&self) -> String {
        self.model_name.clone()
    }
}

#[allow(dead_code)]
fn _assert_segment_serde(s: Segment) -> Value {
    json!(s)
}
