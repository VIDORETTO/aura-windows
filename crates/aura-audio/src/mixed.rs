//! "All outputs": the system audio of every playback device at once (a user
//! with speakers and a headset, or a call routed to a different device than
//! the video). Each device opens its own loopback stream; their audio is
//! converted to 16 kHz mono and summed every tick into one stream.

use crate::dsp::{Resampler, TARGET_RATE, to_mono};
use crate::{AudioError, AudioSource, AudioSourceKind, AudioStream, DeviceSel, RawChunk};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Device id that means "every output" in settings.
pub const ALL_OUTPUTS: &str = "*";

/// How often the mix is emitted.
const TICK: Duration = Duration::from_millis(100);

struct Shared {
    /// 16 kHz mono samples waiting to be mixed, one queue per device.
    queues: Vec<Mutex<Vec<f32>>>,
    stop: AtomicBool,
    alive: AtomicUsize,
    /// A device reported an error worth surfacing when none is left.
    last_error: Mutex<Option<AudioError>>,
}

pub struct MixedStream {
    shared: Arc<Shared>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Opens every playback device of `source`; devices that fail to open are
/// skipped. Errors only when none opens.
pub fn open_all(
    source: &dyn AudioSource,
    kind: AudioSourceKind,
) -> Result<MixedStream, AudioError> {
    let streams: Vec<Box<dyn AudioStream>> = source
        .devices(kind)
        .into_iter()
        .filter_map(|d| source.open(kind, &DeviceSel::Id(d.id)).ok())
        .collect();
    MixedStream::new(streams)
}

impl MixedStream {
    pub fn new(streams: Vec<Box<dyn AudioStream>>) -> Result<Self, AudioError> {
        if streams.is_empty() {
            return Err(AudioError::NoDevice);
        }
        let shared = Arc::new(Shared {
            queues: streams.iter().map(|_| Mutex::new(Vec::new())).collect(),
            stop: AtomicBool::new(false),
            alive: AtomicUsize::new(streams.len()),
            last_error: Mutex::new(None),
        });
        let threads = streams
            .into_iter()
            .enumerate()
            .map(|(i, mut stream)| {
                let sh = shared.clone();
                std::thread::Builder::new()
                    .name(format!("aura-audio-mix-{i}"))
                    .spawn(move || {
                        let mut resampler: Option<(u32, Resampler)> = None;
                        while !sh.stop.load(Ordering::Relaxed) {
                            match stream.next_chunk() {
                                Some(Ok(RawChunk {
                                    samples,
                                    channels,
                                    rate,
                                    ..
                                })) => {
                                    if samples.is_empty() {
                                        continue;
                                    }
                                    let mono = to_mono(&samples, channels);
                                    let r = match &mut resampler {
                                        Some((r_rate, r)) if *r_rate == rate => r,
                                        _ => {
                                            &mut resampler
                                                .insert((rate, Resampler::new(rate, TARGET_RATE)))
                                                .1
                                        }
                                    };
                                    sh.queues[i].lock().unwrap().extend(r.process(&mono));
                                }
                                Some(Err(e)) => {
                                    *sh.last_error.lock().unwrap() = Some(e);
                                    break;
                                }
                                None => break,
                            }
                        }
                        sh.alive.fetch_sub(1, Ordering::SeqCst);
                    })
                    .expect("spawn mixer thread")
            })
            .collect();
        Ok(Self { shared, threads })
    }

    /// Number of devices still delivering audio.
    pub fn devices_alive(&self) -> usize {
        self.shared.alive.load(Ordering::SeqCst)
    }
}

impl AudioStream for MixedStream {
    fn next_chunk(&mut self) -> Option<Result<RawChunk, AudioError>> {
        std::thread::sleep(TICK);
        let mut mix: Vec<f32> = Vec::new();
        for q in &self.shared.queues {
            let taken = std::mem::take(&mut *q.lock().unwrap());
            if taken.len() > mix.len() {
                mix.resize(taken.len(), 0.0);
            }
            for (m, s) in mix.iter_mut().zip(taken) {
                *m += s;
            }
        }
        for s in &mut mix {
            *s = s.clamp(-1.0, 1.0);
        }
        if mix.is_empty() && self.shared.alive.load(Ordering::SeqCst) == 0 {
            return self.shared.last_error.lock().unwrap().take().map(Err);
        }
        Some(Ok(RawChunk {
            samples: mix,
            channels: 1,
            rate: TARGET_RATE,
            at_ms: now_ms(),
        }))
    }
}

impl Drop for MixedStream {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::tone_power;
    use crate::{DeviceInfo, hub::AudioHub};

    /// Two playback devices, each playing a different tone, plus one broken.
    struct TwoOutputs;

    struct Tone {
        freq: f32,
        t: i64,
    }

    impl AudioStream for Tone {
        fn next_chunk(&mut self) -> Option<Result<RawChunk, AudioError>> {
            if self.t >= 600 {
                return None;
            }
            std::thread::sleep(Duration::from_millis(20));
            let rate = 48_000u32;
            let n = (rate / 50) as usize;
            let samples = (0..n)
                .map(|i| {
                    let x = (self.t as f32 / 1000.0) + i as f32 / rate as f32;
                    0.3 * (2.0 * std::f32::consts::PI * self.freq * x).sin()
                })
                .collect();
            self.t += 20;
            Some(Ok(RawChunk {
                samples,
                channels: 1,
                rate,
                at_ms: self.t,
            }))
        }
    }

    impl AudioSource for TwoOutputs {
        fn devices(&self, _k: AudioSourceKind) -> Vec<DeviceInfo> {
            ["speakers", "headset", "broken"]
                .iter()
                .map(|n| DeviceInfo {
                    id: (*n).into(),
                    name: (*n).into(),
                    is_default: *n == "speakers",
                })
                .collect()
        }
        fn open(
            &self,
            _k: AudioSourceKind,
            d: &DeviceSel,
        ) -> Result<Box<dyn AudioStream>, AudioError> {
            match d {
                DeviceSel::Id(id) if id == "speakers" => Ok(Box::new(Tone { freq: 440.0, t: 0 })),
                DeviceSel::Id(id) if id == "headset" => Ok(Box::new(Tone { freq: 1000.0, t: 0 })),
                DeviceSel::Id(_) => Err(AudioError::NoDevice),
                DeviceSel::Default => Ok(Box::new(Tone { freq: 440.0, t: 0 })),
            }
        }
    }

    #[test]
    fn every_output_is_heard_at_once_and_broken_ones_are_skipped() {
        let mut mixed = open_all(&TwoOutputs, AudioSourceKind::SystemAudio).unwrap();
        let mut pcm: Vec<f32> = Vec::new();
        for _ in 0..8 {
            if let Some(Ok(c)) = mixed.next_chunk() {
                pcm.extend(c.samples);
            }
        }
        assert!(pcm.len() > 8_000, "got {}", pcm.len());
        let (low, high) = (
            tone_power(&pcm, TARGET_RATE, 440.0),
            tone_power(&pcm, TARGET_RATE, 1000.0),
        );
        let none = tone_power(&pcm, TARGET_RATE, 2500.0);
        assert!(
            low > 0.01 && high > 0.01,
            "both tones present: {low} {high}"
        );
        assert!(none < low / 10.0, "no other tone: {none}");
    }

    #[test]
    fn no_playback_device_is_an_error() {
        struct Nothing;
        impl AudioSource for Nothing {
            fn devices(&self, _k: AudioSourceKind) -> Vec<DeviceInfo> {
                vec![]
            }
            fn open(
                &self,
                _k: AudioSourceKind,
                _d: &DeviceSel,
            ) -> Result<Box<dyn AudioStream>, AudioError> {
                Err(AudioError::NoDevice)
            }
        }
        assert!(matches!(
            open_all(&Nothing, AudioSourceKind::SystemAudio),
            Err(AudioError::NoDevice)
        ));
    }

    #[tokio::test]
    async fn the_hub_opens_all_outputs_for_the_star_id() {
        let src = Arc::new(TwoOutputs);
        let mut hub = AudioHub::start(
            src,
            AudioSourceKind::SystemAudio,
            DeviceSel::Id(ALL_OUTPUTS.into()),
        )
        .unwrap();
        let mut rx = hub.subscribe();
        tokio::time::sleep(Duration::from_millis(900)).await;
        hub.stop();
        let mut pcm: Vec<f32> = Vec::new();
        while let Ok(c) = rx.try_recv() {
            pcm.extend(c.samples.iter());
        }
        assert!(tone_power(&pcm, TARGET_RATE, 440.0) > 0.01);
        assert!(tone_power(&pcm, TARGET_RATE, 1000.0) > 0.01);
    }
}
