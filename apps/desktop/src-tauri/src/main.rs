// Aura desktop shell. Keep this crate thin: behaviour lives in `aura-app`
// (tested on any OS) and Windows adapters in `aura-win`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod overlay;
#[cfg(windows)]
mod shortcuts;
mod webview_guard;
#[cfg(all(windows, not(feature = "demo")))]
mod win_platform;

use aura_app::events::HostEvent;
use aura_app::localization::text;
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

struct TrayMenu {
    open: MenuItem<tauri::Wry>,
    new: MenuItem<tauri::Wry>,
    pause: MenuItem<tauri::Wry>,
    settings: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
}

pub fn open_url(app: &AppHandle, url: &str) {
    if let Err(e) = app.opener().open_url(url, None::<&str>) {
        tracing::warn!("could not open browser: {e}");
    }
}

/// Settings window, created on demand (it is not needed at startup).
pub fn open_settings(app: &AppHandle, section: Option<&str>) {
    let language = app.state::<AppState>().host.settings().language;
    let route = format!("index.html#/settings/{}", section.unwrap_or("general"));
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.emit("aura://navigate", route.trim_start_matches("index.html#"));
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(route.into()))
        .title(text(language, "native.settingsTitle"))
        .inner_size(960.0, 680.0)
        .min_inner_size(760.0, 520.0)
        .center()
        .build();
    match built {
        Ok(_) => overlay::apply_capture_exclusion(
            app,
            app.state::<AppState>().host.settings().hide_from_capture,
        ),
        Err(e) => tracing::error!("could not open settings: {e}"),
    }
}

/// Full-screen, borderless selector over the frozen monitor (004 TK-002).
pub fn open_region_selector(
    app: &AppHandle,
    f: &aura_app::host::FrozenScreen,
    copy_text: bool,
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
        "index.html#/region?token={}&path={}&w={}&h={}&mode={}",
        f.token,
        q(&f.path.to_string_lossy()),
        f.width,
        f.height,
        if copy_text { "copy" } else { "chip" }
    );
    let w = WebviewWindowBuilder::new(app, "region", WebviewUrl::App(route.into()))
        .title(text(
            app.state::<AppState>().host.settings().language,
            "native.regionTitle",
        ))
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
        let hide = app.state::<AppState>().host.settings().hide_from_capture;
        aura_win::windows_info::exclude_from_capture(hwnd.0 as usize as u64, hide);
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
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_title(text(s.language, "native.settingsTitle"));
    }
    if let Some(window) = app.get_webview_window("region") {
        let _ = window.set_title(text(s.language, "native.regionTitle"));
    }
    if let Some(menu) = app.try_state::<TrayMenu>() {
        for (item, key) in [
            (&menu.open, "native.tray.open"),
            (&menu.new, "native.tray.new"),
            (&menu.pause, "native.tray.pause"),
            (&menu.settings, "native.tray.settings"),
            (&menu.quit, "native.tray.quit"),
        ] {
            let _ = item.set_text(text(s.language, key));
        }
    }
    let autostart = app.autolaunch();
    let result = if s.start_with_windows {
        autostart.enable()
    } else {
        autostart.disable()
    };
    if let Err(e) = result {
        tracing::warn!("autostart: {e}");
    }
    overlay::apply_capture_exclusion(app, s.hide_from_capture);
    #[cfg(windows)]
    shortcuts::apply(app, s);
    let _ = app.emit(
        EVENT,
        serde_json::json!({"channel": "settings", "event": s}),
    );
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let language = app.state::<AppState>().host.settings().language;
    let open = MenuItem::with_id(
        app,
        "open",
        text(language, "native.tray.open"),
        true,
        None::<&str>,
    )?;
    let new = MenuItem::with_id(
        app,
        "new",
        text(language, "native.tray.new"),
        true,
        None::<&str>,
    )?;
    let pause = MenuItem::with_id(
        app,
        "pause",
        text(language, "native.tray.pause"),
        true,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(
        app,
        "settings",
        text(language, "native.tray.settings"),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        app,
        "quit",
        text(language, "native.tray.quit"),
        true,
        None::<&str>,
    )?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &new, &pause, &sep, &settings, &quit])?;
    app.manage(TrayMenu {
        open,
        new,
        pause,
        settings,
        quit,
    });
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
        let hardware = win_platform::hardware(worker.as_deref());
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
                None => AsrBackend::Unavailable,
            },
            hardware,
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
    #[cfg(feature = "e2e")]
    let cfg = {
        let mut cfg = cfg;
        if let Ok(port) = std::env::var("AURA_E2E_AUTH_PORT") {
            let port: u16 = port.parse()?;
            if port == 0 {
                return Err("E2E authorization requires a listening loopback port".into());
            }
            cfg.siwc.authorize_url = format!("http://127.0.0.1:{port}/authorize");
        }
        cfg
    };
    let _ = aura_core::logging::init_logging(&cfg.paths.logs());
    // tauri.conf.json scopes %LOCALAPPDATA%\Aura; AURA_HOME moves the data root.
    let scope = app.asset_protocol_scope();
    for dir in [cfg.paths.captures_tmp(), cfg.paths.workspaces()] {
        if let Err(e) = scope.allow_directory(&dir, true) {
            tracing::warn!("asset scope {}: {e}", dir.display());
        }
    }
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
    overlay::track_previous_app(&handle);
    build_tray(&handle)?;
    let settings = host.settings();
    #[cfg(windows)]
    shortcuts::start(&handle, &settings);
    apply_settings(&handle, &settings);

    // 001 AC-001: starting Aura leaves only the tray icon. A manual start
    // (not `--background` autostart) says where Aura is and how to open it;
    // starting it again while it runs opens the Overlay (single instance).
    if !std::env::args().any(|a| a == "--background") {
        use tauri_plugin_notification::NotificationExt;
        let body = text(settings.language, "native.started.body")
            .replace("{shortcut}", &settings.invoke_shortcut);
        let _ = handle
            .notification()
            .builder()
            .title(text(settings.language, "native.started.title"))
            .body(body)
            .show();
    }
    Ok(())
}

#[cfg(all(test, windows, not(feature = "demo")))]
#[test]
fn native_configuration_never_uses_a_fake_speech_transcriber() {
    assert!(!matches!(host_config().asr, AsrBackend::Fake { .. }));
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
        .plugin(webview_guard::plugin())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::settings_get,
            commands::settings_update,
            commands::privacy_get,
            commands::privacy_set_source,
            commands::privacy_set_paused,
            commands::privacy_set_retention,
            commands::privacy_upsert_exclusion,
            commands::privacy_remove_exclusion,
            commands::privacy_access_log,
            commands::privacy_clear_access_log,
            commands::privacy_access_thumbnail,
            commands::privacy_open_conversation,
            commands::agent_task,
            commands::yolo_set,
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
            commands::providers_model_save,
            commands::providers_model_remove,
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
            commands::conversation_unarchive,
            commands::conversation_close_ephemeral,
            commands::conversation_respond,
            commands::models_list,
            commands::tray_list,
            commands::tray_remove,
            commands::tray_move,
            commands::capture_screen,
            commands::capture_selection,
            commands::context_attach_recent,
            commands::context_attach_skill,
            commands::attach_file,
            commands::attach_clipboard_image,
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
            commands::mcp_diagnose,
            commands::mcp_login,
            commands::workspace_files,
            commands::workspace_read,
            commands::workspace_pdf_preview,
            commands::open_path,
            commands::skills_list,
            commands::skills_review,
            commands::skills_install,
            commands::skills_create,
            commands::skills_delete,
            commands::skills_catalog,
            commands::skills_set_enabled,
            commands::skills_source,
            commands::memories_get,
            commands::memories_forget_fact,
            commands::memories_save,
            commands::memories_forget_all,
            commands::skills_update,
            commands::recording_start,
            commands::recording_stop,
            commands::recordings_list,
            commands::recording_active,
            commands::recording_delete,
            commands::recording_export,
            commands::recording_playback,
            commands::recording_attach,
            commands::attachment_read,
            commands::voice_models,
            commands::voice_install,
            commands::voice_cancel_install,
            commands::voice_remove,
            commands::voice_select,
            commands::ptt_press,
            commands::ptt_release,
            commands::ptt_cancel,
            commands::audio_test_start,
            commands::audio_test_stop,
            commands::audio_devices,
            commands::profiles_list,
            commands::profiles_save,
            commands::profiles_delete,
            commands::profile_active,
            commands::previous_app,
            commands::region_open,
            commands::region_commit,
            commands::region_copy_text,
            commands::meeting_active,
            commands::meeting_start,
            commands::meeting_pause,
            commands::meeting_stop,
            commands::meeting_from_buffer,
            commands::meetings_list,
            commands::meeting_utterances,
            commands::meeting_delete,
            commands::meeting_search,
            commands::capture_hiding_check,
            commands::replace_target,
            commands::replace_selection,
            commands::undo_replace,
            commands::region_cancel,
            commands::overlay_set_mode,
            commands::overlay_hide,
            commands::overlay_moved,
            commands::settings_open,
            commands::open_external,
            commands::diagnostics,
            commands::notify,
            commands::speak,
            commands::speech_options,
            commands::updater_configured,
            commands::onboarding_resume,
            commands::speech_consent,
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
