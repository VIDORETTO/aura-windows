//! UI Automation text extraction: the focused/selected text and the visible
//! text of a window (TextPattern first, then a bounded tree walk of names).

use crate::windows_info::hwnd;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern,
    TreeScope_Descendants, UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_TextControlTypeId,
    UIA_TextPatternId,
};
use windows::core::Interface;

const MAX_NODES: i32 = 2000;

fn automation() -> Option<IUIAutomation> {
    unsafe {
        // S_FALSE / RPC_E_CHANGED_MODE are fine: COM is already initialised.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()
    }
}

fn text_pattern(el: &IUIAutomationElement) -> Option<IUIAutomationTextPattern> {
    unsafe {
        el.GetCurrentPattern(UIA_TextPatternId)
            .ok()?
            .cast::<IUIAutomationTextPattern>()
            .ok()
    }
}

fn clip(s: String, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Visible text of a window, or `None` when UIA exposes nothing useful.
pub fn window_text(window: u64, max_chars: usize) -> Option<String> {
    let uia = automation()?;
    unsafe {
        let root = uia.ElementFromHandle(hwnd(window)).ok()?;
        if let Some(tp) = text_pattern(&root)
            && let Ok(range) = tp.DocumentRange()
            && let Ok(t) = range.GetText(max_chars as i32)
            && !t.is_empty()
        {
            return Some(t.to_string());
        }
        // Walk descendants: documents/edits via TextPattern, others by Name.
        let cond = uia.CreateTrueCondition().ok()?;
        let all = root.FindAll(TreeScope_Descendants, &cond).ok()?;
        let n = all.Length().ok()?.min(MAX_NODES);
        let mut out = String::new();
        for i in 0..n {
            if out.len() >= max_chars * 4 {
                break;
            }
            let Ok(el) = all.GetElement(i) else { continue };
            let Ok(kind) = el.CurrentControlType() else {
                continue;
            };
            if (kind == UIA_DocumentControlTypeId || kind == UIA_EditControlTypeId)
                && let Some(tp) = text_pattern(&el)
                && let Ok(range) = tp.DocumentRange()
                && let Ok(t) = range.GetText(max_chars as i32)
                && !t.is_empty()
            {
                out.push_str(&t.to_string());
                out.push('\n');
                continue;
            }
            if (kind == UIA_TextControlTypeId
                || kind == UIA_EditControlTypeId
                || kind == UIA_DocumentControlTypeId)
                && let Ok(name) = el.CurrentName()
                && !name.is_empty()
            {
                out.push_str(&name.to_string());
                out.push('\n');
            }
        }
        let out = out.trim().to_string();
        (!out.is_empty()).then(|| clip(out, max_chars))
    }
}

/// Selected text in the focused control, if it supports TextPattern.
pub fn focused_selection(max_chars: usize) -> Option<String> {
    let uia = automation()?;
    unsafe {
        let el = uia.GetFocusedElement().ok()?;
        let tp = text_pattern(&el)?;
        let ranges = tp.GetSelection().ok()?;
        let mut out = String::new();
        for i in 0..ranges.Length().ok()? {
            if let Ok(r) = ranges.GetElement(i)
                && let Ok(t) = r.GetText(max_chars as i32)
            {
                out.push_str(&t.to_string());
            }
        }
        (!out.trim().is_empty()).then(|| clip(out, max_chars))
    }
}
