//! Clipboard, synthetic keyboard input and selection capture.
//!
//! Selection capture tries UI Automation first; only if the focused control
//! has no TextPattern it falls back to Ctrl+C, restoring the user's
//! clipboard afterwards. Text insertion (dictation, "inserir no app") pastes
//! via the clipboard, which is the only reliable way across Win32, UWP,
//! Electron and browser controls; `type_text` is used for short strings.

use crate::uia;
use std::time::Duration;
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_CONTROL,
};

struct ClipboardGuard;

impl ClipboardGuard {
    fn open() -> Option<Self> {
        // Another app may hold the clipboard briefly.
        for _ in 0..10 {
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Some(ClipboardGuard);
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        None
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub fn clipboard_text() -> Option<String> {
    let _g = ClipboardGuard::open()?;
    unsafe {
        let h = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
        let hg = HGLOBAL(h.0);
        let ptr = GlobalLock(hg) as *const u16;
        if ptr.is_null() {
            return None;
        }
        let len = GlobalSize(hg) / 2;
        let slice = std::slice::from_raw_parts(ptr, len);
        let text = crate::from_wide(slice);
        let _ = GlobalUnlock(hg);
        Some(text)
    }
}

pub fn set_clipboard_text(text: &str) -> bool {
    let Some(_g) = ClipboardGuard::open() else {
        return false;
    };
    let wide = crate::to_wide(text);
    unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        let Ok(hg) = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2) else {
            return false;
        };
        let ptr = GlobalLock(hg) as *mut u16;
        if ptr.is_null() {
            return false;
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        let _ = GlobalUnlock(hg);
        // Ownership passes to the system on success.
        SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(hg.0))).is_ok()
    }
}

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Sends Ctrl+<letter> (e.g. `b'C'`, `b'V'`).
pub fn send_ctrl(letter: u8) -> bool {
    let vk = VIRTUAL_KEY(letter as u16);
    let inputs = [
        key(VK_CONTROL, false),
        key(vk, false),
        key(vk, true),
        key(VK_CONTROL, true),
    ];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) as usize == inputs.len() }
}

/// Types text as Unicode key events (no clipboard involved).
pub fn type_text(text: &str) -> bool {
    let mut inputs = Vec::with_capacity(text.len() * 2);
    for unit in text.encode_utf16() {
        for up in [false, true] {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: unit,
                        dwFlags: if up {
                            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                        } else {
                            KEYEVENTF_UNICODE
                        },
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }
    }
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) as usize == inputs.len() }
}

/// Text selected in the foreground app (call before showing the Overlay).
pub fn capture_selection(max_chars: usize) -> Option<String> {
    if let Some(t) = uia::focused_selection(max_chars) {
        return Some(t);
    }
    let saved = clipboard_text();
    let before = unsafe { GetClipboardSequenceNumber() };
    if !send_ctrl(b'C') {
        return None;
    }
    // Wait up to ~250 ms for the app to update the clipboard.
    let mut copied = None;
    for _ in 0..25 {
        std::thread::sleep(Duration::from_millis(10));
        if unsafe { GetClipboardSequenceNumber() } != before {
            copied = clipboard_text();
            break;
        }
    }
    if copied.is_some()
        && let Some(s) = &saved
    {
        set_clipboard_text(s);
    }
    copied
        .filter(|t| !t.trim().is_empty())
        .map(|t| t.chars().take(max_chars).collect())
}

/// Pastes `text` into the focused control, then restores the clipboard.
pub fn paste_text(text: &str) -> bool {
    let saved = clipboard_text();
    if !set_clipboard_text(text) {
        return type_text(text);
    }
    let ok = send_ctrl(b'V');
    // Give the target app time to read the clipboard before restoring it.
    std::thread::sleep(Duration::from_millis(150));
    if let Some(s) = saved {
        set_clipboard_text(&s);
    }
    ok
}
