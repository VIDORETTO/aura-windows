//! Audio attachments (007 TK-004) without the worker: WAV is decoded here,
//! resampled to 16 kHz mono and transcribed with the selected speech model
//! into time-stamped blocks. Other codecs (MP3, M4A, video) go to the
//! platform `HeavyIngestor` (Media Foundation in the worker on Windows).

use crate::voice::Voice;
use aura_asr::transcriber::{AsrOptions, Segment};
use aura_audio::dsp::{Resampler, TARGET_RATE, to_mono};
use aura_ingest::model::{Block, Content, DocKind, IngestedDoc, Locator};
use aura_ingest::{Format, HeavyIngestor, IngestError, MAX_MEDIA_SECONDS, Selector};
use std::path::Path;
use std::sync::Arc;

/// Groups consecutive segments into ~30 s blocks (good granularity to cite).
const BLOCK_MS: i64 = 30_000;

pub fn decode_wav(path: &Path) -> Result<(Vec<f32>, u32), IngestError> {
    let mut r = hound::WavReader::open(path).map_err(|e| IngestError::Corrupt(e.to_string()))?;
    let spec = r.spec();
    let channels = spec.channels.max(1) as usize;
    let frames = r.duration() as u64;
    if frames / spec.sample_rate.max(1) as u64 > MAX_MEDIA_SECONDS {
        return Err(IngestError::TooLarge {
            limit: MAX_MEDIA_SECONDS,
        });
    }
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r
            .samples::<f32>()
            .collect::<Result<_, _>>()
            .map_err(|e| IngestError::Corrupt(e.to_string()))?,
        hound::SampleFormat::Int => {
            let scale = (1i64 << (spec.bits_per_sample.clamp(1, 32) - 1)) as f32;
            r.samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale))
                .collect::<Result<_, _>>()
                .map_err(|e| IngestError::Corrupt(e.to_string()))?
        }
    };
    let mono = to_mono(&samples, channels);
    let pcm = if spec.sample_rate == TARGET_RATE {
        mono
    } else {
        Resampler::new(spec.sample_rate, TARGET_RATE).process(&mono)
    };
    Ok((pcm, spec.sample_rate))
}

pub fn fmt_ts(ms: i64) -> String {
    let s = ms.max(0) / 1000;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

pub fn segments_to_blocks(segments: &[Segment]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut cur: Vec<&Segment> = Vec::new();
    let flush = |cur: &mut Vec<&Segment>, blocks: &mut Vec<Block>| {
        if let (Some(first), Some(last)) = (cur.first(), cur.last()) {
            let text = cur
                .iter()
                .map(|s| format!("[{}] {}", fmt_ts(s.start_ms), s.text.trim()))
                .collect::<Vec<_>>()
                .join("\n");
            blocks.push(Block {
                locator: Locator::Time {
                    from_ms: first.start_ms,
                    to_ms: last.end_ms,
                },
                content: Content::Text { text },
            });
        }
        cur.clear();
    };
    for s in segments.iter().filter(|s| !s.text.trim().is_empty()) {
        if cur
            .first()
            .is_some_and(|f| s.end_ms - f.start_ms > BLOCK_MS)
        {
            flush(&mut cur, &mut blocks);
        }
        cur.push(s);
    }
    flush(&mut cur, &mut blocks);
    blocks
}

pub use aura_ingest::parse_time_range;

pub struct MediaIngestor {
    pub voice: Arc<Voice>,
    pub media: Arc<dyn aura_capture::encoder::MediaFileDecoder>,
    pub fallback: Arc<dyn HeavyIngestor>,
    pub handle: tokio::runtime::Handle,
}

/// Keyframes kept per video (sent as images to the model).
pub const VIDEO_FRAMES: usize = 8;

impl MediaIngestor {
    fn is_wav(path: &Path) -> bool {
        path.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("wav"))
    }

    fn file_name(path: &Path) -> String {
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Transcribes 16 kHz PCM into time blocks; returns (blocks, model label).
    fn transcribe(&self, pcm: &[f32]) -> Result<(Vec<Block>, String), IngestError> {
        let transcriber = self
            .voice
            .transcriber_for_files()
            .map_err(|_| IngestError::NeedsAsr)?;
        // Called from `spawn_blocking`: blocking on the runtime is allowed here.
        let t = self
            .handle
            .block_on(transcriber.transcribe(pcm, &AsrOptions::default()))
            .map_err(|e| match e {
                aura_asr::transcriber::AsrError::ModelMissing => IngestError::NeedsAsr,
                other => IngestError::Worker(other.to_string()),
            })?;
        let duration_ms = (pcm.len() as i64 * 1000) / TARGET_RATE as i64;
        let mut blocks = segments_to_blocks(&t.segments);
        if blocks.is_empty() && !t.text.trim().is_empty() {
            blocks.push(Block {
                locator: Locator::Time {
                    from_ms: 0,
                    to_ms: duration_ms,
                },
                content: Content::Text {
                    text: t.text.trim().to_string(),
                },
            });
        }
        Ok((blocks, transcriber.label()))
    }

    fn ingest_audio(&self, path: &Path) -> Result<IngestedDoc, IngestError> {
        let pcm = if Self::is_wav(path) {
            decode_wav(path)?.0
        } else {
            self.media.audio_16k(path).map_err(IngestError::Worker)?
        };
        if pcm.len() as u64 / TARGET_RATE as u64 > MAX_MEDIA_SECONDS {
            return Err(IngestError::TooLarge {
                limit: MAX_MEDIA_SECONDS,
            });
        }
        let duration_ms = (pcm.len() as i64 * 1000) / TARGET_RATE as i64;
        let (blocks, label) = self.transcribe(&pcm)?;
        Ok(IngestedDoc {
            kind: DocKind::Audio,
            file_name: Self::file_name(path),
            summary: format!("áudio {} · transcrito ({label})", fmt_ts(duration_ms)),
            blocks,
            warnings: vec![],
        })
    }

    fn ingest_video(&self, path: &Path) -> Result<IngestedDoc, IngestError> {
        let mut frames = self
            .media
            .video_frames(path, VIDEO_FRAMES)
            .map_err(IngestError::Worker)?;
        // Seeks may land on the same keyframe; one image per timestamp.
        frames.sort_by_key(|(at, _)| *at);
        frames.dedup_by_key(|(at, _)| *at);
        let dir = path.with_extension("frames");
        std::fs::create_dir_all(&dir)?;
        let mut blocks = Vec::new();
        for (at, f) in &frames {
            let png = dir.join(format!("{at:010}.png"));
            std::fs::write(&png, f.downscale(1024).to_png())?;
            blocks.push(Block {
                locator: Locator::Time {
                    from_ms: *at,
                    to_ms: *at,
                },
                content: Content::Image { path: png },
            });
        }
        let mut warnings = Vec::new();
        let mut label = None;
        match self.media.audio_16k(path) {
            Ok(pcm) if !pcm.is_empty() => match self.transcribe(&pcm) {
                Ok((text_blocks, l)) => {
                    blocks.extend(text_blocks);
                    label = Some(l);
                }
                Err(IngestError::NeedsAsr) => warnings
                    .push("sem modelo de voz: o vídeo foi anexado só com quadros".to_string()),
                Err(e) => warnings.push(format!("transcrição falhou: {e}")),
            },
            _ => warnings.push("o vídeo não tem áudio".to_string()),
        }
        let start = |b: &Block| {
            if let Locator::Time { from_ms, .. } = b.locator {
                from_ms
            } else {
                0
            }
        };
        blocks.sort_by_key(start);
        let last = frames.last().map(|(t, _)| *t).unwrap_or(0);
        Ok(IngestedDoc {
            kind: DocKind::Video,
            file_name: Self::file_name(path),
            summary: format!(
                "vídeo ~{} · {} quadros{}",
                fmt_ts(last),
                frames.len(),
                label
                    .map(|l| format!(" · transcrito ({l})"))
                    .unwrap_or_default()
            ),
            blocks,
            warnings,
        })
    }
}

impl HeavyIngestor for MediaIngestor {
    fn ingest(&self, format: Format, path: &Path) -> Result<IngestedDoc, IngestError> {
        let result = match format {
            Format::Audio => self.ingest_audio(path),
            Format::Video => self.ingest_video(path),
            _ => return self.fallback.ingest(format, path),
        };
        // Unsupported codec here → let the platform worker try.
        match result {
            Err(IngestError::Worker(_)) => self.fallback.ingest(format, path),
            other => other,
        }
    }

    fn read(
        &self,
        format: Format,
        path: &Path,
        selector: &Selector,
    ) -> Result<Vec<Block>, IngestError> {
        self.fallback.read(format, path, selector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(a: i64, b: i64, t: &str) -> Segment {
        Segment {
            start_ms: a,
            end_ms: b,
            text: t.into(),
        }
    }

    #[test]
    fn blocks_group_segments_with_timestamps() {
        let s = vec![
            seg(0, 4000, "Olá"),
            seg(4000, 20000, "pauta"),
            seg(20000, 35000, "próximo"),
            seg(61000, 65000, "fim"),
        ];
        let b = segments_to_blocks(&s);
        // 0–20 s | 20–35 s | 61–65 s (each block spans at most 30 s).
        assert_eq!(b.len(), 3);
        assert_eq!(
            b[0].locator,
            Locator::Time {
                from_ms: 0,
                to_ms: 20000
            }
        );
        assert!(
            matches!(&b[1].content, Content::Text { text } if text.starts_with("[00:20] próximo"))
        );
        assert_eq!(fmt_ts(3_725_000), "1:02:05");
    }

    #[test]
    fn time_ranges() {
        assert_eq!(parse_time_range("01:00-02:30"), Some((60_000, 150_000)));
        assert_eq!(parse_time_range("75-90"), Some((75_000, 90_000)));
        assert_eq!(parse_time_range("2:00-1:00"), None);
        assert_eq!(parse_time_range("abc"), None);
    }

    #[test]
    fn wav_decodes_to_16k_mono() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&p, spec).unwrap();
        for i in 0..48_000 {
            let v = ((i as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 10_000.0) as i16;
            w.write_sample(v).unwrap();
            w.write_sample(v).unwrap();
        }
        w.finalize().unwrap();
        let (pcm, rate) = decode_wav(&p).unwrap();
        assert_eq!(rate, 48_000);
        assert!((pcm.len() as i64 - 16_000).abs() < 50, "{}", pcm.len());
        assert!(pcm.iter().any(|s| s.abs() > 0.2));
    }
}
