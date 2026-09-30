//! Global shortcuts → actions (Windows): toggle Overlay, privacy pause,
//! global dictation, double-tap Ctrl and push-to-talk (only while the
//! Overlay is visible, so Ctrl+Space keeps working in other apps).

use crate::commands::AppState;
use crate::overlay;
use aura_core::settings::{Settings, Shortcut};
use aura_win::hotkeys::{Binding, HotkeyConfig, HotkeyEvent, HotkeyService};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

const TOGGLE: u32 = 1;
const PAUSE: u32 = 2;
const GLOBAL_VOICE: u32 = 3;

static SERVICE: Mutex<Option<HotkeyService>> = Mutex::new(None);
static GLOBAL_DICTATION: AtomicBool = AtomicBool::new(false);

fn config(s: &Settings) -> HotkeyConfig {
    let mut bindings = Vec::new();
    for (id, text) in [
        (TOGGLE, &s.invoke_shortcut),
        (PAUSE, &s.privacy_pause_shortcut),
        (GLOBAL_VOICE, &s.global_voice_shortcut),
    ] {
        if let Ok(shortcut) = Shortcut::parse(text) {
            bindings.push(Binding { id, shortcut });
        }
    }
    HotkeyConfig {
        bindings,
        push_to_talk: Shortcut::parse(&s.push_to_talk_shortcut).ok(),
        double_tap_ctrl: s.double_tap_ctrl,
    }
}

fn report(app: &AppHandle, errors: Vec<aura_win::hotkeys::RegistrationError>) {
    for e in errors {
        let which = match e.id {
            TOGGLE => "abrir o Aura",
            PAUSE => "pausar a privacidade",
            _ => "ditado global",
        };
        let _ = app.emit(crate::EVENT, serde_json::json!({
            "channel": "notice",
            "event": {"level": "warning", "message": format!("O atalho para {which} está {}. Escolha outro em Configurações.", e.reason)}
        }));
    }
}

pub fn start(app: &AppHandle, s: &Settings) {
    let (service, events, errors) = match HotkeyService::start(config(s)) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("hotkeys unavailable: {e}");
            return;
        }
    };
    report(app, errors);
    *SERVICE.lock().unwrap() = Some(service);
    let app = app.clone();
    std::thread::Builder::new()
        .name("aura-hotkey-dispatch".into())
        .spawn(move || {
            while let Ok(ev) = events.recv() {
                dispatch(&app, ev);
            }
        })
        .expect("hotkey dispatch thread");
}

pub fn apply(app: &AppHandle, s: &Settings) {
    if let Some(svc) = SERVICE.lock().unwrap().as_ref() {
        report(app, svc.update(config(s)));
    }
}

fn dispatch(app: &AppHandle, ev: HotkeyEvent) {
    let host = app.state::<AppState>().host.clone();
    match ev {
        HotkeyEvent::Triggered(TOGGLE) | HotkeyEvent::DoubleTapCtrl => {
            let a = app.clone();
            let _ = app.run_on_main_thread(move || overlay::toggle(&a));
        }
        HotkeyEvent::Triggered(PAUSE) => {
            if let Err(e) = host.toggle_pause() {
                tracing::warn!("pause: {e}");
            }
        }
        HotkeyEvent::Triggered(GLOBAL_VOICE) => {
            // First press starts dictation for the app in front; the second
            // transcribes and pastes it there.
            let starting = !GLOBAL_DICTATION.fetch_xor(true, Ordering::SeqCst);
            tauri::async_runtime::spawn(async move {
                if starting {
                    host.prepare_overlay();
                    if let Err(e) = host.ptt_press(None).await {
                        GLOBAL_DICTATION.store(false, Ordering::SeqCst);
                        tracing::warn!("dictation: {e}");
                    }
                } else if let aura_asr::ptt::PttState::Done { text } =
                    host.ptt_release(None, vec![]).await
                {
                    host.insert_into_app(&text);
                }
            });
        }
        HotkeyEvent::PushToTalkDown if overlay::is_visible(app) => {
            let _ = app.emit_to(overlay::LABEL, "aura://ptt", "down");
            tauri::async_runtime::spawn(async move {
                if let Err(e) = host.ptt_press(None).await {
                    tracing::warn!("push-to-talk: {e}");
                }
            });
        }
        HotkeyEvent::PushToTalkUp if overlay::is_visible(app) => {
            let _ = app.emit_to(overlay::LABEL, "aura://ptt", "up");
            let lang = match host.settings().language {
                aura_core::settings::Language::PtBr => Some("pt".to_string()),
                aura_core::settings::Language::En => None,
            };
            tauri::async_runtime::spawn(async move {
                // The result arrives in the UI as a `voice` event.
                host.ptt_release(lang, vec![]).await;
            });
        }
        _ => {}
    }
}
