//! The `Transcriber` seam: local (worker), cloud (BYOK) and fake adapters.

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrOptions {
    /// `None` = automatic detection.
    pub language: Option<String>,
    pub vocabulary: Vec<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AsrError {
    #[error("no speech model installed")]
    ModelMissing,
    #[error("could not load the model: {0}")]
    LoadFailed(String),
    #[error("the transcription process stopped unexpectedly")]
    WorkerCrashed,
    #[error("transcription timed out")]
    Timeout,
    #[error("cloud transcription failed: {0}")]
    Cloud(String),
}

pub trait Transcriber: Send + Sync {
    /// `pcm` is 16 kHz mono f32.
    fn transcribe<'a>(
        &'a self,
        pcm: &'a [f32],
        opts: &'a AsrOptions,
    ) -> BoxFut<'a, Result<Transcript, AsrError>>;
    fn label(&self) -> String;
}

/// Deterministic transcriber for tests and the demo mode.
pub struct FakeTranscriber {
    pub text: String,
    pub delay: std::time::Duration,
}

impl Transcriber for FakeTranscriber {
    fn transcribe<'a>(
        &'a self,
        pcm: &'a [f32],
        opts: &'a AsrOptions,
    ) -> BoxFut<'a, Result<Transcript, AsrError>> {
        Box::pin(async move {
            if !self.delay.is_zero() {
                tokio::time::sleep(self.delay).await;
            }
            let dur = (pcm.len() as i64) * 1000 / 16_000;
            let text = crate::text::apply_vocabulary(&self.text, &opts.vocabulary);
            Ok(Transcript {
                language: opts.language.clone().or(Some("pt".into())),
                segments: vec![Segment {
                    start_ms: 0,
                    end_ms: dur,
                    text: text.clone(),
                }],
                text,
            })
        })
    }
    fn label(&self) -> String {
        "fake".into()
    }
}

/// Uses `primary` and falls back to `fallback` on failure (AC-010 of 006).
pub struct FallbackTranscriber {
    pub primary: Arc<dyn Transcriber>,
    pub fallback: Option<Arc<dyn Transcriber>>,
    pub on_fallback: Arc<dyn Fn(&AsrError) + Send + Sync>,
}

impl Transcriber for FallbackTranscriber {
    fn transcribe<'a>(
        &'a self,
        pcm: &'a [f32],
        opts: &'a AsrOptions,
    ) -> BoxFut<'a, Result<Transcript, AsrError>> {
        Box::pin(async move {
            match self.primary.transcribe(pcm, opts).await {
                Ok(t) => Ok(t),
                Err(e) => match &self.fallback {
                    Some(f) => {
                        (self.on_fallback)(&e);
                        f.transcribe(pcm, opts).await
                    }
                    None => Err(e),
                },
            }
        })
    }
    fn label(&self) -> String {
        self.primary.label()
    }
}
