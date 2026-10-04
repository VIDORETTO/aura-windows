//! Native mouse probe for the isolated Aura QA executable only.
//! WebDriver pointer actions do not drive the Windows modal sizing loop.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{thread::sleep, time::Duration};
    use windows::Win32::{
        Foundation::{HWND, LPARAM, POINT, RECT},
        Graphics::Gdi::ClientToScreen,
        UI::{
            HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
            Input::KeyboardAndMouse::{
                INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT,
                SendInput,
            },
            WindowsAndMessaging::{
                EnumWindows, GetClientRect, GetCursorPos, GetForegroundWindow, IsWindowVisible,
                SetCursorPos, SetForegroundWindow,
            },
        },
    };
    use windows::core::BOOL;
    struct Target {
        process: String,
        windows: Vec<HWND>,
    }
    unsafe extern "system" fn collect(w: HWND, data: LPARAM) -> BOOL {
        let target = unsafe { &mut *(data.0 as *mut Target) };
        if unsafe { IsWindowVisible(w).as_bool() }
            && aura_win::windows_info::title(w) == "Aura"
            && aura_win::windows_info::process_name(aura_win::windows_info::pid_of(w))
                == target.process
        {
            target.windows.push(w);
        }
        BOOL(1)
    }
    let args: Vec<_> = std::env::args().collect();
    let process = args.get(1).ok_or("expected QA process filename")?;
    if !matches!(
        process.as_str(),
        "aura-original-2026-10-03.exe" | "aura-qa.exe"
    ) {
        return Err("probe only supports the isolated QA executable".into());
    }
    let edge = args.get(2).ok_or("expected edge")?;
    let dx: i32 = args.get(3).ok_or("expected delta x")?.parse()?;
    let dy: i32 = args.get(4).ok_or("expected delta y")?.parse()?;
    let mut target = Target {
        process: process.clone(),
        windows: vec![],
    };
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        EnumWindows(Some(collect), LPARAM(&mut target as *mut Target as isize))?;
    }
    if target.windows.len() != 1 {
        return Err("expected exactly one visible QA Overlay".into());
    }
    let w = target.windows[0];
    let mut rect = RECT::default();
    let mut origin = POINT::default();
    let mut cursor = POINT::default();
    unsafe {
        GetClientRect(w, &mut rect)?;
        ClientToScreen(w, &mut origin).ok()?;
        GetCursorPos(&mut cursor)?;
        let _ = SetForegroundWindow(w);
    }
    if unsafe { GetForegroundWindow() } != w {
        return Err("QA Overlay did not receive foreground focus; refusing mouse input".into());
    }
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    let corner = (edge.contains("North") || edge.contains("South"))
        && (edge.contains("East") || edge.contains("West"));
    let inset = if corner { 5 } else { 3 };
    let x = if edge.contains("West") {
        inset
    } else if edge.contains("East") {
        width - inset
    } else {
        width / 2
    };
    let y = if edge == "Move" {
        16
    } else if edge.contains("North") {
        // With undecorated shadows Tao reserves y<=SM_CYFRAME for HTTOP.
        // Start within the DOM corner below that native strip.
        if corner { 9 } else { 3 }
    } else if edge.contains("South") {
        height - 3
    } else {
        height / 2
    };
    let send = |flags| {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: flags,
                    ..Default::default()
                },
            },
        };
        unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) == 1 }
    };
    unsafe {
        SetCursorPos(origin.x + x, origin.y + y)?;
    }
    sleep(Duration::from_millis(150));
    if !send(MOUSEEVENTF_LEFTDOWN) {
        return Err("mouse down failed".into());
    }
    // Always release the button even if a cursor movement fails.
    sleep(Duration::from_millis(150));
    let movement = (1..=20).try_for_each(|step| {
        unsafe {
            SetCursorPos(origin.x + x + dx * step / 20, origin.y + y + dy * step / 20)?;
        }
        sleep(Duration::from_millis(20));
        let mut current = RECT::default();
        unsafe {
            GetClientRect(w, &mut current)?;
        }
        let at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        println!(
            "sample {at} {} {}",
            current.right - current.left,
            current.bottom - current.top
        );
        Ok::<_, windows::core::Error>(())
    });
    let released = send(MOUSEEVENTF_LEFTUP);
    unsafe {
        SetCursorPos(cursor.x, cursor.y)?;
    }
    movement?;
    if !released {
        return Err("mouse up failed".into());
    }
    sleep(Duration::from_millis(300));
    unsafe {
        GetClientRect(w, &mut rect)?;
    }
    println!(
        "edge={edge} before={width}x{height} after={}x{}",
        rect.right - rect.left,
        rect.bottom - rect.top
    );
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("qa_resize requires Windows");
    std::process::exit(1);
}
