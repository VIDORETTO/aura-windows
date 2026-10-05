//! Every Aura window is a desktop surface, not a web page: the WebView2
//! right-click menu keeps only editing actions (`aura_app::webview`) and the
//! browser shortcuts (F5, Ctrl+P, Ctrl+S "Save as", Alt+←, Ctrl+F…) are off.
//! Debug builds keep both for DevTools.

use tauri::{Runtime, Webview};

pub fn plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("aura-webview-guard")
        .on_webview_ready(|webview| harden(&webview))
        .build()
}

#[cfg(not(windows))]
fn harden<R: Runtime>(_webview: &Webview<R>) {}

#[cfg(windows)]
fn harden<R: Runtime>(webview: &Webview<R>) {
    if cfg!(debug_assertions) {
        return;
    }
    let result = webview.with_webview(|pw| {
        // SAFETY: COM calls on the web view's own controller, on its thread.
        if let Err(e) = unsafe { apply(&pw.controller()) } {
            tracing::warn!("webview guard: {e}");
        }
    });
    if let Err(e) = result {
        tracing::warn!("webview guard: {e}");
    }
}

#[cfg(windows)]
unsafe fn apply(
    controller: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller,
) -> windows::core::Result<()> {
    use aura_app::webview::{MenuEntry, context_menu_keep};
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use windows::core::{Interface, PWSTR};

    unsafe {
        let core = controller.CoreWebView2()?;
        if let Ok(settings) = core.Settings()?.cast::<ICoreWebView2Settings3>() {
            settings.SetAreBrowserAcceleratorKeysEnabled(false)?;
        }
        let handler =
            webview2_com::ContextMenuRequestedEventHandler::create(Box::new(|_sender, args| {
                let Some(args) = args else { return Ok(()) };
                let items = args.MenuItems()?;
                let mut count = 0u32;
                items.Count(&mut count)?;
                let mut names = Vec::with_capacity(count as usize);
                let mut kinds = Vec::with_capacity(count as usize);
                for i in 0..count {
                    let item = items.GetValueAtIndex(i)?;
                    let mut kind = COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND::default();
                    item.Kind(&mut kind)?;
                    let mut name = PWSTR::null();
                    item.Name(&mut name)?;
                    names.push(webview2_com::take_pwstr(name));
                    kinds.push(kind);
                }
                let entries: Vec<MenuEntry<'_>> = names
                    .iter()
                    .zip(&kinds)
                    .map(|(n, k)| match *k {
                        COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SEPARATOR => MenuEntry::Separator,
                        COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SUBMENU => MenuEntry::Submenu(n),
                        _ => MenuEntry::Command(n),
                    })
                    .collect();
                let keep = context_menu_keep(&entries);
                tracing::debug!("context menu {names:?} → keep {keep:?}");
                if keep.is_empty() {
                    // Nothing useful here (page background): no menu at all.
                    return args.SetHandled(true);
                }
                for i in (0..count).rev() {
                    if !keep.contains(&(i as usize)) {
                        items.RemoveValueAtIndex(i)?;
                    }
                }
                Ok(())
            }));
        let mut token = 0i64;
        core.cast::<ICoreWebView2_11>()?
            .add_ContextMenuRequested(&handler, &mut token)?;
    }
    Ok(())
}
