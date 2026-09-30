//! `FrameSource` over GDI (`BitBlt` for monitors, `PrintWindow` with
//! `PW_RENDERFULLCONTENT` for windows).
//!
//! GDI was chosen over Windows.Graphics.Capture for the first release: no
//! yellow capture border, no D3D device, works on Windows 10 2004+, and one
//! monitor frame costs ~15–40 ms, which fits on-demand capture and the 1 fps
//! background buffer (ADR 0004). Windows excluded with
//! `WDA_EXCLUDEFROMCAPTURE` (the Overlay) are left out by the OS.
//! Requires a per-monitor-v2 DPI-aware process (the Tauri manifest sets it).

use crate::windows_info::{bounds, exists, hwnd, is_minimized};
use aura_capture::frame::Frame;
use aura_capture::source::{CaptureError, FrameSource, Target};
use aura_core::placement::Rect;
use std::ffi::c_void;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC,
    HGDIOBJ, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

const PW_RENDERFULLCONTENT: u32 = 0x0000_0002;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn os(e: impl std::fmt::Display) -> CaptureError {
    CaptureError::Os(e.to_string())
}

/// Owns a memory DC + bitmap and frees them on drop.
struct Canvas {
    screen: HDC,
    mem: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    w: i32,
    h: i32,
}

impl Canvas {
    fn new(w: i32, h: i32) -> Result<Self, CaptureError> {
        if w <= 0 || h <= 0 {
            return Err(CaptureError::Os("empty area".into()));
        }
        unsafe {
            let screen = GetDC(None);
            if screen.is_invalid() {
                return Err(os("GetDC failed"));
            }
            let mem = CreateCompatibleDC(Some(screen));
            let bmp = CreateCompatibleBitmap(screen, w, h);
            if mem.is_invalid() || bmp.is_invalid() {
                ReleaseDC(None, screen);
                return Err(os("CreateCompatibleBitmap failed"));
            }
            let old = SelectObject(mem, bmp.into());
            Ok(Self {
                screen,
                mem,
                bmp,
                old,
                w,
                h,
            })
        }
    }

    fn read(&self) -> Result<Frame, CaptureError> {
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: self.w,
                biHeight: -self.h, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; (self.w * self.h * 4) as usize];
        let lines = unsafe {
            GetDIBits(
                self.mem,
                self.bmp,
                0,
                self.h as u32,
                Some(bgra.as_mut_ptr() as *mut c_void),
                &mut info,
                DIB_RGB_COLORS,
            )
        };
        if lines == 0 {
            return Err(os("GetDIBits failed"));
        }
        for px in bgra.as_chunks_mut::<4>().0 {
            px[3] = 0xff;
        }
        Ok(Frame {
            width: self.w as u32,
            height: self.h as u32,
            bgra,
            captured_at_ms: now_ms(),
        })
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.mem, self.old);
            let _ = DeleteObject(self.bmp.into());
            let _ = DeleteDC(self.mem);
            ReleaseDC(None, self.screen);
        }
    }
}

fn grab_screen(area: Rect) -> Result<Frame, CaptureError> {
    let c = Canvas::new(area.w, area.h)?;
    unsafe {
        BitBlt(
            c.mem,
            0,
            0,
            area.w,
            area.h,
            Some(c.screen),
            area.x,
            area.y,
            SRCCOPY | CAPTUREBLT,
        )
    }
    .map_err(os)?;
    c.read()
}

/// Mostly-black frames mean PrintWindow could not render (some GPU apps).
fn looks_blank(f: &Frame) -> bool {
    let step = (f.bgra.len() / 4 / 997).max(1) * 4;
    let lit = f
        .bgra
        .as_chunks::<4>()
        .0
        .iter()
        .step_by(step / 4)
        .filter(|p| p[0] > 8 || p[1] > 8 || p[2] > 8)
        .count();
    lit == 0
}

fn grab_window(h: HWND) -> Result<Frame, CaptureError> {
    if !exists(h) {
        return Err(CaptureError::WindowGone);
    }
    if is_minimized(h) {
        return Err(CaptureError::WindowMinimized);
    }
    let visible = bounds(h).ok_or(CaptureError::WindowGone)?;
    let mut wr = RECT::default();
    unsafe { GetWindowRect(h, &mut wr) }.map_err(|_| CaptureError::WindowGone)?;
    let (ww, wh) = (wr.right - wr.left, wr.bottom - wr.top);
    let c = Canvas::new(ww, wh)?;
    let printed =
        unsafe { PrintWindow(h, c.mem, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)) }.as_bool();
    let full = if printed { Some(c.read()?) } else { None };
    let frame = match full {
        Some(f) if !looks_blank(&f) => {
            // Crop the invisible resize borders (DWM frame vs window rect).
            let crop = Rect::new(
                visible.x - wr.left,
                visible.y - wr.top,
                visible.w,
                visible.h,
            );
            f.crop(crop).unwrap_or(f)
        }
        // Fallback: copy what is on screen at the window position.
        _ => grab_screen(visible)?,
    };
    Ok(frame)
}

#[derive(Default)]
pub struct GdiSource;

impl FrameSource for GdiSource {
    fn capture(&self, target: &Target) -> Result<Frame, CaptureError> {
        match target {
            Target::Monitor { area, .. } => grab_screen(*area),
            Target::Window { window } => grab_window(hwnd(*window)),
        }
    }
}
