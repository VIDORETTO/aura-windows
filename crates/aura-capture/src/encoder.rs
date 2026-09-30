//! Encoder and sealer seams shared by the recorder and the Windows adapters.
//! Kept free of storage dependencies so `aura-win` builds without SQLite.

use crate::frame::Frame;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SegmentError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage: {0}")]
    Store(String),
    #[error("encryption: {0}")]
    Seal(String),
    #[error("encoder: {0}")]
    Encoder(String),
}

/// Encodes frames into self-contained segments.
pub trait VideoEncoder: Send {
    /// Adds a frame; returns bytes of a finished segment when one closes.
    fn push(&mut self, frame: &Frame) -> Result<Option<Vec<u8>>, SegmentError>;
    /// Flushes the current segment (pause, stop).
    fn flush(&mut self) -> Result<Option<Vec<u8>>, SegmentError>;
    /// File extension of decrypted segments (e.g. `mp4`).
    fn format(&self) -> &'static str;
}

/// Test encoder: a segment is the list of synthetic frame numbers, closed
/// every `frames_per_segment` frames.
pub struct NullEncoder {
    pub frames_per_segment: usize,
    buf: Vec<i64>,
}

impl NullEncoder {
    pub fn new(frames_per_segment: usize) -> Self {
        Self {
            frames_per_segment,
            buf: Vec::new(),
        }
    }
}

impl VideoEncoder for NullEncoder {
    fn push(&mut self, frame: &Frame) -> Result<Option<Vec<u8>>, SegmentError> {
        self.buf
            .push(crate::source::SyntheticSource::number_of(frame));
        if self.buf.len() >= self.frames_per_segment {
            return self.flush();
        }
        Ok(None)
    }
    fn flush(&mut self) -> Result<Option<Vec<u8>>, SegmentError> {
        if self.buf.is_empty() {
            return Ok(None);
        }
        let out = serde_json::to_vec(&std::mem::take(&mut self.buf)).expect("json");
        Ok(Some(out))
    }
    fn format(&self) -> &'static str {
        "json"
    }
}

pub trait Sealer: Send + Sync {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, SegmentError>;
    fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, SegmentError>;
}

/// Screen frames per segment at 1 fps (10 s segments, ADR 0004).
pub const FRAMES_PER_SEGMENT: usize = 10;

/// Encodes screen segments and decodes keyframes back (platform-specific:
/// Media Foundation on Windows, [`NullCodec`] in tests and the demo).
pub trait VideoCodec: Send + Sync {
    fn encoder(&self) -> Result<Box<dyn VideoEncoder>, String>;
    /// Frames of one decrypted segment as `(offset_ms, frame)`, at most about `max`.
    fn keyframes(&self, segment: &[u8], max: usize) -> Result<Vec<(i64, Frame)>, String>;
}

/// Codec over [`NullEncoder`]: a segment is the JSON list of synthetic frame
/// numbers, one frame per second.
pub struct NullCodec;

impl VideoCodec for NullCodec {
    fn encoder(&self) -> Result<Box<dyn VideoEncoder>, String> {
        Ok(Box::new(NullEncoder::new(FRAMES_PER_SEGMENT)))
    }
    fn keyframes(&self, segment: &[u8], max: usize) -> Result<Vec<(i64, Frame)>, String> {
        let numbers: Vec<i64> = serde_json::from_slice(segment).map_err(|e| e.to_string())?;
        Ok(numbers
            .iter()
            .enumerate()
            .take(max.max(1) * 4)
            .map(|(i, n)| {
                (
                    i as i64 * 1000,
                    Frame::solid(
                        64,
                        40,
                        [(n % 256) as u8, ((n / 256) % 256) as u8, 0x42, 0xff],
                    ),
                )
            })
            .collect())
    }
}

/// Decodes media files for attachments (007 TK-004/005): Media Foundation
/// on Windows, a fake in tests; `None` platforms report "unsupported".
pub trait MediaFileDecoder: Send + Sync {
    /// Whole audio track as 16 kHz mono f32.
    fn audio_16k(&self, path: &std::path::Path) -> Result<Vec<f32>, String>;
    /// Up to `max` frames spread over the video, as `(time_ms, frame)`.
    fn video_frames(&self, path: &std::path::Path, max: usize)
    -> Result<Vec<(i64, Frame)>, String>;
}

/// No decoder available (Linux builds).
pub struct NoMediaDecoder;

impl MediaFileDecoder for NoMediaDecoder {
    fn audio_16k(&self, _path: &std::path::Path) -> Result<Vec<f32>, String> {
        Err("este formato de mídia requer o Aura para Windows".into())
    }
    fn video_frames(
        &self,
        _path: &std::path::Path,
        _max: usize,
    ) -> Result<Vec<(i64, Frame)>, String> {
        Err("este formato de mídia requer o Aura para Windows".into())
    }
}
