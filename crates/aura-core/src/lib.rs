//! Aura core: domain vocabulary (see `CONTEXT.md`) and pure rules.
//!
//! Nothing in this crate touches Windows APIs; OS-specific behaviour lives in
//! adapter crates (`aura-win`, `aura-capture`, `aura-audio`) behind traits.

pub mod context;
pub mod credentials;
pub mod events;
pub mod gesture;
pub mod jsonrpc;
pub mod logging;
pub mod placement;
pub mod secret;
pub mod settings;

pub use secret::Secret;
