//! Windows adapters for the OS seams defined in `aura-core`, `aura-capture`
//! and `aura-audio`.
//!
//! Everything that touches Win32/WinRT is `cfg(windows)`. Pure helpers
//! (key mapping, credential chunking) are portable and unit-tested on any OS.
//! See `docs/HANDOFF.md` for what was only type-checked from Linux and must be
//! validated on a Windows machine.

pub mod chunks;
pub mod keys;

#[cfg(windows)]
pub mod audio;
#[cfg(windows)]
pub mod capture;
#[cfg(windows)]
pub mod credman;
#[cfg(windows)]
pub mod encoder;
#[cfg(windows)]
pub mod hotkeys;
#[cfg(windows)]
pub mod input;
#[cfg(windows)]
pub mod job;
#[cfg(windows)]
pub mod media;
#[cfg(windows)]
pub mod ocr;
#[cfg(windows)]
pub mod speech;
#[cfg(windows)]
pub mod system;
#[cfg(windows)]
pub mod uia;
#[cfg(windows)]
pub mod windows_info;

/// Converts a wide (UTF-16) buffer up to the first NUL.
pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// NUL-terminated UTF-16 for Win32 `PCWSTR` parameters.
pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
