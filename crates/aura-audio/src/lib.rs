//! Audio pipeline (`specs/005-captura-de-audio`). OS capture (WASAPI mic and
//! loopback) implements [`AudioSource`] in `aura-win`.

pub mod clips;
pub mod dsp;
pub mod hub;
pub mod mixed;
#[cfg(feature = "store")]
pub mod recorder;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioSourceKind {
    Mic,
    SystemAudio,
}

impl AudioSourceKind {
    pub fn key(&self) -> &'static str {
        match self {
            AudioSourceKind::Mic => "mic",
            AudioSourceKind::SystemAudio => "system",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceSel {
    Default,
    Id(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AudioError {
    #[error("no audio device available")]
    NoDevice,
    #[error("microphone access denied in Windows privacy settings")]
    AccessDenied,
    #[error("the device was disconnected")]
    DeviceLost,
    #[error("os error: {0}")]
    Os(String),
}

/// Raw capture from a device: interleaved f32 at the device rate.
pub struct RawChunk {
    pub samples: Vec<f32>,
    pub channels: usize,
    pub rate: u32,
    pub at_ms: i64,
}

/// A running device capture. Dropping it stops the device.
pub trait AudioStream: Send {
    /// Next chunk, or `None` when the device stopped.
    fn next_chunk(&mut self) -> Option<Result<RawChunk, AudioError>>;
}

pub trait AudioSource: Send + Sync {
    fn devices(&self, kind: AudioSourceKind) -> Vec<DeviceInfo>;
    fn open(
        &self,
        kind: AudioSourceKind,
        device: &DeviceSel,
    ) -> Result<Box<dyn AudioStream>, AudioError>;
}
