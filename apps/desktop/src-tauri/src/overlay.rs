//! The Overlay window: pre-created hidden (tauri.conf.json), positioned on the
//! monitor of the previous app, shown without recreating the WebView (the
//! 100 ms hotkey→visible budget of 001).

use aura_app::host::{Host, OverlayMode, SavedPlacement};
use aura_core::placement::{MIN_EXPANDED, Monitor, Rect, monitor_for_rect, place_overlay};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindow,
};

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

fn apply_minimum(w: &WebviewWindow, mode: OverlayMode) {
    let height = match mode {
        OverlayMode::Compact => 64.0,
        OverlayMode::Expanded => f64::from(MIN_EXPANDED.1),
    };
    let scale = w.scale_factor().unwrap_or(1.0);
    let mut minimum = LogicalSize::new(480.0, height);
    // Tao's undecorated Windows track size omits the invisible native frame.
    // Include its measured extent so the CLIENT area meets our logical minimum.
    if let (Ok(outer), Ok(inner)) = (w.outer_size(), w.inner_size()) {
        minimum.width += f64::from(outer.width.saturating_sub(inner.width)) / scale;
        minimum.height += f64::from(outer.height.saturating_sub(inner.height)) / scale;
    }
    if let Err(e) = w.set_min_size(Some(minimum)) {
        tracing::warn!("could not set overlay minimum size: {e}");
    }
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

/// While the Overlay is visible but not focused, keep reading the app in
/// front and its selection, so returning to the Overlay brings the current
/// selection without hiding and showing it (012 AC-001).
pub fn track_previous_app(app: &AppHandle) {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    let Some(w) = window(app) else { return };
    let away = Arc::new(AtomicBool::new(false));
    let flag = away.clone();
    w.on_window_event(move |e| {
        if let tauri::WindowEvent::Focused(focused) = e {
            flag.store(!focused, Ordering::SeqCst);
        }
    });
    let app = app.clone();
    std::thread::Builder::new()
        .name("aura-previous-app".into())
        .spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(300));
                if away.load(Ordering::SeqCst) && is_visible(&app) {
                    let _ = host(&app).prepare_overlay();
                }
            }
        })
        .ok();
}

pub fn show(app: &AppHandle) {
    let Some(w) = window(app) else { return };
    let h = host(app);
    // Snapshot BEFORE showing: afterwards the foreground window is ours.
    let prev = h.prepare_overlay();
    let mode = *MODE.lock().unwrap();
    apply_minimum(&w, mode);
    let monitor = target_monitor(app, prev.as_ref());
    let saved = h.saved_placement(&monitor, mode);
    if let Some(r) = place_overlay(&monitors(app), &monitor, saved.as_ref(), mode) {
        apply_rect(&w, r);
    }
    // Re-applied on every show: follows the Windows transparency setting.
    #[cfg(windows)]
    if let Ok(hwnd) = w.hwnd() {
        aura_win::windows_info::apply_overlay_chrome(hwnd.0 as usize as u64);
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
    // Read before changing the minimum, which can itself resize the window.
    let current = window_rect(&w);
    let h = host(app);
    let mons = monitors(app);
    let current_monitor = current.and_then(|rect| monitor_for_rect(&mons, rect));
    let monitor = current_monitor
        .map(|monitor| monitor.id.clone())
        .unwrap_or_else(|| target_monitor(app, h.previous_app().as_ref()));
    apply_minimum(&w, mode);
    let saved = h.saved_placement(&monitor, mode);
    let Some(mut r) = place_overlay(&mons, &monitor, saved.as_ref(), mode) else {
        return;
    };
    if let (Some(current), Some(monitor)) = (current, current_monitor) {
        // A saved mode supplies its size, never a new anchor during a conversation.
        r.x = current.x;
        r.y = current.y;
        r = aura_core::placement::clamp_into(r, monitor.work_area);
    }
    apply_rect(&w, r);
}

fn window_rect(w: &WebviewWindow) -> Option<Rect> {
    let pos = w.outer_position().ok()?;
    let size = w.inner_size().ok()?;
    Some(Rect::new(
        pos.x,
        pos.y,
        size.width as i32,
        size.height as i32,
    ))
}

/// Called by the UI after the user moved/resized the Overlay.
pub fn remember_placement(app: &AppHandle, h: &Host) {
    let Some(w) = window(app) else { return };
    // Inner size: `apply_rect` restores it with `set_size` (inner). The outer
    // size includes the invisible resize borders and would grow every reopen.
    let (Ok(pos), Ok(size)) = (w.outer_position(), w.inner_size()) else {
        return;
    };
    let rect = Rect::new(pos.x, pos.y, size.width as i32, size.height as i32);
    let mons = monitors(app);
    let monitor = monitor_for_rect(&mons, rect)
        .map(|monitor| monitor.id.clone())
        .unwrap_or_else(|| target_monitor(app, h.previous_app().as_ref()));
    let dpi = w
        .scale_factor()
        .map(|f| (f * 96.0).round() as u32)
        .unwrap_or(96);
    let mode = *MODE.lock().unwrap();
    let placement = SavedPlacement {
        monitor_id: monitor,
        rect,
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
        // Visual QA screenshots of the demo build only (never in a shipped build).
        let capturable = cfg!(feature = "demo") && std::env::var_os("AURA_QA_CAPTURABLE").is_some();
        let hide = host(app).settings().hide_from_capture;
        if !capturable && hide && !aura_win::windows_info::exclude_from_capture(id, true) {
            tracing::warn!("SetWindowDisplayAffinity failed: overlay may appear in captures");
        }
        aura_win::windows_info::apply_overlay_chrome(id);
    }
    #[cfg(not(windows))]
    let _ = app;
}

/// Applies "hide from screen sharing" to every Aura window that exists now
/// (Overlay, Settings, region selector). Windows opened later read the setting
/// when they are created.
pub fn apply_capture_exclusion(app: &AppHandle, hide: bool) {
    #[cfg(windows)]
    for label in [LABEL, "settings", "region"] {
        if let Some(w) = app.get_webview_window(label)
            && let Ok(hwnd) = w.hwnd()
            && !aura_win::windows_info::exclude_from_capture(hwnd.0 as usize as u64, hide)
        {
            tracing::warn!("SetWindowDisplayAffinity failed for {label}");
        }
    }
    #[cfg(not(windows))]
    let _ = (app, hide);
}
