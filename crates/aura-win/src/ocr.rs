//! OCR with `Windows.Media.Ocr` (on-device, uses installed language packs)
//! and the `ScreenText` adapter combining it with UI Automation.

use crate::uia;
use aura_capture::frame::Frame;
use aura_capture::source::{CaptureError, ScreenText};
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Security::Cryptography::CryptographicBuffer;
use windows::core::HSTRING;

fn os(e: windows::core::Error) -> CaptureError {
    CaptureError::Os(e.message().to_string())
}

/// Languages with an installed OCR pack (BCP-47 tags).
pub fn available_languages() -> Vec<String> {
    OcrEngine::AvailableRecognizerLanguages()
        .map(|langs| {
            langs
                .into_iter()
                .filter_map(|l| l.LanguageTag().ok())
                .map(|t| t.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn engine(language: &str) -> Result<OcrEngine, CaptureError> {
    if !language.is_empty()
        && let Ok(lang) = Language::CreateLanguage(&HSTRING::from(language))
        && OcrEngine::IsLanguageSupported(&lang).unwrap_or(false)
        && let Ok(e) = OcrEngine::TryCreateFromLanguage(&lang)
    {
        return Ok(e);
    }
    OcrEngine::TryCreateFromUserProfileLanguages().map_err(|_| CaptureError::Unsupported)
}

pub fn recognize(frame: &Frame, language: &str) -> Result<String, CaptureError> {
    let engine = engine(language)?;
    let max = OcrEngine::MaxImageDimension().map_err(os)?;
    let frame = if frame.width.max(frame.height) > max {
        frame.downscale(max)
    } else {
        frame.clone()
    };
    let buffer = CryptographicBuffer::CreateFromByteArray(&frame.bgra).map_err(os)?;
    let bitmap = SoftwareBitmap::CreateCopyWithAlphaFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        frame.width as i32,
        frame.height as i32,
        BitmapAlphaMode::Premultiplied,
    )
    .map_err(os)?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(os)?
        .join()
        .map_err(os)?;
    // Keep line structure (Text joins lines with spaces).
    let lines = result.Lines().map_err(os)?;
    let mut out = Vec::new();
    for line in lines {
        out.push(line.Text().map_err(os)?.to_string());
    }
    Ok(out.join("\n"))
}

#[derive(Default)]
pub struct WinScreenText;

impl ScreenText for WinScreenText {
    fn uia_text(&self, window: u64, max_chars: usize) -> Option<String> {
        uia::window_text(window, max_chars)
    }
    fn ocr(&self, frame: &Frame, language: &str) -> Result<String, CaptureError> {
        recognize(frame, language)
    }
}
