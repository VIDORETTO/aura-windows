//! One device stream per source, fanned out to subscribers (level meter,
//! push-to-talk, recorder) as 16 kHz mono chunks (OT-002 of 005).

use crate::dsp::{Resampler, TARGET_RATE, rms_dbfs, to_mono};
use crate::{AudioError, AudioSource, AudioSourceKind, DeviceSel, RawChunk};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub samples: Arc<[f32]>,
    pub at_ms: i64,
    pub level_dbfs: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HubEvent {
    DeviceFallback { wanted: String, using: String },
    Stopped { reason: Option<AudioError> },
}

/// Opens `device`; for system audio the id `*` means every output at once.
fn open_device(
    source: &dyn AudioSource,
    kind: AudioSourceKind,
    device: &DeviceSel,
) -> Result<Box<dyn crate::AudioStream>, AudioError> {
    match device {
        DeviceSel::Id(id)
            if kind == AudioSourceKind::SystemAudio && id == crate::mixed::ALL_OUTPUTS =>
        {
            Ok(Box::new(crate::mixed::open_all(source, kind)?))
        }
        other => source.open(kind, other),
    }
}

/// Pumps one device on a dedicated thread (audio APIs are blocking).
pub struct AudioHub {
    tx: broadcast::Sender<Chunk>,
    events: broadcast::Sender<HubEvent>,
    running: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    /// The requested device id when it was unavailable and the default opened.
    fallback_from: Option<String>,
}

impl AudioHub {
    pub fn start(
        source: Arc<dyn AudioSource>,
        kind: AudioSourceKind,
        device: DeviceSel,
    ) -> Result<Self, AudioError> {
        let (tx, _) = broadcast::channel(256);
        let (events, _) = broadcast::channel(16);
        let mut fallback_from = None;
        let mut stream = match open_device(source.as_ref(), kind, &device) {
            Ok(s) => s,
            Err(AudioError::NoDevice | AudioError::DeviceLost) if device != DeviceSel::Default => {
                let wanted = match &device {
                    DeviceSel::Id(id) => id.clone(),
                    DeviceSel::Default => String::new(),
                };
                let s = source.open(kind, &DeviceSel::Default)?;
                fallback_from = Some(wanted.clone());
                let _ = events.send(HubEvent::DeviceFallback {
                    wanted,
                    using: "default".into(),
                });
                s
            }
            Err(e) => return Err(e),
        };
        let running = Arc::new(AtomicBool::new(true));
        let (t_tx, t_events, t_running) = (tx.clone(), events.clone(), running.clone());
        let thread = std::thread::Builder::new()
            .name(format!("aura-audio-{}", kind.key()))
            .spawn(move || {
                let mut resampler: Option<(u32, Resampler)> = None;
                let mut reason = None;
                while t_running.load(Ordering::Relaxed) {
                    match stream.next_chunk() {
                        Some(Ok(RawChunk {
                            samples,
                            channels,
                            rate,
                            at_ms,
                        })) => {
                            let mono = to_mono(&samples, channels);
                            let r = match &mut resampler {
                                Some((r_rate, r)) if *r_rate == rate => r,
                                _ => {
                                    &mut resampler
                                        .insert((rate, Resampler::new(rate, TARGET_RATE)))
                                        .1
                                }
                            };
                            let out = r.process(&mono);
                            if out.is_empty() {
                                continue;
                            }
                            let level = rms_dbfs(&out);
                            let _ = t_tx.send(Chunk {
                                samples: out.into(),
                                at_ms,
                                level_dbfs: level,
                            });
                        }
                        Some(Err(e)) => {
                            reason = Some(e);
                            break;
                        }
                        None => break,
                    }
                }
                let _ = t_events.send(HubEvent::Stopped { reason });
            })
            .map_err(|e| AudioError::Os(e.to_string()))?;
        Ok(Self {
            tx,
            events,
            running,
            thread: Some(thread),
            fallback_from,
        })
    }

    /// The chosen device id when it was missing and the OS default was used.
    pub fn fallback_from(&self) -> Option<&str> {
        self.fallback_from.as_deref()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Chunk> {
        self.tx.subscribe()
    }

    pub fn events(&self) -> broadcast::Receiver<HubEvent> {
        self.events.subscribe()
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for AudioHub {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Test/demo source: a sine at `freq` Hz, 48 kHz stereo, 20 ms chunks, for
/// `total_ms` (then the device "stops").
pub struct SyntheticAudio {
    pub freq: f32,
    pub amplitude: f32,
    pub total_ms: i64,
    pub realtime: bool,
    pub fail_ids: Vec<String>,
}

struct SyntheticStream {
    freq: f32,
    amplitude: f32,
    t: i64,
    total_ms: i64,
    phase: f32,
    realtime: bool,
    /// Wall-clock origin in real-time mode (like a real device); 0 otherwise.
    origin_ms: i64,
}

impl crate::AudioStream for SyntheticStream {
    fn next_chunk(&mut self) -> Option<Result<RawChunk, AudioError>> {
        if self.t >= self.total_ms {
            return None;
        }
        if self.realtime {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let rate = 48_000u32;
        let n = (rate / 50) as usize;
        let mut samples = Vec::with_capacity(n * 2);
        for _ in 0..n {
            let v = self.amplitude * self.phase.sin();
            self.phase += 2.0 * std::f32::consts::PI * self.freq / rate as f32;
            samples.push(v);
            samples.push(v);
        }
        let at = self.origin_ms + self.t;
        self.t += 20;
        Some(Ok(RawChunk {
            samples,
            channels: 2,
            rate,
            at_ms: at,
        }))
    }
}

impl AudioSource for SyntheticAudio {
    fn devices(&self, _kind: AudioSourceKind) -> Vec<crate::DeviceInfo> {
        vec![crate::DeviceInfo {
            id: "synthetic".into(),
            name: "Synthetic".into(),
            is_default: true,
        }]
    }
    fn open(
        &self,
        _kind: AudioSourceKind,
        device: &DeviceSel,
    ) -> Result<Box<dyn crate::AudioStream>, AudioError> {
        if let DeviceSel::Id(id) = device
            && self.fail_ids.contains(id)
        {
            return Err(AudioError::NoDevice);
        }
        Ok(Box::new(SyntheticStream {
            freq: self.freq,
            amplitude: self.amplitude,
            t: 0,
            total_ms: self.total_ms,
            phase: 0.0,
            realtime: self.realtime,
            origin_ms: if self.realtime {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0)
            } else {
                0
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn two_subscribers_share_one_stream_and_see_the_level() {
        let src = Arc::new(SyntheticAudio {
            freq: 440.0,
            amplitude: 0.355,
            total_ms: 1000,
            realtime: false,
            fail_ids: vec![],
        });
        let mut hub = AudioHub::start(src, AudioSourceKind::Mic, DeviceSel::Default).unwrap();
        let mut a = hub.subscribe();
        let mut b = hub.subscribe();
        let mut ev = hub.events();
        // Wait for the synthetic device to finish.
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), ev.recv()).await;
        hub.stop();
        let mut got_a = 0;
        let mut levels = Vec::new();
        while let Ok(c) = a.try_recv() {
            got_a += c.samples.len();
            levels.push(c.level_dbfs);
        }
        let mut got_b = 0;
        while let Ok(c) = b.try_recv() {
            got_b += c.samples.len();
        }
        assert_eq!(got_a, got_b);
        assert!(got_a > 15_000);
        let steady = &levels[levels.len() / 2..];
        assert!(steady.iter().all(|l| (l + 12.0).abs() < 1.0), "{steady:?}");
    }

    #[tokio::test]
    async fn missing_device_falls_back_to_default() {
        let src = Arc::new(SyntheticAudio {
            freq: 440.0,
            amplitude: 0.1,
            total_ms: 100,
            realtime: false,
            fail_ids: vec!["usb-mic".into()],
        });
        let hub = AudioHub::start(src, AudioSourceKind::Mic, DeviceSel::Id("usb-mic".into()));
        assert!(hub.is_ok());
    }
}
