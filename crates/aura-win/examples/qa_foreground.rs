//! Empty, short-lived native foreground context for multi-monitor QA.
//! Does not target, inspect or send input to another application.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::time::{Duration, Instant};
    use windows::Win32::UI::HiDpi::{
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, GetForegroundWindow, MSG, PM_REMOVE,
        PeekMessageW, TranslateMessage, WINDOW_EX_STYLE, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };
    use windows::core::w;
    let args: Vec<_> = std::env::args().collect();
    let x: i32 = args.get(1).ok_or("expected physical x")?.parse()?;
    let y: i32 = args.get(2).ok_or("expected physical y")?.parse()?;
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let window = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("STATIC"),
            w!("Aura QA foreground"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            x,
            y,
            450,
            300,
            None,
            None,
            None,
            None,
        )?;
        let own = aura_core::events::PreviousApp {
            window: aura_win::windows_info::id_of(window),
            pid: std::process::id(),
            process_name: "qa_foreground.exe".into(),
            title: "Aura QA foreground".into(),
            monitor_id: String::new(),
        };
        let _ = aura_win::windows_info::restore_focus(&own);
        // Cross-thread foreground activation can finish after the API returns.
        let focus_deadline = Instant::now() + Duration::from_secs(20);
        while GetForegroundWindow() != window && Instant::now() < focus_deadline {
            let mut message = MSG::default();
            while PeekMessageW(&mut message, Some(window), 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        if GetForegroundWindow() != window {
            DestroyWindow(window)?;
            return Err("QA foreground fixture did not receive native focus".into());
        }
        println!("READY");
        std::io::stdout().flush()?;
        let until = Instant::now() + Duration::from_secs(30);
        while Instant::now() < until {
            let mut message = MSG::default();
            while PeekMessageW(&mut message, Some(window), 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        DestroyWindow(window)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Native foreground QA requires Windows");
}
