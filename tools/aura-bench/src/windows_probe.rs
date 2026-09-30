//! Windows-only probes (run on the reference machine / `windows-latest`):
//! idle private working set + CPU of `aura.exe` and its WebView2 children,
//! and hotkey → Overlay visible latency.

use crate::baseline::percentile;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    VIRTUAL_KEY, VK_CONTROL, VK_SHIFT, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, IsWindowVisible};
use windows::core::PCWSTR;

/// `aura.exe` pid plus every descendant (WebView2 runs as children).
fn process_tree(root: u32) -> Vec<u32> {
    let mut pairs = Vec::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return vec![root];
        };
        let mut e = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                pairs.push((e.th32ProcessID, e.th32ParentProcessID));
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    let mut tree = vec![root];
    let mut i = 0;
    while i < tree.len() {
        let parent = tree[i];
        tree.extend(pairs.iter().filter(|(_, p)| *p == parent).map(|(c, _)| *c));
        i += 1;
    }
    tree
}

fn open(pid: u32) -> Option<HANDLE> {
    unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
        .ok()
    }
}

fn private_bytes(h: HANDLE) -> u64 {
    let mut c = PROCESS_MEMORY_COUNTERS_EX {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        ..Default::default()
    };
    unsafe {
        if GetProcessMemoryInfo(h, &mut c as *mut _ as *mut PROCESS_MEMORY_COUNTERS, c.cb).is_ok() {
            return c.PrivateUsage as u64;
        }
    }
    0
}

fn cpu_100ns(h: HANDLE) -> u64 {
    let (mut a, mut b, mut k, mut u) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    unsafe {
        if GetProcessTimes(h, &mut a, &mut b, &mut k, &mut u).is_ok() {
            let f = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
            return f(k) + f(u);
        }
    }
    0
}

pub fn idle(pid: u32, seconds: u64) -> BTreeMap<String, f64> {
    let sample = || {
        let handles: Vec<HANDLE> = process_tree(pid).into_iter().filter_map(open).collect();
        let mem: u64 = handles.iter().map(|h| private_bytes(*h)).sum();
        let cpu: u64 = handles.iter().map(|h| cpu_100ns(*h)).sum();
        for h in handles {
            unsafe {
                let _ = CloseHandle(h);
            }
        }
        (mem, cpu)
    };
    let (_, cpu0) = sample();
    let t0 = Instant::now();
    let mut mems = Vec::new();
    while t0.elapsed() < Duration::from_secs(seconds) {
        std::thread::sleep(Duration::from_secs(1));
        mems.push(sample().0 as f64 / (1024.0 * 1024.0));
    }
    let (_, cpu1) = sample();
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as f64;
    let wall_100ns = t0.elapsed().as_secs_f64() * 1e7;
    BTreeMap::from([
        ("idle.private_ws_mb".to_string(), percentile(&mems, 50.0)),
        (
            "idle.cpu_avg_pct".to_string(),
            (cpu1.saturating_sub(cpu0)) as f64 / (wall_100ns * cores) * 100.0,
        ),
    ])
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

/// Sends the default invoke shortcut and waits for the "Aura" window.
pub fn open_latency(samples: usize) -> BTreeMap<String, f64> {
    let title: Vec<u16> = "Aura".encode_utf16().chain(Some(0)).collect();
    let chord = [
        key(VK_CONTROL, false),
        key(VK_SHIFT, false),
        key(VK_SPACE, false),
        key(VK_SPACE, true),
        key(VK_SHIFT, true),
        key(VK_CONTROL, true),
    ];
    let mut out = Vec::new();
    for _ in 0..samples {
        let hwnd = unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) }.ok();
        let visible = || {
            hwnd.map(|h| unsafe { IsWindowVisible(h).as_bool() })
                .unwrap_or(false)
        };
        if visible() {
            // Close it first (toggle), then measure an open.
            unsafe { SendInput(&chord, std::mem::size_of::<INPUT>() as i32) };
            std::thread::sleep(Duration::from_millis(300));
        }
        let t0 = Instant::now();
        unsafe { SendInput(&chord, std::mem::size_of::<INPUT>() as i32) };
        while !visible() && t0.elapsed() < Duration::from_secs(2) {
            std::thread::sleep(Duration::from_micros(500));
        }
        out.push(t0.elapsed().as_secs_f64() * 1000.0);
        std::thread::sleep(Duration::from_millis(400));
        unsafe { SendInput(&chord, std::mem::size_of::<INPUT>() as i32) };
        std::thread::sleep(Duration::from_millis(400));
    }
    BTreeMap::from([
        ("open.p50_ms".to_string(), percentile(&out, 50.0)),
        ("open.p95_ms".to_string(), percentile(&out, 95.0)),
    ])
}
