//! Speech engines hosted by the worker.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transcript {
    pub language: Option<String>,
    pub segments: Vec<Segment>,
    pub text: String,
}

pub trait Engine: Send {
    fn transcribe(
        &mut self,
        pcm: &[f32],
        language: Option<&str>,
        prompt: &str,
    ) -> Result<Transcript, String>;
}

/// Deterministic engine for tests and the demo mode. Text comes from
/// `AURA_WORKER_FAKE_TEXT` (default "olá do worker").
pub struct FakeEngine {
    pub text: String,
}

impl Engine for FakeEngine {
    fn transcribe(
        &mut self,
        pcm: &[f32],
        language: Option<&str>,
        _prompt: &str,
    ) -> Result<Transcript, String> {
        let end = pcm.len() as i64 * 1000 / 16_000;
        Ok(Transcript {
            language: language.map(str::to_string).or(Some("pt".into())),
            segments: vec![Segment {
                start_ms: 0,
                end_ms: end,
                text: self.text.clone(),
            }],
            text: self.text.clone(),
        })
    }
}

pub fn load(engine: &str, model_dir: &std::path::Path) -> Result<Box<dyn Engine>, String> {
    match engine {
        "fake" => Ok(Box::new(FakeEngine {
            text: std::env::var("AURA_WORKER_FAKE_TEXT").unwrap_or_else(|_| "olá do worker".into()),
        })),
        #[cfg(feature = "engines")]
        "onnx-parakeet" => real::parakeet(model_dir),
        #[cfg(feature = "engines")]
        "ggml-whisper" => real::whisper(model_dir),
        other => {
            let _ = model_dir;
            Err(format!(
                "engine '{other}' is not available in this build (enable the `engines` feature)"
            ))
        }
    }
}

#[cfg(feature = "engines")]
mod real {
    //! Real engines via `transcribe-rs` 0.3 (compiled on Windows).
    use super::{Engine, Segment, Transcript};
    use std::path::Path;
    use transcribe_rs::onnx::Quantization;
    use transcribe_rs::onnx::parakeet::{ParakeetModel, ParakeetParams, TimestampGranularity};
    use transcribe_rs::whisper_cpp::{WhisperEngine, WhisperInferenceParams};

    struct Parakeet(ParakeetModel);
    struct Whisper(WhisperEngine);

    pub fn parakeet(dir: &Path) -> Result<Box<dyn Engine>, String> {
        let model = ParakeetModel::load(&dir.to_path_buf(), &Quantization::Int8)
            .map_err(|e| e.to_string())?;
        Ok(Box::new(Parakeet(model)))
    }

    pub fn whisper(dir: &Path) -> Result<Box<dyn Engine>, String> {
        let file = std::fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "bin"))
            .ok_or("no .bin model in directory")?;
        Ok(Box::new(Whisper(
            WhisperEngine::load(&file).map_err(|e| e.to_string())?,
        )))
    }

    impl Engine for Parakeet {
        fn transcribe(
            &mut self,
            pcm: &[f32],
            _language: Option<&str>,
            _prompt: &str,
        ) -> Result<Transcript, String> {
            let r = self
                .0
                .transcribe_with(
                    pcm,
                    &ParakeetParams {
                        timestamp_granularity: Some(TimestampGranularity::Segment),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
            let segments = r
                .segments
                .unwrap_or_default()
                .into_iter()
                .map(|s| Segment {
                    start_ms: (s.start * 1000.0) as i64,
                    end_ms: (s.end * 1000.0) as i64,
                    text: s.text,
                })
                .collect();
            Ok(Transcript {
                language: None,
                segments,
                text: r.text,
            })
        }
    }

    impl Engine for Whisper {
        fn transcribe(
            &mut self,
            pcm: &[f32],
            language: Option<&str>,
            prompt: &str,
        ) -> Result<Transcript, String> {
            let params = WhisperInferenceParams {
                language: language.map(str::to_string),
                initial_prompt: (!prompt.is_empty()).then(|| prompt.to_string()),
                ..Default::default()
            };
            let r = self
                .0
                .transcribe_with(pcm, &params)
                .map_err(|e| e.to_string())?;
            let segments = r
                .segments
                .unwrap_or_default()
                .into_iter()
                .map(|s| Segment {
                    start_ms: (s.start * 1000.0) as i64,
                    end_ms: (s.end * 1000.0) as i64,
                    text: s.text,
                })
                .collect();
            Ok(Transcript {
                language: language.map(str::to_string),
                segments,
                text: r.text,
            })
        }
    }
}
