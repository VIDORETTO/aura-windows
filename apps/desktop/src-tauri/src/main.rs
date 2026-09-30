// Aura desktop shell. Keep this crate thin: behaviour lives in `aura-app`
// (tested on any OS) and Windows adapters in `aura-win`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod overlay;
#[cfg(windows)]
mod shortcuts;
#[cfg(windows)]
mod win_platform;

use aura_app::events::HostEvent;
use aura_app::paths::AppPaths;
use aura_app::voice::AsrBackend;
use aura_app::{CodexRuntime, Host, HostConfig};
use aura_core::settings::Settings;
use commands::AppState;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt as _;

pub const EVENT: &str = "aura://event";

pub fn open_url(app: &AppHandle, url: &str) {
    if let Err(e) = app.opener().open_url(url, None::<&str>) {
        tracing::warn!("could not open browser: {e}");
    }
}

/// Settings window, created on demand (it is not needed at startup).
pub fn open_settings(app: &AppHandle, section: Option<&str>) {
    let route = format!("index.html#/settings/{}", section.unwrap_or("general"));
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.emit("aura://navigate", route.trim_start_matches("index.html#"));
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(route.into()))
        .title("Aura — Configurações")
        .inner_size(960.0, 680.0)
        .min_inner_size(760.0, 520.0)
        .center()
        .build();
    if let Err(e) = built {
        tracing::error!("could not open settings: {e}");
    }
}

/// Full-screen, borderless selector over the frozen monitor (004 TK-002).
pub fn open_region_selector(
    app: &AppHandle,
    f: &aura_app::host::FrozenScreen,
) -> Result<(), aura_app::HostError> {
    overlay::hide(app);
    close_region_selector(app);
    let q = |s: &str| {
        s.replace('%', "%25")
            .replace('&', "%26")
            .replace('#', "%23")
            .replace(' ', "%20")
    };
    let route = format!(
        "index.html#/region?token={}&path={}&w={}&h={}",
        f.token,
        q(&f.path.to_string_lossy()),
        f.width,
        f.height
    );
    let w = WebviewWindowBuilder::new(app, "region", WebviewUrl::App(route.into()))
        .title("Aura — região")
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .position(f.area.x as f64, f.area.y as f64)
        .inner_size(f.area.w as f64, f.area.h as f64)
        .focused(true)
        .build()
        .map_err(|e| aura_app::HostError::new("window", e.to_string()))?;
    // The frozen image is already in the window; keep the selector itself out of captures.
    #[cfg(windows)]
    if let Ok(hwnd) = w.hwnd() {
        aura_win::windows_info::exclude_from_capture(hwnd.0 as usize as u64, true);
    }
    let _ = w.set_focus();
    Ok(())
}

pub fn close_region_selector(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("region") {
        let _ = w.close();
    }
}

/// Applies settings that live outside the host (autostart, hotkeys).
pub fn apply_settings(app: &AppHandle, s: &Settings) {
    let autostart = app.autolaunch();
    let result = if s.start_with_windows {
        autostart.enable()
    } else {
        autostart.disable()
    };
    if let Err(e) = result {
        tracing::warn!("autostart: {e}");
    }
    #[cfg(windows)]
    shortcuts::apply(app, s);
    let _ = app.emit(
        EVENT,
        serde_json::json!({"channel": "settings", "event": s}),
    );
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir Aura", true, None::<&str>)?;
    let new = MenuItem::with_id(app, "new", "Nova conversa", true, None::<&str>)?;
    let pause = MenuItem::with_id(
        app,
        "pause",
        "Pausar/retomar privacidade",
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, "settings", "Configurações…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &new, &pause, &sep, &settings, &quit])?;
    let mut builder = TrayIconBuilder::with_id("aura")
        .tooltip("Aura")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => overlay::show(app),
            "new" => {
                overlay::show(app);
                let _ = app.emit_to(overlay::LABEL, "aura://new-conversation", ());
            }
            "pause" => {
                let host = app.state::<AppState>().host.clone();
                if let Err(e) = host.toggle_pause() {
                    tracing::warn!("pause: {e}");
                }
            }
            "settings" => open_settings(app, None),
            "quit" => {
                let host = app.state::<AppState>().host.clone();
                tauri::async_runtime::block_on(host.shutdown());
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                overlay::toggle(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

fn host_config() -> HostConfig {
    let paths = AppPaths::new(AppPaths::default_root());
    #[cfg(all(windows, not(feature = "demo")))]
    {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));
        let worker = exe_dir
            .map(|d| d.join("aura-worker.exe"))
            .filter(|p| p.exists());
        // Children (app-server, worker) die with Aura, even on a crash.
        let adopt: Option<aura_app::launcher::SpawnHook> = win_platform::child_job().map(|job| {
            let hook: aura_app::launcher::SpawnHook = Arc::new(move |pid| {
                if !job.adopt(pid) {
                    tracing::warn!(pid, "could not adopt child process into the job");
                }
            });
            hook
        });
        HostConfig {
            platform: win_platform::platform(),
            codex: CodexRuntime::Binary {
                program: None,
                on_spawn: adopt.clone(),
            },
            siwc: Default::default(),
            in_memory_store: false,
            recent: None,
            asr: match worker {
                Some(program) => AsrBackend::Worker {
                    program,
                    idle: std::time::Duration::from_secs(120),
                    on_spawn: adopt,
                },
                None => AsrBackend::Fake {
                    text: String::new(),
                },
            },
            hardware: win_platform::hardware(),
            disk: Arc::new(win_platform::WinDisk),
            capture_interval: std::time::Duration::from_secs(1),
            paths,
        }
    }
    #[cfg(any(not(windows), feature = "demo"))]
    {
        let _ = CodexRuntime::Fake;
        let (platform, _fg) = aura_app::platform::Platform::fake();
        let mut cfg = HostConfig::demo(paths, platform);
        cfg.asr = AsrBackend::Fake {
            text: "texto ditado de exemplo".into(),
        };
        cfg
    }
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    let cfg = host_config();
    let _ = aura_core::logging::init_logging(&cfg.paths.logs());
    let host: Arc<Host> = tauri::async_runtime::block_on(Host::start(cfg))?;
    app.manage(AppState { host: host.clone() });

    // Host events → every window.
    let mut rx = host.subscribe();
    let h2 = handle.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(ev) => {
                    if let HostEvent::Consent(_) = &ev
                        && !overlay::is_visible(&h2)
                    {
                        overlay::show(&h2);
                    }
                    let _ = h2.emit(EVENT, &ev);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("ui lagged {n} events")
                }
                Err(_) => break,
            }
        }
    });

    // First run: download + verify the pinned Codex app-server in background.
    #[cfg(all(windows, not(feature = "demo")))]
    {
        let h = host.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = h.ensure_codex().await {
                tracing::error!("app-server install failed: {e}");
            }
        });
    }

    overlay::decorate(&handle);
    build_tray(&handle)?;
    let settings = host.settings();
    #[cfg(windows)]
    shortcuts::start(&handle, &settings);
    apply_settings(&handle, &settings);

    // `--background` (autostart) keeps the Overlay hidden; otherwise greet.
    if !std::env::args().any(|a| a == "--background") {
        overlay::show(&handle);
    }
    Ok(())
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            overlay::show(app)
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::settings_get,
            commands::settings_update,
            commands::privacy_get,
            commands::privacy_set_source,
            commands::privacy_set_paused,
            commands::privacy_upsert_exclusion,
            commands::privacy_remove_exclusion,
            commands::privacy_access_log,
            commands::privacy_clear_access_log,
            commands::consent_answer,
            commands::auth_status,
            commands::auth_login,
            commands::auth_cancel,
            commands::auth_logout,
            commands::auth_switch,
            commands::auth_mark_welcomed,
            commands::providers_presets,
            commands::providers_list,
            commands::providers_save,
            commands::providers_remove,
            commands::providers_test,
            commands::conversation_start,
            commands::conversation_send,
            commands::conversation_steer,
            commands::conversation_interrupt,
            commands::conversation_compact,
            commands::conversation_set_mode,
            commands::conversation_history,
            commands::conversation_open,
            commands::conversation_rename,
            commands::conversation_pin,
            commands::conversation_archive,
            commands::conversation_delete,
            commands::conversation_close_ephemeral,
            commands::conversation_respond,
            commands::models_list,
            commands::tray_list,
            commands::tray_remove,
            commands::tray_move,
            commands::capture_screen,
            commands::capture_selection,
            commands::attach_file,
            commands::attachments_list,
            commands::insert_into_app,
            commands::quick_list,
            commands::quick_save,
            commands::quick_delete,
            commands::quick_toggle,
            commands::quick_expand,
            commands::mcp_list,
            commands::mcp_save,
            commands::mcp_delete,
            commands::mcp_detect,
            commands::mcp_import,
            commands::agent_restart,
            commands::mcp_status,
            commands::mcp_login,
            commands::workspace_files,
            commands::workspace_read,
            commands::open_path,
            commands::skills_list,
            commands::skills_review,
            commands::skills_install,
            commands::skills_create,
            commands::skills_delete,
            commands::recording_start,
            commands::recording_stop,
            commands::recordings_list,
            commands::recording_active,
            commands::recording_delete,
            commands::recording_export,
            commands::attachment_read,
            commands::voice_models,
            commands::voice_install,
            commands::voice_cancel_install,
            commands::voice_remove,
            commands::voice_select,
            commands::ptt_press,
            commands::ptt_release,
            commands::ptt_cancel,
            commands::audio_devices,
            commands::profiles_list,
            commands::profiles_save,
            commands::profiles_delete,
            commands::profile_active,
            commands::previous_app,
            commands::region_open,
            commands::region_commit,
            commands::region_cancel,
            commands::overlay_set_mode,
            commands::overlay_hide,
            commands::overlay_moved,
            commands::settings_open,
            commands::open_external,
            commands::diagnostics,
            commands::notify,
            commands::speak,
            commands::diagnostics_export,
            commands::erase_all_data,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Aura");

    app.run(|app, event| {
        // Tray app: closing windows never quits; only "Sair" does.
        if let RunEvent::ExitRequested { api, code, .. } = &event
            && code.is_none()
        {
            api.prevent_exit();
        }
        if let RunEvent::Exit = event
            && let Some(state) = app.try_state::<AppState>()
        {
            tauri::async_runtime::block_on(state.host.shutdown());
        }
    });
}
