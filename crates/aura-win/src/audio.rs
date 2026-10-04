//! `AudioSource` over WASAPI via `cpal`: microphone capture and system-audio
//! loopback (an input stream opened on an output device).
//!
//! `cpal::Stream` is not `Send`, so each stream lives on its own thread; the
//! handle returned to the hub only owns channels. Device ids are the
//! friendly names (cpal 0.16 has no stable endpoint id) — see HANDOFF.

use aura_audio::{
    AudioError, AudioSource, AudioSourceKind, AudioStream, DeviceInfo, DeviceSel, RawChunk,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn map_err(msg: String) -> AudioError {
    let lower = msg.to_lowercase();
    if lower.contains("0x80070005") || lower.contains("access") && lower.contains("denied") {
        AudioError::AccessDenied
    } else if lower.contains("not available")
        || lower.contains("no longer")
        || lower.contains("0x88890004")
    {
        AudioError::DeviceLost
    } else {
        AudioError::Os(msg)
    }
}

#[derive(Default)]
pub struct WasapiSource;

fn devices_of(kind: AudioSourceKind) -> (Vec<Device>, Option<String>) {
    let host = cpal::default_host();
    match kind {
        AudioSourceKind::Mic => (
            host.input_devices()
                .map(|d| d.collect())
                .unwrap_or_default(),
            host.default_input_device().and_then(|d| d.name().ok()),
        ),
        AudioSourceKind::SystemAudio => (
            host.output_devices()
                .map(|d| d.collect())
                .unwrap_or_default(),
            host.default_output_device().and_then(|d| d.name().ok()),
        ),
    }
}

fn pick(kind: AudioSourceKind, sel: &DeviceSel) -> Result<Device, AudioError> {
    let host = cpal::default_host();
    let default = || match kind {
        AudioSourceKind::Mic => host.default_input_device(),
        AudioSourceKind::SystemAudio => host.default_output_device(),
    };
    match sel {
        DeviceSel::Default => default().ok_or(AudioError::NoDevice),
        DeviceSel::Id(id) => {
            let (list, _) = devices_of(kind);
            // A missing chosen device is reported; AudioHub opens the
            // default and the app warns the user (005 AC-002).
            list.into_iter()
                .find(|d| d.name().ok().as_deref() == Some(id))
                .ok_or(AudioError::NoDevice)
        }
    }
}

impl AudioSource for WasapiSource {
    fn devices(&self, kind: AudioSourceKind) -> Vec<DeviceInfo> {
        let (list, default) = devices_of(kind);
        list.into_iter()
            .filter_map(|d| d.name().ok())
            .map(|name| DeviceInfo {
                id: name.clone(),
                is_default: default.as_deref() == Some(&name),
                name,
            })
            .collect()
    }

    fn open(
        &self,
        kind: AudioSourceKind,
        device: &DeviceSel,
    ) -> Result<Box<dyn AudioStream>, AudioError> {
        let (data_tx, data_rx) = sync_channel::<Result<RawChunk, AudioError>>(256);
        let (ready_tx, ready_rx) = sync_channel::<Result<(), AudioError>>(1);
        let (stop_tx, stop_rx) = sync_channel::<()>(1);
        let device = device.clone();
        std::thread::Builder::new()
            .name(format!("aura-audio-{}", kind.key()))
            .spawn(move || {
                let stream = match build(kind, &device, data_tx) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                if let Err(e) = stream.play() {
                    let _ = ready_tx.send(Err(map_err(e.to_string())));
                    return;
                }
                let _ = ready_tx.send(Ok(()));
                // Park until the handle is dropped; dropping `stream` stops WASAPI.
                let _ = stop_rx.recv();
                drop(stream);
            })
            .map_err(|e| AudioError::Os(e.to_string()))?;
        ready_rx
            .recv()
            .map_err(|_| AudioError::Os("audio thread died".into()))??;
        Ok(Box::new(WasapiStream {
            rx: data_rx,
            _stop: stop_tx,
            format: (2, 48_000),
        }))
    }
}

fn build(
    kind: AudioSourceKind,
    sel: &DeviceSel,
    tx: SyncSender<Result<RawChunk, AudioError>>,
) -> Result<cpal::Stream, AudioError> {
    let dev = pick(kind, sel)?;
    let supported = match kind {
        AudioSourceKind::Mic => dev.default_input_config(),
        AudioSourceKind::SystemAudio => dev.default_output_config(),
    }
    .map_err(|e| map_err(e.to_string()))?;
    let format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let channels = config.channels as usize;
    let rate = config.sample_rate.0;

    let err_tx = tx.clone();
    let on_error = move |e: cpal::StreamError| {
        let _ = err_tx.try_send(Err(map_err(e.to_string())));
    };
    // Drop chunks rather than block the real-time callback if the hub lags.
    let push = move |samples: Vec<f32>| match tx.try_send(Ok(RawChunk {
        samples,
        channels,
        rate,
        at_ms: now_ms(),
    })) {
        Ok(()) | Err(TrySendError::Full(_)) => {}
        Err(TrySendError::Disconnected(_)) => {}
    };
    let stream = match format {
        SampleFormat::F32 => dev.build_input_stream(
            &config,
            move |d: &[f32], _: &cpal::InputCallbackInfo| push(d.to_vec()),
            on_error,
            None,
        ),
        SampleFormat::I16 => dev.build_input_stream(
            &config,
            move |d: &[i16], _: &cpal::InputCallbackInfo| {
                push(d.iter().map(|s| *s as f32 / 32768.0).collect())
            },
            on_error,
            None,
        ),
        SampleFormat::I32 => dev.build_input_stream(
            &config,
            move |d: &[i32], _: &cpal::InputCallbackInfo| {
                push(d.iter().map(|s| *s as f32 / 2_147_483_648.0).collect())
            },
            on_error,
            None,
        ),
        SampleFormat::U16 => dev.build_input_stream(
            &config,
            move |d: &[u16], _: &cpal::InputCallbackInfo| {
                push(d.iter().map(|s| (*s as f32 - 32768.0) / 32768.0).collect())
            },
            on_error,
            None,
        ),
        other => {
            return Err(AudioError::Os(format!(
                "unsupported sample format {other:?}"
            )));
        }
    };
    stream.map_err(|e| map_err(e.to_string()))
}

struct WasapiStream {
    rx: Receiver<Result<RawChunk, AudioError>>,
    _stop: SyncSender<()>,
    /// Format of the last chunk, for the empty chunks of a silent device.
    format: (usize, u32),
}

/// Loopback capture delivers no packets while nothing plays. Waiting is
/// bounded so the reader can notice a stop request (and not hang its join).
const IDLE_WAIT: std::time::Duration = std::time::Duration::from_millis(100);

impl AudioStream for WasapiStream {
    fn next_chunk(&mut self) -> Option<Result<RawChunk, AudioError>> {
        match self.rx.recv_timeout(IDLE_WAIT) {
            Ok(Ok(chunk)) => {
                self.format = (chunk.channels, chunk.rate);
                Some(Ok(chunk))
            }
            Ok(Err(e)) => Some(Err(e)),
            Err(RecvTimeoutError::Timeout) => Some(Ok(RawChunk {
                samples: Vec::new(),
                channels: self.format.0,
                rate: self.format.1,
                at_ms: now_ms(),
            })),
            Err(RecvTimeoutError::Disconnected) => None,
        }
    }
}
