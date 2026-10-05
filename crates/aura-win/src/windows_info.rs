//! Top-level window facts, inventory (EnumWindows), monitors, the foreground
//! ("Aplicativo anterior") tracker and overlay window helpers.

use crate::{from_wide, to_wide};
use aura_capture::source::WindowInventory;
use aura_core::events::PreviousApp;
use aura_core::placement::{Monitor, Rect};
use aura_policy::WindowInfo;
use std::ffi::c_void;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, POINT, RECT};
use windows::Win32::Graphics::Dwm::{
    DWM_SYSTEMBACKDROP_TYPE, DWM_WINDOW_CORNER_PREFERENCE, DWMSBT_NONE, DWMSBT_TRANSIENTWINDOW,
    DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DWMWA_SYSTEMBACKDROP_TYPE,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmGetWindowAttribute, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO,
    MONITORINFOEXW, MonitorFromPoint, MonitorFromWindow,
};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentThreadId, OpenProcess, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GWL_EXSTYLE, GetClassNameW, GetCursorPos, GetForegroundWindow,
    GetWindowDisplayAffinity, GetWindowLongW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SW_RESTORE, SetForegroundWindow,
    SetWindowDisplayAffinity, ShowWindow, WDA_EXCLUDEFROMCAPTURE, WDA_NONE, WS_EX_TOOLWINDOW,
};
use windows::core::{BOOL, PWSTR};

pub fn hwnd(id: u64) -> HWND {
    HWND(id as usize as *mut c_void)
}

pub fn id_of(h: HWND) -> u64 {
    h.0 as usize as u64
}

fn rect_of(r: RECT) -> Rect {
    Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top)
}

pub fn title(h: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(h);
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u16; len as usize + 1];
        let n = GetWindowTextW(h, &mut buf);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

pub fn class_name(h: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(h, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

pub fn pid_of(h: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(h, Some(&mut pid)) };
    pid
}

/// Executable file name (`notepad.exe`) of a process, if accessible.
pub fn process_name(pid: u32) -> String {
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
        let mut buf = vec![0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if !ok {
            return String::new();
        }
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        path.rsplit(['\\', '/'])
            .next()
            .unwrap_or_default()
            .to_string()
    }
}

/// Visible bounds without the invisible resize borders (DWM frame bounds).
pub fn bounds(h: HWND) -> Option<Rect> {
    unsafe {
        let mut r = RECT::default();
        if DwmGetWindowAttribute(
            h,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut r as *mut _ as *mut c_void,
            std::mem::size_of::<RECT>() as u32,
        )
        .is_ok()
        {
            return Some(rect_of(r));
        }
        GetWindowRect(h, &mut r).ok()?;
        Some(rect_of(r))
    }
}

pub fn is_cloaked(h: HWND) -> bool {
    let mut cloaked = 0u32;
    unsafe {
        DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut c_void, 4).is_ok()
            && cloaked != 0
    }
}

pub fn is_minimized(h: HWND) -> bool {
    unsafe { IsIconic(h).as_bool() }
}

pub fn exists(h: HWND) -> bool {
    unsafe { IsWindow(Some(h)).as_bool() }
}

pub fn info(h: HWND) -> Option<WindowInfo> {
    if !exists(h) {
        return None;
    }
    let pid = pid_of(h);
    Some(WindowInfo {
        id: id_of(h),
        pid,
        process: process_name(pid),
        title: title(h),
        class: class_name(h),
        rect: bounds(h)?,
    })
}

/// Windows a user would consider "on screen": visible, not cloaked (other
/// virtual desktop / suspended UWP), not tool windows, non-empty.
fn is_user_window(h: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(h).as_bool() || is_cloaked(h) || IsIconic(h).as_bool() {
            return false;
        }
        let ex = GetWindowLongW(h, GWL_EXSTYLE) as u32;
        if ex & WS_EX_TOOLWINDOW.0 != 0 {
            return false;
        }
    }
    bounds(h).is_some_and(|r| r.w > 1 && r.h > 1)
}

pub fn top_level_windows() -> Vec<HWND> {
    unsafe extern "system" fn collect(h: HWND, lp: LPARAM) -> BOOL {
        let out = unsafe { &mut *(lp.0 as *mut Vec<HWND>) };
        out.push(h);
        BOOL(1)
    }
    let mut out: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// `WindowInventory` over the live desktop (z-order, front to back).
#[derive(Default)]
pub struct Win32Inventory;

impl WindowInventory for Win32Inventory {
    fn visible_windows(&self, area: Rect) -> Vec<WindowInfo> {
        top_level_windows()
            .into_iter()
            .filter(|h| is_user_window(*h))
            .filter_map(info)
            .filter(|w| w.rect.intersect(&area).is_some())
            .collect()
    }

    fn window(&self, id: u64) -> Option<WindowInfo> {
        info(hwnd(id))
    }
}

fn monitor_info(m: HMONITOR) -> Option<(String, Rect, Rect)> {
    unsafe {
        let mut mi = MONITORINFOEXW::default();
        mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if !GetMonitorInfoW(m, &mut mi as *mut _ as *mut MONITORINFO).as_bool() {
            return None;
        }
        Some((
            from_wide(&mi.szDevice),
            rect_of(mi.monitorInfo.rcMonitor),
            rect_of(mi.monitorInfo.rcWork),
        ))
    }
}

fn dpi_of(m: HMONITOR) -> u32 {
    let (mut x, mut y) = (96u32, 96u32);
    unsafe {
        let _ = GetDpiForMonitor(m, MDT_EFFECTIVE_DPI, &mut x, &mut y);
    }
    x
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorDesc {
    pub monitor: Monitor,
    /// Full monitor area (capture target), physical pixels.
    pub area: Rect,
}

pub fn monitors() -> Vec<MonitorDesc> {
    unsafe extern "system" fn collect(m: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
        let out = unsafe { &mut *(lp.0 as *mut Vec<HMONITOR>) };
        out.push(m);
        BOOL(1)
    }
    let mut handles: Vec<HMONITOR> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut handles as *mut _ as isize),
        );
    }
    handles
        .into_iter()
        .filter_map(|m| {
            let (id, area, work) = monitor_info(m)?;
            Some(MonitorDesc {
                monitor: Monitor {
                    id,
                    work_area: work,
                    dpi: dpi_of(m),
                },
                area,
            })
        })
        .collect()
}

pub fn cursor_pos() -> (i32, i32) {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

/// Monitor under the mouse cursor (where the Overlay opens).
pub fn monitor_at_cursor() -> Option<MonitorDesc> {
    let (x, y) = cursor_pos();
    let m = unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST) };
    let (id, _, _) = monitor_info(m)?;
    monitors().into_iter().find(|d| d.monitor.id == id)
}

fn monitor_id_of(h: HWND) -> String {
    let m = unsafe { MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST) };
    monitor_info(m).map(|(id, _, _)| id).unwrap_or_default()
}

/// Snapshot of the foreground window, taken right before the Overlay shows.
pub fn foreground_app() -> Option<PreviousApp> {
    let h = unsafe { GetForegroundWindow() };
    if h.0.is_null() {
        return None;
    }
    let pid = pid_of(h);
    Some(PreviousApp {
        window: id_of(h),
        pid,
        process_name: process_name(pid),
        title: title(h),
        monitor_id: monitor_id_of(h),
    })
}

/// Gives focus back to the previous app (dictation insertion, Esc).
/// Windows only lets the foreground thread change focus, so we briefly
/// attach our input queue to it.
pub fn restore_focus(prev: &PreviousApp) -> bool {
    let target = hwnd(prev.window);
    if !exists(target) {
        return false;
    }
    unsafe {
        if IsIconic(target).as_bool() {
            let _ = ShowWindow(target, SW_RESTORE);
        }
        let fg = GetForegroundWindow();
        let fg_thread = GetWindowThreadProcessId(fg, None);
        let me = GetCurrentThreadId();
        let attached =
            fg_thread != 0 && fg_thread != me && AttachThreadInput(me, fg_thread, true).as_bool();
        let _ = BringWindowToTop(target);
        let ok = SetForegroundWindow(target).as_bool();
        if attached {
            let _ = AttachThreadInput(me, fg_thread, false);
        }
        ok
    }
}

/// Keeps the Overlay out of screenshots, recordings and screen sharing — and
/// out of Aura's own captures (FR-006 of 004).
pub fn exclude_from_capture(window: u64, exclude: bool) -> bool {
    unsafe {
        SetWindowDisplayAffinity(
            hwnd(window),
            if exclude {
                WDA_EXCLUDEFROMCAPTURE
            } else {
                WDA_NONE
            },
        )
        .is_ok()
    }
}

/// Whether the OS currently keeps `window` out of captures
/// (`WDA_EXCLUDEFROMCAPTURE`); `None` when the window cannot be read.
pub fn is_excluded_from_capture(window: u64) -> Option<bool> {
    let mut affinity = 0u32;
    unsafe {
        GetWindowDisplayAffinity(hwnd(window), &mut affinity)
            .ok()
            .map(|_| affinity == WDA_EXCLUDEFROMCAPTURE.0)
    }
}

/// Windows "Transparency effects" (Personalization › Colors); missing = on.
pub fn transparency_effects_enabled() -> bool {
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            windows::core::w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"),
            windows::core::w!("EnableTransparency"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut c_void),
            Some(&mut size),
        )
    };
    !r.is_ok() || value != 0
}

/// Rounded corners + acrylic-like transient backdrop (Windows 11; no-op on 10).
/// With transparency effects off DWM paints that backdrop as a solid fill,
/// which hides the Overlay's own opacity: then no system backdrop at all.
pub fn apply_overlay_chrome(window: u64) {
    let h = hwnd(window);
    unsafe {
        let corner: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            h,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner as *const _ as *const c_void,
            4,
        );
        let backdrop: DWM_SYSTEMBACKDROP_TYPE = if transparency_effects_enabled() {
            DWMSBT_TRANSIENTWINDOW
        } else {
            DWMSBT_NONE
        };
        let _ = DwmSetWindowAttribute(
            h,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const _ as *const c_void,
            4,
        );
    }
}

/// Opens a file or folder with the shell ("revelar na pasta" uses explorer).
pub fn shell_open(path: &str) -> bool {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let op = to_wide("open");
    let file = to_wide(path);
    let r = unsafe {
        ShellExecuteW(
            None,
            windows::core::PCWSTR(op.as_ptr()),
            windows::core::PCWSTR(file.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    r.0 as isize > 32
}
