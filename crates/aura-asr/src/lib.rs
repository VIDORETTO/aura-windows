//! Speech recognition (`specs/006-voz-e-asr`).
//!
//! * [`catalog`]: verified model list and hardware-aware recommendation.
//! * [`download`]: resumable, verified, atomic model installation.
//! * [`transcriber`]: the `Transcriber` seam with worker, cloud and fake adapters.
//! * [`ptt`]: push-to-talk controller.

pub mod catalog;
pub mod cloud;
pub mod download;
pub mod ptt;
pub mod text;
pub mod transcriber;
pub mod worker_client;
