//! OS seams bundled for the host. The Tauri shell builds a Windows
//! [`Platform`] from `aura-win`; tests and the Linux demo use [`Platform::fake`].

use aura_audio::AudioSource;
use aura_capture::frame::Frame;
use aura_capture::source::{
    CaptureError, FrameSource, ScreenText, StaticInventory, SyntheticSource, WindowInventory,
};
use aura_core::credentials::{CredentialStore, MemoryCredentialStore};
use aura_core::events::PreviousApp;
use aura_core::placement::Rect;
use aura_ingest::{HeavyIngestor, NoHeavy};
use aura_policy::WindowInfo;
use aura_store::protect::{SecretProtector, StaticKeyProtector};
use std::sync::{Arc, Mutex};

/// The app the user was in before the Overlay (and a way back to it).
pub trait Foreground: Send + Sync {
    fn current(&self) -> Option<PreviousApp>;
    fn restore(&self, app: &PreviousApp) -> bool;
    /// Full-monitor capture area of `monitor_id` (physical pixels).
    fn monitor_area(&self, monitor_id: &str) -> Option<Rect>;
    /// Text selected in the foreground app (UIA, then clipboard).
    fn selection(&self, max_chars: usize) -> Option<String>;
    /// Pastes text into the focused control of the foreground app.
    fn paste(&self, text: &str) -> bool;
    /// Puts text on the clipboard ("copy text from the screen", 020).
    fn set_clipboard(&self, _text: &str) -> bool {
        false
    }
    /// Browser URL of the window when available (UIA address bar).
    fn url_of(&self, _window: u64) -> Option<String> {
        None
    }
    /// Called right before the Overlay shows: remember the foreground app
    /// (and its selected text) while it is still in front.
    fn snapshot(&self) {}
}

#[derive(Clone)]
pub struct Platform {
    pub credentials: Arc<dyn CredentialStore>,
    pub protector: Arc<dyn SecretProtector>,
    pub frames: Arc<dyn FrameSource>,
    pub inventory: Arc<dyn WindowInventory>,
    pub screen_text: Arc<dyn ScreenText>,
    pub audio: Arc<dyn AudioSource>,
    pub foreground: Arc<dyn Foreground>,
    pub heavy_ingest: Arc<dyn HeavyIngestor>,
    /// Audio/video file decoding for attachments.
    pub media: Arc<dyn aura_capture::encoder::MediaFileDecoder>,
    /// Text-to-speech for "Ler em voz alta".
    pub speech: Arc<dyn crate::speech::Speech>,
    /// Screen segment encoder/decoder (Media Foundation on Windows).
    pub codec: Arc<dyn aura_capture::encoder::VideoCodec>,
    /// OS name shown in diagnostics.
    pub os: String,
}

/// A desk with one "editor" window, for tests and the Linux demo.
pub struct FakeForeground {
    pub app: Mutex<Option<PreviousApp>>,
    pub area: Rect,
    pub selection: Mutex<Option<String>>,
    pub pasted: Mutex<Vec<String>>,
    pub clipboard: Mutex<Option<String>>,
}

impl Foreground for FakeForeground {
    fn current(&self) -> Option<PreviousApp> {
        self.app.lock().unwrap().clone()
    }
    fn restore(&self, _app: &PreviousApp) -> bool {
        true
    }
    fn monitor_area(&self, _monitor_id: &str) -> Option<Rect> {
        Some(self.area)
    }
    fn selection(&self, _max: usize) -> Option<String> {
        self.selection.lock().unwrap().clone()
    }
    fn paste(&self, text: &str) -> bool {
        self.pasted.lock().unwrap().push(text.to_string());
        true
    }
    fn set_clipboard(&self, text: &str) -> bool {
        *self.clipboard.lock().unwrap() = Some(text.to_string());
        true
    }
}

pub struct FakeScreenText {
    pub text: String,
}

impl ScreenText for FakeScreenText {
    fn uia_text(&self, _window: u64, max_chars: usize) -> Option<String> {
        Some(self.text.chars().take(max_chars).collect())
    }
    fn ocr(&self, _frame: &Frame, _language: &str) -> Result<String, CaptureError> {
        Ok(self.text.clone())
    }
}

pub const FAKE_WINDOW: u64 = 0x1001;

impl Platform {
    pub fn fake() -> (Self, Arc<FakeForeground>) {
        let area = Rect::new(0, 0, 1280, 800);
        let editor = WindowInfo {
            id: FAKE_WINDOW,
            pid: 4242,
            process: "Code.exe".into(),
            title: "main.rs — Aura".into(),
            class: "Chrome_WidgetWin_1".into(),
            rect: Rect::new(0, 0, 1280, 760),
        };
        let fg = Arc::new(FakeForeground {
            app: Mutex::new(Some(PreviousApp {
                window: FAKE_WINDOW,
                pid: 4242,
                process_name: "Code.exe".into(),
                title: "main.rs — Aura".into(),
                monitor_id: "\\\\.\\DISPLAY1".into(),
            })),
            area,
            selection: Mutex::new(None),
            pasted: Mutex::new(Vec::new()),
            clipboard: Mutex::new(None),
        });
        let platform = Self {
            credentials: Arc::new(MemoryCredentialStore::default()),
            protector: Arc::new(StaticKeyProtector::new([7u8; 32])),
            frames: Arc::new(SyntheticSource::new(320, 200)),
            inventory: Arc::new(StaticInventory(vec![editor])),
            screen_text: Arc::new(FakeScreenText {
                text: "error[E0308]: mismatched types".into(),
            }),
            audio: Arc::new(aura_audio::hub::SyntheticAudio {
                freq: 440.0,
                amplitude: 0.2,
                total_ms: 60_000,
                realtime: true,
                fail_ids: vec![],
            }),
            foreground: fg.clone(),
            heavy_ingest: Arc::new(NoHeavy),
            codec: Arc::new(aura_capture::encoder::NullCodec),
            speech: Arc::new(crate::speech::FakeSpeech),
            media: Arc::new(FakeMedia),
            os: std::env::consts::OS.into(),
        };
        (platform, fg)
    }
}

/// Test decoder: any `.mp3/.m4a` is 2 s of tone; any `.mp4` has 3 frames + tone.
pub struct FakeMedia;

impl aura_capture::encoder::MediaFileDecoder for FakeMedia {
    fn audio_16k(&self, _path: &std::path::Path) -> Result<Vec<f32>, String> {
        Ok(aura_audio::dsp::sine(300.0, 16_000, 2.0, 0.3))
    }
    fn video_frames(
        &self,
        path: &std::path::Path,
        max: usize,
    ) -> Result<Vec<(i64, Frame)>, String> {
        if !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("mp4"))
        {
            return Err("sem vídeo".into());
        }
        Ok((0..3.min(max) as i64)
            .map(|i| (i * 1000, Frame::solid(64, 36, [i as u8 * 60, 80, 120, 255])))
            .collect())
    }
}
