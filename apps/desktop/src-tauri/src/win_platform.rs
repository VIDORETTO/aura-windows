//! The Windows `Platform` for the host, built from `aura-win` adapters.

use aura_app::platform::{Foreground, Platform};
use aura_core::events::PreviousApp;
use aura_core::placement::Rect;
use aura_win::windows_info as wi;
use std::sync::{Arc, Mutex};

/// Tracks the last non-Aura foreground app (the "Aplicativo anterior").
pub struct WinForeground {
    own_pid: u32,
    last: Mutex<Option<PreviousApp>>,
    selection: Mutex<Option<String>>,
}

impl WinForeground {
    pub fn new() -> Self {
        Self {
            own_pid: std::process::id(),
            last: Mutex::new(None),
            selection: Mutex::new(None),
        }
    }
}

impl Foreground for WinForeground {
    fn current(&self) -> Option<PreviousApp> {
        match wi::foreground_app() {
            Some(a) if a.pid != self.own_pid => {
                *self.last.lock().unwrap() = Some(a.clone());
                Some(a)
            }
            _ => self.last.lock().unwrap().clone(),
        }
    }

    fn restore(&self, app: &PreviousApp) -> bool {
        wi::restore_focus(app)
    }

    fn monitor_area(&self, monitor_id: &str) -> Option<Rect> {
        wi::monitors()
            .into_iter()
            .find(|m| m.monitor.id == monitor_id)
            .map(|m| m.area)
    }

    /// Selection of the previous app as last seen by [`Foreground::snapshot`]
    /// (UI Automation only: no synthetic Ctrl+C). Not consumed: the Host keeps
    /// one chip per selection.
    fn selection(&self, max_chars: usize) -> Option<String> {
        self.selection
            .lock()
            .unwrap()
            .clone()
            .map(|s| s.chars().take(max_chars).collect())
    }

    fn paste(&self, text: &str) -> bool {
        aura_win::input::paste_text(text)
    }

    fn set_clipboard(&self, text: &str) -> bool {
        aura_win::input::set_clipboard_text(text)
    }

    fn snapshot(&self) {
        // With Aura itself in front, the focused element is ours: keep what
        // was read from the previous app.
        match wi::foreground_app() {
            Some(a) if a.pid != self.own_pid => {
                *self.last.lock().unwrap() = Some(a);
                *self.selection.lock().unwrap() = aura_win::uia::focused_selection(50_000);
            }
            _ => {}
        }
    }
}

pub fn platform() -> Platform {
    Platform {
        credentials: Arc::new(aura_win::credman::WinCredentialStore),
        protector: Arc::new(aura_store::protect::DpapiProtector),
        frames: Arc::new(aura_win::capture::GdiSource),
        inventory: Arc::new(aura_win::windows_info::Win32Inventory),
        screen_text: Arc::new(aura_win::ocr::WinScreenText),
        audio: Arc::new(aura_win::audio::WasapiSource),
        foreground: Arc::new(WinForeground::new()),
        heavy_ingest: Arc::new(aura_ingest::NoHeavy),
        codec: Arc::new(aura_win::encoder::MfCodec { bitrate: 1_500_000 }),
        speech: Arc::new(WinSpeech),
        media: Arc::new(aura_win::media::MfMediaDecoder),
        os: "windows".into(),
    }
}

/// Hardware facts for the ASR recommendation; worker is the bundled
/// speech worker, asked whether it can run models on a GPU.
pub fn hardware(worker: Option<&std::path::Path>) -> aura_asr::catalog::Hardware {
    let hw = aura_win::system::probe();
    aura_asr::catalog::Hardware {
        ram_mb: hw.ram_mb,
        cpu_cores: hw.cpu_threads,
        gpus: hw
            .gpus
            .into_iter()
            .map(|g| aura_asr::catalog::Gpu {
                vendor: match g.vendor_id {
                    0x10DE => "nvidia",
                    0x1002 | 0x1022 => "amd",
                    0x8086 => "intel",
                    0x5143 => "qualcomm",
                    _ => "other",
                }
                .into(),
                name: g.name,
                dedicated: g.dedicated_vram_mb >= 512,
                vram_mb: g.dedicated_vram_mb,
            })
            .collect(),
        npu: false,
        gpu_inference: worker.is_some_and(aura_asr::worker_client::gpu_inference),
    }
}

pub struct WinDisk;

impl aura_asr::download::DiskSpace for WinDisk {
    fn free_bytes(&self, path: &std::path::Path) -> Option<u64> {
        let mut p = path.to_path_buf();
        // The target may not exist yet; walk up to an existing ancestor.
        while !p.exists() {
            p = p.parent()?.to_path_buf();
        }
        aura_win::system::free_space(&p)
    }
}

/// Process-wide Job Object (kill-on-close) for sidecars.
pub fn child_job() -> Option<&'static aura_win::job::ChildJob> {
    static JOB: std::sync::OnceLock<Option<aura_win::job::ChildJob>> = std::sync::OnceLock::new();
    JOB.get_or_init(|| match aura_win::job::ChildJob::new() {
        Ok(j) => Some(j),
        Err(e) => {
            tracing::warn!("job object unavailable: {e}");
            None
        }
    })
    .as_ref()
}

/// Windows.Media.SpeechSynthesis for "Ler em voz alta".
pub struct WinSpeech;

impl aura_app::speech::Speech for WinSpeech {
    fn voices(&self) -> Vec<aura_app::speech::SpeechVoice> {
        aura_win::speech::voices()
            .into_iter()
            .map(|(id, name, language)| aura_app::speech::SpeechVoice { id, name, language })
            .collect()
    }

    fn synthesize(
        &self,
        text: &str,
        language: &str,
        voice: Option<&str>,
    ) -> Result<(Vec<u8>, String), String> {
        aura_win::speech::synthesize(text, language, voice).map(|b| (b, "audio/wav".to_string()))
    }
}
