//! The Overlay window: pre-created hidden (tauri.conf.json), positioned on the
//! monitor of the previous app, shown without recreating the WebView (the
//! 100 ms hotkey→visible budget of 001).

use aura_app::host::{Host, OverlayMode, SavedPlacement};
use aura_core::placement::{Monitor, Rect, place_overlay};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

pub const LABEL: &str = "overlay";

static MODE: Mutex<OverlayMode> = Mutex::new(OverlayMode::Compact);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Shown {
    previous_app: Option<aura_core::events::PreviousApp>,
    mode: OverlayMode,
}

pub fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

fn host(app: &AppHandle) -> std::sync::Arc<Host> {
    app.state::<crate::commands::AppState>().host.clone()
}

/// Monitors with ids matching `PreviousApp::monitor_id`.
fn monitors(app: &AppHandle) -> Vec<Monitor> {
    #[cfg(windows)]
    {
        let _ = app;
        aura_win::windows_info::monitors()
            .into_iter()
            .map(|d| d.monitor)
            .collect()
    }
    #[cfg(not(windows))]
    {
        app.available_monitors()
            .unwrap_or_default()
            .into_iter()
            .map(|m| Monitor {
                id: m.name().cloned().unwrap_or_default(),
                work_area: Rect::new(
                    m.position().x,
                    m.position().y,
                    m.size().width as i32,
                    m.size().height as i32,
                ),
                dpi: (m.scale_factor() * 96.0).round() as u32,
            })
            .collect()
    }
}

fn apply_rect(w: &WebviewWindow, r: Rect) {
    let _ = w.set_size(PhysicalSize::new(r.w.max(1) as u32, r.h.max(1) as u32));
    let _ = w.set_position(PhysicalPosition::new(r.x, r.y));
}

fn target_monitor(app: &AppHandle, prev: Option<&aura_core::events::PreviousApp>) -> String {
    if let Some(p) = prev.filter(|p| !p.monitor_id.is_empty()) {
        return p.monitor_id.clone();
    }
    #[cfg(windows)]
    {
        let _ = app;
        aura_win::windows_info::monitor_at_cursor()
            .map(|m| m.monitor.id)
            .unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        app.primary_monitor()
            .ok()
            .flatten()
            .and_then(|m| m.name().cloned())
            .unwrap_or_default()
    }
}

pub fn is_visible(app: &AppHandle) -> bool {
    window(app)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

pub fn show(app: &AppHandle) {
    let Some(w) = window(app) else { return };
    let h = host(app);
    // Snapshot BEFORE showing: afterwards the foreground window is ours.
    let prev = h.prepare_overlay();
    let mode = *MODE.lock().unwrap();
    let monitor = target_monitor(app, prev.as_ref());
    let saved = h.saved_placement(&monitor, mode);
    if let Some(r) = place_overlay(&monitors(app), &monitor, saved.as_ref(), mode) {
        apply_rect(&w, r);
    }
    let _ = w.show();
    let _ = w.set_focus();
    #[cfg(windows)]
    aura_win::hotkeys::set_push_to_talk_armed(true);
    let _ = app.emit_to(
        LABEL,
        "aura://overlay",
        Shown {
            previous_app: prev,
            mode,
        },
    );
}

pub fn hide(app: &AppHandle) {
    #[cfg(windows)]
    aura_win::hotkeys::set_push_to_talk_armed(false);
    if let Some(w) = window(app) {
        let _ = w.hide();
        let _ = app.emit_to(LABEL, "aura://overlay-hidden", ());
    }
}

pub fn toggle(app: &AppHandle) {
    let focused = window(app)
        .and_then(|w| w.is_focused().ok())
        .unwrap_or(false);
    if is_visible(app) && focused {
        hide(app)
    } else {
        show(app)
    }
}

/// Compact ↔ expanded, keeping the top edge where the user left it.
pub fn set_mode(app: &AppHandle, mode: OverlayMode) {
    *MODE.lock().unwrap() = mode;
    let Some(w) = window(app) else { return };
    let h = host(app);
    let monitor = target_monitor(app, h.previous_app().as_ref());
    let mons = monitors(app);
    let saved = h.saved_placement(&monitor, mode);
    let Some(mut r) = place_overlay(&mons, &monitor, saved.as_ref(), mode) else {
        return;
    };
    if saved.is_none()
        && let Ok(pos) = w.outer_position()
    {
        r.x = pos.x;
        r.y = pos.y;
        if let Some(m) = mons.iter().find(|m| m.id == monitor) {
            r = aura_core::placement::clamp_into(r, m.work_area);
        }
    }
    apply_rect(&w, r);
}

/// Called by the UI after the user moved/resized the Overlay.
pub fn remember_placement(app: &AppHandle, h: &Host) {
    let Some(w) = window(app) else { return };
    let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) else {
        return;
    };
    let monitor = target_monitor(app, h.previous_app().as_ref());
    let dpi = w
        .scale_factor()
        .map(|f| (f * 96.0).round() as u32)
        .unwrap_or(96);
    let mode = *MODE.lock().unwrap();
    let placement = SavedPlacement {
        monitor_id: monitor,
        rect: Rect::new(pos.x, pos.y, size.width as i32, size.height as i32),
        dpi,
    };
    if let Err(e) = h.save_placement(&placement, mode) {
        tracing::warn!("could not save overlay placement: {e}");
    }
}

/// Windows-only chrome: invisible to screen capture, rounded, backdrop.
pub fn decorate(app: &AppHandle) {
    #[cfg(windows)]
    if let Some(w) = window(app)
        && let Ok(hwnd) = w.hwnd()
    {
        let id = hwnd.0 as usize as u64;
        if !aura_win::windows_info::exclude_from_capture(id, true) {
            tracing::warn!("SetWindowDisplayAffinity failed: overlay may appear in captures");
        }
        aura_win::windows_info::apply_overlay_chrome(id);
    }
    #[cfg(not(windows))]
    let _ = app;
}
