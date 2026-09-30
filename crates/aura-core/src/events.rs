//! Application-level events published on the host bus.

use serde::{Deserialize, Serialize};

/// Actions coming from the tray menu, the second instance or global shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppEvent {
    OpenOverlay,
    ToggleOverlay,
    NewConversation,
    NewEphemeralConversation,
    OpenSettings,
    TogglePrivacyPause,
    Quit,
}

/// The window that was in the foreground right before the Overlay appeared
/// ("Aplicativo anterior" in `CONTEXT.md`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviousApp {
    /// Opaque window handle as an integer (HWND on Windows).
    pub window: u64,
    pub pid: u32,
    /// Executable name, e.g. `notepad.exe`.
    pub process_name: String,
    pub title: String,
    pub monitor_id: String,
}
