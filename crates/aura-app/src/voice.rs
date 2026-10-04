//! Voice (006): ASR model catalog and downloads, push-to-talk dictation.
//!
//! The microphone hub runs only while push-to-talk is held (or a recording
//! needs it); audio stays in memory (OT-003). Local transcription goes
//! through `aura-worker`; without a worker (Linux demo) a fake transcriber
//! can be configured.

use crate::events::HostEvent;
use aura_asr::catalog::{Catalog, Hardware, ModelEntry, recommend};
use aura_asr::download::{CancelToken, DiskSpace, Downloader};
use aura_asr::ptt::{PttState, PushToTalk};
use aura_asr::transcriber::{AsrOptions, FakeTranscriber, Transcriber};
use aura_asr::worker_client::{WorkerClient, WorkerTranscriber};
use aura_audio::hub::AudioHub;
use aura_audio::{AudioSource, AudioSourceKind, DeviceSel};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Clone)]
pub enum AsrBackend {
    /// A broken/missing local installation; never synthesize fake results.
    Unavailable,
    /// `aura-worker.exe` next to the app (Windows builds with `engines`).
    Worker {
        program: PathBuf,
        idle: Duration,
        on_spawn: Option<crate::launcher::SpawnHook>,
    },
    /// Deterministic text (tests and the Linux demo).
    Fake { text: String },
}

/// Speech worker state for Diagnostics (010 AC-011).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkerDiag {
    /// `aura-worker.exe` is present next to the app.
    pub installed: bool,
    /// The worker process is running now (it stops when idle).
    pub running: bool,
    pub spawns: u64,
    /// The worker runs speech models on the GPU.
    pub gpu: bool,
    /// Selected speech model.
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelView {
    pub entry: ModelEntry,
    pub size_bytes: u64,
    pub installed: bool,
    pub selected: bool,
    pub recommended: bool,
    pub downloading: bool,
}

pub struct Voice {
    catalog: Catalog,
    downloader: Arc<Downloader>,
    backend: AsrBackend,
    worker: Option<Arc<WorkerClient>>,
    hardware: Hardware,
    selected: RwLock<Option<String>>,
    downloads: Mutex<HashMap<String, CancelToken>>,
    hub: Mutex<Option<AudioHub>>,
    ptt: Mutex<Option<Arc<PushToTalk>>>,
    /// Cloud transcription (BYOK provider): fallback, or primary without a local model.
    cloud: RwLock<Option<Arc<dyn Transcriber>>>,
    audio: Arc<dyn AudioSource>,
    events: broadcast::Sender<HostEvent>,
    /// Runtime captured in `new`: `install`/`ptt` may run off the runtime.
    rt: tokio::runtime::Handle,
}

impl Voice {
    pub fn new(
        models_dir: PathBuf,
        backend: AsrBackend,
        hardware: Hardware,
        disk: Arc<dyn DiskSpace>,
        audio: Arc<dyn AudioSource>,
        events: broadcast::Sender<HostEvent>,
        selected: Option<String>,
    ) -> Self {
        let mut downloader = Downloader::new(models_dir);
        downloader.space = disk;
        let worker = match &backend {
            AsrBackend::Worker {
                program,
                idle,
                on_spawn,
            } => Some(WorkerClient::new(program.clone(), *idle, on_spawn.clone())),
            AsrBackend::Fake { .. } | AsrBackend::Unavailable => None,
        };
        Self {
            catalog: Catalog::builtin(),
            downloader: Arc::new(downloader),
            backend,
            worker,
            hardware,
            selected: RwLock::new(selected),
            downloads: Mutex::new(HashMap::new()),
            hub: Mutex::new(None),
            ptt: Mutex::new(None),
            cloud: RwLock::new(None),
            audio,
            events,
            rt: tokio::runtime::Handle::current(),
        }
    }

    pub fn catalog(&self, ui_lang: &str) -> Vec<ModelView> {
        let rec = recommend(&self.catalog, &self.hardware, ui_lang);
        let selected = self.selected.read().unwrap().clone();
        let downloading = self.downloads.lock().unwrap();
        self.catalog
            .models
            .iter()
            .map(|m| ModelView {
                size_bytes: m.size_bytes(),
                installed: self.downloader.is_installed(&m.id),
                selected: selected.as_deref() == Some(&m.id),
                recommended: rec.as_deref() == Some(&m.id),
                downloading: downloading.contains_key(&m.id),
                entry: m.clone(),
            })
            .collect()
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.read().unwrap().clone()
    }

    pub fn select(&self, id: &str) -> Result<(), String> {
        if !self.downloader.is_installed(id) && !matches!(self.backend, AsrBackend::Fake { .. }) {
            return Err("modelo não instalado".into());
        }
        *self.selected.write().unwrap() = Some(id.to_string());
        *self.ptt.lock().unwrap() = None; // rebuilt with the new model
        Ok(())
    }

    /// Starts a background download; progress arrives as `Download` events.
    pub fn install(self: &Arc<Self>, id: &str) -> Result<(), String> {
        let entry = self.catalog.get(id).cloned().ok_or("modelo desconhecido")?;
        let token = CancelToken::default();
        {
            let mut d = self.downloads.lock().unwrap();
            if d.contains_key(id) {
                return Ok(());
            }
            d.insert(id.to_string(), token.clone());
        }
        let me = self.clone();
        self.rt.spawn(async move {
            let events = me.events.clone();
            let id = entry.id.clone();
            let progress = move |p: aura_asr::download::Progress| {
                let _ = events.send(HostEvent::Download {
                    id: p.model_id,
                    bytes: p.bytes_done,
                    total: Some(p.total),
                    done: false,
                    error: None,
                });
            };
            let result = me.downloader.install(&entry, &token, &progress).await;
            me.downloads.lock().unwrap().remove(&id);
            let (done, error) = match result {
                Ok(_) => (true, None),
                Err(e) => (false, Some(e.to_string())),
            };
            if done && me.selected().is_none() {
                let _ = me.select(&id);
            }
            let _ = me.events.send(HostEvent::Download {
                id,
                bytes: 0,
                total: None,
                done,
                error,
            });
        });
        Ok(())
    }

    pub fn cancel_install(&self, id: &str) {
        if let Some(t) = self.downloads.lock().unwrap().get(id) {
            t.cancel();
        }
    }

    pub fn remove(&self, id: &str) -> Result<(), String> {
        if self.selected().as_deref() == Some(id) {
            *self.selected.write().unwrap() = None;
            *self.ptt.lock().unwrap() = None;
        }
        self.downloader.remove(id).map_err(|e| e.to_string())
    }

    /// Transcriber for attachments (same model as dictation).
    pub fn transcriber_for_files(&self) -> Result<Arc<dyn Transcriber>, String> {
        self.transcriber()
    }

    /// Sets (or clears) the cloud transcriber; takes effect on the next use.
    pub fn set_cloud(&self, cloud: Option<Arc<dyn Transcriber>>) {
        *self.cloud.write().unwrap() = cloud;
        *self.ptt.lock().unwrap() = None;
    }

    fn transcriber(&self) -> Result<Arc<dyn Transcriber>, String> {
        let local = self.local_transcriber();
        let cloud = self.cloud.read().unwrap().clone();
        match (local, cloud) {
            (Ok(local), Some(cloud)) => {
                let events = self.events.clone();
                Ok(Arc::new(aura_asr::transcriber::FallbackTranscriber {
                    primary: local,
                    fallback: Some(cloud),
                    on_fallback: Arc::new(move |e| {
                        let _ = events.send(HostEvent::Notice {
                            level: "info".into(),
                            message: format!("Transcrição local falhou ({e}); usando a nuvem."),
                        });
                    }),
                }))
            }
            (Ok(local), None) => Ok(local),
            (Err(_), Some(cloud)) => Ok(cloud),
            (Err(error), None) => Err(error),
        }
    }

    fn local_transcriber(&self) -> Result<Arc<dyn Transcriber>, String> {
        match &self.backend {
            AsrBackend::Unavailable => Err("worker_missing".into()),
            AsrBackend::Fake { text } => Ok(Arc::new(FakeTranscriber {
                text: text.clone(),
                delay: Duration::from_millis(10),
            })),
            AsrBackend::Worker { .. } => {
                let id = self.selected().ok_or("model_missing")?;
                let entry = self.catalog.get(&id).ok_or("model_missing")?;
                Ok(Arc::new(WorkerTranscriber {
                    client: self.worker.clone().expect("worker backend"),
                    engine: entry.engine.clone(),
                    model_dir: self.downloader.installed_dir(&id),
                    model_name: entry.name.clone(),
                }))
            }
        }
    }

    fn ptt(&self) -> Result<Arc<PushToTalk>, String> {
        let mut guard = self.ptt.lock().unwrap();
        if let Some(p) = guard.as_ref() {
            return Ok(p.clone());
        }
        let p = Arc::new(
            PushToTalk::new(self.transcriber()?).with_partials(Duration::from_millis(1500)),
        );
        let mut states = p.states();
        let events = self.events.clone();
        self.rt.spawn(async move {
            while let Ok(s) = states.recv().await {
                let _ = events.send(HostEvent::Voice(s));
            }
        });
        *guard = Some(p.clone());
        Ok(p)
    }

    /// Key down: opens the microphone (if needed) and starts listening.
    /// Returns the chosen device id when it was missing and the OS default
    /// was opened instead (005 AC-002).
    pub async fn press(&self, device: DeviceSel) -> Result<Option<String>, String> {
        let ptt = self.ptt()?;
        let mut fallback = None;
        let rx = {
            let mut hub = self.hub.lock().unwrap();
            if hub.is_none() {
                let started = AudioHub::start(self.audio.clone(), AudioSourceKind::Mic, device)
                    .map_err(|e| e.to_string())?;
                fallback = started.fallback_from().map(str::to_string);
                *hub = Some(started);
            }
            hub.as_ref().expect("hub").subscribe()
        };
        if let Some(w) = &self.worker {
            // Load the model while the user speaks.
            w.set_keep_warm(true);
        }
        ptt.press(rx).await;
        Ok(fallback)
    }

    fn stop_hub(&self) {
        if let Some(mut h) = self.hub.lock().unwrap().take() {
            h.stop();
        }
        if let Some(w) = &self.worker {
            w.set_keep_warm(false);
        }
    }

    /// Key up: transcribes what was said.
    pub async fn release(&self, opts: AsrOptions) -> PttState {
        let Ok(ptt) = self.ptt() else {
            return PttState::Failed {
                error: "model_missing".into(),
            };
        };
        let state = ptt.release(&opts).await;
        self.stop_hub();
        state
    }

    pub async fn cancel(&self) -> PttState {
        let state = match self.ptt() {
            Ok(p) => p.cancel().await,
            Err(_) => PttState::Cancelled,
        };
        self.stop_hub();
        state
    }

    pub async fn worker_diag(&self) -> WorkerDiag {
        let installed =
            matches!(&self.backend, AsrBackend::Worker { program, .. } if program.exists());
        let (running, spawns) = match &self.worker {
            Some(w) => (w.is_running().await, w.spawn_count()),
            None => (false, 0),
        };
        WorkerDiag {
            installed,
            running,
            spawns,
            gpu: self.hardware.gpu_inference,
            model: self.selected.read().unwrap().clone(),
        }
    }

    pub async fn shutdown(&self) {
        self.stop_hub();
        if let Some(w) = &self.worker {
            w.stop().await;
        }
    }
}
