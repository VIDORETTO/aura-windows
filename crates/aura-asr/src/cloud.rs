//! Cloud ASR through a BYOK provider's `/audio/transcriptions` (006 TK-006).

use crate::transcriber::{AsrError, AsrOptions, BoxFut, Segment, Transcriber, Transcript};
use aura_core::Secret;
use serde_json::Value;

pub struct CloudTranscriber {
    pub base_url: String,
    pub model: String,
    pub credential: Option<Secret<String>>,
    pub http: reqwest::Client,
    pub verbose_json: bool,
}

impl Transcriber for CloudTranscriber {
    fn transcribe<'a>(
        &'a self,
        pcm: &'a [f32],
        opts: &'a AsrOptions,
    ) -> BoxFut<'a, Result<Transcript, AsrError>> {
        Box::pin(async move {
            let wav = aura_audio::dsp::wav_bytes(pcm, 16_000);
            let part = reqwest::multipart::Part::bytes(wav)
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .map_err(|e| AsrError::Cloud(e.to_string()))?;
            let mut form = reqwest::multipart::Form::new()
                .text("model", self.model.clone())
                .text(
                    "response_format",
                    if self.verbose_json {
                        "verbose_json"
                    } else {
                        "json"
                    },
                )
                .part("file", part);
            if let Some(lang) = &opts.language {
                form = form.text("language", lang.clone());
            }
            let prompt = crate::text::prompt_for_whisper(&opts.vocabulary);
            if !prompt.is_empty() {
                form = form.text("prompt", prompt);
            }
            let url = format!(
                "{}/audio/transcriptions",
                self.base_url.trim_end_matches('/')
            );
            let mut req = self.http.post(url).multipart(form);
            if let Some(key) = &self.credential {
                req = req.bearer_auth(key.expose());
            }
            let resp = req
                .send()
                .await
                .map_err(|e| AsrError::Cloud(e.to_string()))?;
            if !resp.status().is_success() {
                return Err(AsrError::Cloud(format!("HTTP {}", resp.status())));
            }
            let v: Value = resp
                .json()
                .await
                .map_err(|e| AsrError::Cloud(e.to_string()))?;
            let text = v["text"].as_str().unwrap_or_default().trim().to_string();
            let segments: Vec<Segment> = v["segments"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|s| Segment {
                            start_ms: (s["start"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
                            end_ms: (s["end"].as_f64().unwrap_or(0.0) * 1000.0) as i64,
                            text: s["text"].as_str().unwrap_or_default().trim().to_string(),
                        })
                        .collect()
                })
                .unwrap_or_else(|| {
                    vec![Segment {
                        start_ms: 0,
                        end_ms: pcm.len() as i64 * 1000 / 16_000,
                        text: text.clone(),
                    }]
                });
            let text = crate::text::apply_vocabulary(&text, &opts.vocabulary);
            Ok(Transcript {
                language: v["language"].as_str().map(str::to_string),
                segments,
                text,
            })
        })
    }
    fn label(&self) -> String {
        format!("nuvem · {}", self.model)
    }
}
