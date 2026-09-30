//! OS seams: frame sources, window inventory and screen text. Production
//! adapters live in `aura-win`; the synthetic ones drive tests and demos.

use crate::frame::Frame;
use aura_core::placement::Rect;
use aura_policy::WindowInfo;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Monitor { id: String, area: Rect },
    Window { window: u64 },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CaptureError {
    #[error("the window is minimized")]
    WindowMinimized,
    #[error("the window no longer exists")]
    WindowGone,
    #[error("screen capture is not supported on this system")]
    Unsupported,
    #[error("hardware video encoder unavailable")]
    EncoderUnavailable,
    #[error("os error: {0}")]
    Os(String),
}

pub trait FrameSource: Send + Sync {
    fn capture(&self, target: &Target) -> Result<Frame, CaptureError>;
}

pub trait WindowInventory: Send + Sync {
    /// Visible top-level windows intersecting `area`, front to back.
    fn visible_windows(&self, area: Rect) -> Vec<WindowInfo>;
    fn window(&self, id: u64) -> Option<WindowInfo>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenTextResult {
    pub text: String,
    /// `uia` or `ocr`.
    pub origin: String,
}

pub trait ScreenText: Send + Sync {
    fn uia_text(&self, window: u64, max_chars: usize) -> Option<String>;
    fn ocr(&self, frame: &Frame, language: &str) -> Result<String, CaptureError>;
}

/// Reads text with UIA first and OCR as fallback (AC-011 of 004).
pub fn read_screen_text(
    text: &dyn ScreenText,
    source: &dyn FrameSource,
    window: u64,
    mode: &str,
    max_chars: usize,
    language: &str,
) -> Result<ScreenTextResult, CaptureError> {
    if mode != "ocr" {
        if let Some(t) = text
            .uia_text(window, max_chars)
            .filter(|t| !t.trim().is_empty())
        {
            return Ok(ScreenTextResult {
                text: truncate(&t, max_chars),
                origin: "uia".into(),
            });
        }
        if mode == "uia" {
            return Ok(ScreenTextResult {
                text: String::new(),
                origin: "uia".into(),
            });
        }
    }
    let frame = source.capture(&Target::Window { window })?;
    let t = text.ocr(&frame, language)?;
    Ok(ScreenTextResult {
        text: truncate(&t, max_chars),
        origin: "ocr".into(),
    })
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Deterministic frames for tests: frame `n` has every pixel set to
/// `[n % 256, n / 256 % 256, 0x42, 0xff]` so keyframe tests can read `n` back.
pub struct SyntheticSource {
    pub width: u32,
    pub height: u32,
    counter: Arc<AtomicI64>,
}

impl SyntheticSource {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            counter: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn number_of(frame: &Frame) -> i64 {
        let p = frame.pixel(0, 0);
        p[0] as i64 + (p[1] as i64) * 256
    }
}

impl FrameSource for SyntheticSource {
    fn capture(&self, _target: &Target) -> Result<Frame, CaptureError> {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        let mut f = Frame::solid(
            self.width,
            self.height,
            [(n % 256) as u8, ((n / 256) % 256) as u8, 0x42, 0xff],
        );
        f.captured_at_ms = n * 1000;
        Ok(f)
    }
}

/// Inventory from a fixed list (tests, Linux builds).
pub struct StaticInventory(pub Vec<WindowInfo>);

impl WindowInventory for StaticInventory {
    fn visible_windows(&self, area: Rect) -> Vec<WindowInfo> {
        self.0
            .iter()
            .filter(|w| w.rect.intersect(&area).is_some())
            .cloned()
            .collect()
    }
    fn window(&self, id: u64) -> Option<WindowInfo> {
        self.0.iter().find(|w| w.id == id).cloned()
    }
}
