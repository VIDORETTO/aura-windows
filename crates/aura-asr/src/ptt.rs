//! Push-to-talk controller (006 TK-004): Idle → Listening → Transcribing →
//! Done | Empty | Cancelled | Failed. Audio stays in memory (OT-003).

use crate::text::trim_silence;
use crate::transcriber::{AsrError, AsrOptions, Transcriber};
use aura_audio::hub::Chunk;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast, oneshot};

pub const MAX_SPEECH_SECONDS: usize = 5 * 60;
pub const VAD_THRESHOLD_DBFS: f32 = -45.0;
pub const MIN_SPEECH_MS: u32 = 300;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "state",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PttState {
    Idle,
    Listening,
    /// Live transcript of what was said so far (006 TK-007).
    Partial {
        text: String,
    },
    Transcribing,
    Done {
        text: String,
    },
    Empty,
    Cancelled,
    Failed {
        error: String,
    },
}

struct Session {
    stop: oneshot::Sender<bool>,
    collected: oneshot::Receiver<Vec<f32>>,
}

pub struct PushToTalk {
    transcriber: Arc<dyn Transcriber>,
    /// Re-transcribe the growing buffer this often while listening.
    partial_every: Option<std::time::Duration>,
    session: Mutex<Option<Session>>,
    state: broadcast::Sender<PttState>,
}

impl PushToTalk {
    pub fn new(transcriber: Arc<dyn Transcriber>) -> Self {
        let (state, _) = broadcast::channel(32);
        Self {
            transcriber,
            partial_every: None,
            session: Mutex::new(None),
            state,
        }
    }

    /// Enables live partial transcripts every `every` while listening.
    pub fn with_partials(mut self, every: std::time::Duration) -> Self {
        self.partial_every = Some(every);
        self
    }

    pub fn states(&self) -> broadcast::Receiver<PttState> {
        self.state.subscribe()
    }

    /// Starts collecting audio from the microphone hub.
    pub async fn press(&self, mut audio: broadcast::Receiver<Chunk>) {
        let mut guard = self.session.lock().await;
        if guard.is_some() {
            return;
        }
        let (stop_tx, mut stop_rx) = oneshot::channel::<bool>();
        let (out_tx, out_rx) = oneshot::channel::<Vec<f32>>();
        let partial_every = self.partial_every;
        let transcriber = self.transcriber.clone();
        let states = self.state.clone();
        tokio::spawn(async move {
            let mut buf: Vec<f32> = Vec::new();
            let mut tick = tokio::time::interval(
                partial_every.unwrap_or(std::time::Duration::from_secs(3600)),
            );
            tick.tick().await;
            let busy = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let mut last_len = 0usize;
            loop {
                tokio::select! {
                    _ = tick.tick(), if partial_every.is_some() => {
                        // At least 0.5 s of audio and 0.4 s of new audio; never overlap runs.
                        if buf.len() >= 8_000 && buf.len() >= last_len + 6_400
                            && !busy.swap(true, std::sync::atomic::Ordering::SeqCst)
                        {
                            last_len = buf.len();
                            let snapshot = buf.clone();
                            let (t, s, b) = (transcriber.clone(), states.clone(), busy.clone());
                            tokio::spawn(async move {
                                if let Ok(tr) = t.transcribe(&snapshot, &AsrOptions::default()).await
                                    && !tr.text.trim().is_empty()
                                {
                                    let _ = s.send(PttState::Partial { text: tr.text.trim().to_string() });
                                }
                                b.store(false, std::sync::atomic::Ordering::SeqCst);
                            });
                        }
                    }
                    keep = &mut stop_rx => {
                        let _ = out_tx.send(if keep.unwrap_or(false) { buf } else { Vec::new() });
                        return;
                    }
                    chunk = audio.recv() => match chunk {
                        Ok(c) => {
                            buf.extend_from_slice(&c.samples);
                            if buf.len() >= MAX_SPEECH_SECONDS * 16_000 {
                                buf.truncate(MAX_SPEECH_SECONDS * 16_000);
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => {
                            // Device stopped: wait for release/cancel.
                            let keep = stop_rx.await.unwrap_or(false);
                            let _ = out_tx.send(if keep { buf } else { Vec::new() });
                            return;
                        }
                    }
                }
            }
        });
        *guard = Some(Session {
            stop: stop_tx,
            collected: out_rx,
        });
        let _ = self.state.send(PttState::Listening);
    }

    /// `Esc` while listening: discard everything.
    pub async fn cancel(&self) -> PttState {
        if let Some(s) = self.session.lock().await.take() {
            let _ = s.stop.send(false);
        }
        let _ = self.state.send(PttState::Cancelled);
        PttState::Cancelled
    }

    /// Release: trims silence and transcribes.
    pub async fn release(&self, opts: &AsrOptions) -> PttState {
        let Some(s) = self.session.lock().await.take() else {
            return PttState::Idle;
        };
        let _ = s.stop.send(true);
        let pcm = s.collected.await.unwrap_or_default();
        let result = match trim_silence(&pcm, VAD_THRESHOLD_DBFS, MIN_SPEECH_MS) {
            None => PttState::Empty,
            Some(speech) => {
                let _ = self.state.send(PttState::Transcribing);
                match self.transcriber.transcribe(&speech, opts).await {
                    Ok(t) if t.text.trim().is_empty() => PttState::Empty,
                    Ok(t) => PttState::Done {
                        text: t.text.trim().to_string(),
                    },
                    Err(AsrError::ModelMissing) => PttState::Failed {
                        error: "model_missing".into(),
                    },
                    Err(e) => PttState::Failed {
                        error: e.to_string(),
                    },
                }
            }
        };
        let _ = self.state.send(result.clone());
        result
    }
}

/// Inserts `text` at the caret position (in characters) of `current`.
pub fn insert_at_caret(current: &str, caret_chars: usize, text: &str) -> (String, usize) {
    let caret = caret_chars.min(current.chars().count());
    let byte = current
        .char_indices()
        .nth(caret)
        .map(|(i, _)| i)
        .unwrap_or(current.len());
    let (a, b) = current.split_at(byte);
    let needs_space = !a.is_empty() && !a.ends_with(char::is_whitespace);
    let insert = if needs_space {
        format!(" {text}")
    } else {
        text.to_string()
    };
    (format!("{a}{insert}{b}"), caret + insert.chars().count())
}

#[cfg(test)]
mod partial_tests {
    use super::*;
    use crate::transcriber::FakeTranscriber;
    use aura_audio::hub::Chunk;

    #[tokio::test]
    async fn partials_are_emitted_while_listening() {
        let fake = Arc::new(FakeTranscriber {
            text: "olá parcial".into(),
            delay: std::time::Duration::ZERO,
        });
        let ptt = PushToTalk::new(fake).with_partials(std::time::Duration::from_millis(30));
        let mut states = ptt.states();
        let (tx, rx) = tokio::sync::broadcast::channel(64);
        ptt.press(rx).await;
        for i in 0..6 {
            let samples: Vec<f32> = (0..4_000)
                .map(|n| ((n as f32) * 0.05).sin() * 0.3)
                .collect();
            tx.send(Chunk {
                samples: samples.into(),
                at_ms: i * 250,
                level_dbfs: -10.0,
            })
            .unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        }
        let mut saw_partial = false;
        while let Ok(s) = states.try_recv() {
            if s == (PttState::Partial {
                text: "olá parcial".into(),
            }) {
                saw_partial = true;
            }
        }
        assert!(saw_partial);
        ptt.cancel().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcriber::FakeTranscriber;
    use aura_audio::dsp::sine;
    use std::time::Duration;

    fn feed(tx: &broadcast::Sender<Chunk>, samples: &[f32]) {
        for (i, c) in samples.chunks(320).enumerate() {
            let _ = tx.send(Chunk {
                samples: c.to_vec().into(),
                at_ms: i as i64 * 20,
                level_dbfs: -20.0,
            });
        }
    }

    fn ptt() -> PushToTalk {
        PushToTalk::new(Arc::new(FakeTranscriber {
            text: "olá mundo".into(),
            delay: Duration::ZERO,
        }))
    }

    #[tokio::test]
    async fn speech_is_transcribed_after_release() {
        let (tx, _) = broadcast::channel(4096);
        let p = ptt();
        p.press(tx.subscribe()).await;
        feed(
            &tx,
            &[
                vec![0.0; 8000],
                sine(300.0, 16_000, 1.0, 0.3),
                vec![0.0; 8000],
            ]
            .concat(),
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(
            p.release(&AsrOptions::default()).await,
            PttState::Done {
                text: "olá mundo".into()
            }
        );
    }

    #[tokio::test]
    async fn silence_is_empty_and_escape_cancels() {
        let (tx, _) = broadcast::channel(4096);
        let p = ptt();
        p.press(tx.subscribe()).await;
        feed(&tx, &vec![0.0; 32_000]);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(p.release(&AsrOptions::default()).await, PttState::Empty);
        p.press(tx.subscribe()).await;
        feed(&tx, &sine(300.0, 16_000, 1.0, 0.3));
        assert_eq!(p.cancel().await, PttState::Cancelled);
        assert_eq!(p.release(&AsrOptions::default()).await, PttState::Idle);
    }

    #[test]
    fn caret_insertion() {
        assert_eq!(
            insert_at_caret("Pergunta:", 9, "olá mundo"),
            ("Pergunta: olá mundo".into(), 19)
        );
        assert_eq!(insert_at_caret("", 0, "oi"), ("oi".into(), 2));
        assert_eq!(insert_at_caret("ação ", 5, "x"), ("ação x".into(), 6));
    }
}
