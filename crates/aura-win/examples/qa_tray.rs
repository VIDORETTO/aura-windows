//! Reads the actual Windows popup menu of the isolated QA app.
//! Sends tray-icon 0.25.1's right-button-up callback, not a physical tray click.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{thread::sleep, time::Duration};
    use windows::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetMenuBarInfo, GetMenuItemCount, GetMenuStringW,
            MENUBARINFO, MF_BYPOSITION, OBJID_CLIENT, PostMessageW, WM_CANCELMODE, WM_RBUTTONUP,
        },
    };
    use windows::core::BOOL;
    struct Target {
        process: String,
        class: &'static str,
        windows: Vec<HWND>,
    }
    unsafe extern "system" fn collect(w: HWND, data: LPARAM) -> BOOL {
        let target = unsafe { &mut *(data.0 as *mut Target) };
        let mut class = [0u16; 128];
        let length = unsafe { GetClassNameW(w, &mut class) };
        if String::from_utf16_lossy(&class[..length.max(0) as usize]) == target.class
            && aura_win::windows_info::process_name(aura_win::windows_info::pid_of(w))
                == target.process
        {
            target.windows.push(w);
        }
        BOOL(1)
    }
    fn find(target: &mut Target) -> windows::core::Result<()> {
        target.windows.clear();
        unsafe { EnumWindows(Some(collect), LPARAM(target as *mut Target as isize)) }
    }
    let process = std::env::args()
        .nth(1)
        .ok_or("expected isolated QA process")?;
    if !matches!(
        process.as_str(),
        "aura-qa.exe" | "aura-original-2026-10-03.exe"
    ) {
        return Err("only isolated QA executables are supported".into());
    }
    let mut target = Target {
        process,
        class: "tray_icon_app",
        windows: vec![],
    };
    find(&mut target)?;
    if target.windows.len() != 1 {
        return Err("expected exactly one QA tray window".into());
    }
    let tray = target.windows[0];
    // Version-specific notification callback documented in the local dependency.
    unsafe {
        PostMessageW(Some(tray), 6002, WPARAM(0), LPARAM(WM_RBUTTONUP as isize))?;
    }
    let read = (|| -> Result<Vec<String>, Box<dyn std::error::Error>> {
        target.class = "#32768";
        for _ in 0..40 {
            sleep(Duration::from_millis(50));
            find(&mut target)?;
            if !target.windows.is_empty() {
                break;
            }
        }
        if target.windows.len() != 1 {
            return Err("expected exactly one QA popup menu".into());
        }
        let mut info = MENUBARINFO {
            cbSize: std::mem::size_of::<MENUBARINFO>() as u32,
            ..Default::default()
        };
        unsafe {
            GetMenuBarInfo(target.windows[0], OBJID_CLIENT, 0, &mut info)?;
        }
        let count = unsafe { GetMenuItemCount(Some(info.hMenu)) };
        if count <= 0 {
            return Err("QA menu is empty or unreadable".into());
        }
        let mut texts = vec![];
        for index in 0..count {
            let mut text = [0u16; 256];
            let length =
                unsafe { GetMenuStringW(info.hMenu, index as u32, Some(&mut text), MF_BYPOSITION) };
            if length > 0 {
                texts.push(String::from_utf16_lossy(&text[..length as usize]));
            }
        }
        Ok(texts)
    })();
    // Dismiss only the QA menu, including failed reads; never choose a command.
    unsafe {
        PostMessageW(Some(tray), WM_CANCELMODE, WPARAM(0), LPARAM(0))?;
    }
    for text in read? {
        println!("{text}");
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("QA tray probe requires Windows");
}
