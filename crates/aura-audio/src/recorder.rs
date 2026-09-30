//! Audio segments (10 s) encoded, sealed and stored with the screen segment
//! store (OT-005 of 004, OT-001 of 005).
//!
//! Default encoder: 16-bit PCM compressed with zstd (portable, lossless).
//! The Windows build may plug a Media Foundation AAC encoder through
//! [`AudioEncoder`] to shrink files (see HANDOFF, H-013).

use crate::hub::Chunk;
use aura_capture::retention::{RetentionPolicy, SegmentMeta};
use aura_capture::segments::{SegmentError, SegmentStore};

pub trait AudioEncoder: Send {
    fn encode(&mut self, samples_16k_mono: &[f32]) -> Result<Vec<u8>, SegmentError>;
    fn decode(&self, bytes: &[u8]) -> Result<Vec<f32>, SegmentError>;
}

pub struct PcmZstd;

impl AudioEncoder for PcmZstd {
    fn encode(&mut self, samples: &[f32]) -> Result<Vec<u8>, SegmentError> {
        let pcm: Vec<u8> = samples
            .iter()
            .flat_map(|s| ((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes())
            .collect();
        zstd::encode_all(&pcm[..], 6).map_err(|e| SegmentError::Encoder(e.to_string()))
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<f32>, SegmentError> {
        let pcm = zstd::decode_all(bytes).map_err(|e| SegmentError::Encoder(e.to_string()))?;
        Ok(pcm
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / i16::MAX as f32)
            .collect())
    }
}

pub const SEGMENT_MS: i64 = 10_000;

/// Accumulates 16 kHz chunks and closes a segment every 10 s.
pub struct AudioSegmentWriter {
    source: String,
    kind: String,
    recording_id: Option<String>,
    encoder: Box<dyn AudioEncoder>,
    store: SegmentStore,
    retention: RetentionPolicy,
    buf: Vec<f32>,
    start_ms: Option<i64>,
}

impl AudioSegmentWriter {
    pub fn new(
        source: &str,
        kind: &str,
        recording_id: Option<String>,
        encoder: Box<dyn AudioEncoder>,
        store: SegmentStore,
        retention: RetentionPolicy,
    ) -> Self {
        Self {
            source: source.into(),
            kind: kind.into(),
            recording_id,
            encoder,
            store,
            retention,
            buf: Vec::new(),
            start_ms: None,
        }
    }

    pub fn push(&mut self, chunk: &Chunk) -> Result<Option<SegmentMeta>, SegmentError> {
        self.start_ms.get_or_insert(chunk.at_ms);
        self.buf.extend_from_slice(&chunk.samples);
        if self.buf.len() as i64 >= SEGMENT_MS * 16 {
            return self.flush();
        }
        Ok(None)
    }

    /// Closes the current segment (10 s boundary, pause or stop).
    pub fn flush(&mut self) -> Result<Option<SegmentMeta>, SegmentError> {
        let Some(start) = self.start_ms.take() else {
            return Ok(None);
        };
        if self.buf.is_empty() {
            return Ok(None);
        }
        let samples = std::mem::take(&mut self.buf);
        let end = start + (samples.len() as i64 * 1000 / 16_000);
        let bytes = self.encoder.encode(&samples)?;
        let meta = self.store.write(
            SegmentMeta {
                id: uuid::Uuid::new_v4().to_string(),
                source: self.source.clone(),
                kind: self.kind.clone(),
                recording_id: self.recording_id.clone(),
                start_ms: start,
                end_ms: end,
                bytes: 0,
                manual: false,
            },
            &bytes,
        )?;
        self.store.apply_retention(&self.retention, end)?;
        Ok(Some(meta))
    }
}

/// Decodes and concatenates the audio between `from_ms` and `to_ms`.
pub fn read_range(
    store: &SegmentStore,
    decoder: &dyn AudioEncoder,
    source: &str,
    from_ms: i64,
    to_ms: i64,
) -> Result<Vec<f32>, SegmentError> {
    let mut out = Vec::new();
    for seg in store.range(source, from_ms, to_ms)? {
        let samples = decoder.decode(&store.read(&seg.id)?)?;
        let skip = (((from_ms - seg.start_ms).max(0)) * 16) as usize;
        let take_until = (((to_ms.min(seg.end_ms)) - seg.start_ms).max(0) * 16) as usize;
        out.extend_from_slice(&samples[skip.min(samples.len())..take_until.min(samples.len())]);
    }
    Ok(out)
}
