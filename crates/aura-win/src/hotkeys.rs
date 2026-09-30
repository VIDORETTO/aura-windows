//! Global shortcuts (`RegisterHotKey`) plus a low-level keyboard hook for the
//! gestures `RegisterHotKey` cannot express: double-tap Ctrl (001 AC-006) and
//! push-to-talk hold/release (006).
//!
//! Everything runs on one dedicated thread with its own message loop (hooks
//! and hotkeys are bound to the registering thread). The hook does O(1) work
//! and forwards events over a channel, well within `LowLevelHooksTimeout`.

use crate::keys::{ChordEvent, ChordTracker, hotkey_modifiers, is_ctrl, vk_for};
use aura_core::gesture::{DoubleTapDetector, Gesture, Key, KeyEvent};
use aura_core::settings::Shortcut;
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, RegisterHotKey, UnregisterHotKey,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
    LLKHF_INJECTED, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_APP, WM_HOTKEY, WM_KEYDOWN, WM_KEYUP,
    WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// Caller-defined id echoed back in [`HotkeyEvent::Triggered`].
    pub id: u32,
    pub shortcut: Shortcut,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub bindings: Vec<Binding>,
    pub push_to_talk: Option<Shortcut>,
    pub double_tap_ctrl: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    Triggered(u32),
    DoubleTapCtrl,
    PushToTalkDown,
    PushToTalkUp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationError {
    pub id: u32,
    /// "em uso por outro app" or "tecla não suportada".
    pub reason: String,
}

enum Command {
    Apply(HotkeyConfig, Sender<Vec<RegistrationError>>),
}

const WM_AURA_COMMAND: u32 = WM_APP + 1;

/// Push-to-talk is only armed while the Overlay is visible, so the chord
/// (default Ctrl+Space) keeps working normally in other apps.
static PTT_ARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Arms/disarms push-to-talk (call on Overlay show/hide).
pub fn set_push_to_talk_armed(armed: bool) {
    PTT_ARMED.store(armed, std::sync::atomic::Ordering::SeqCst);
}

struct HookState {
    tx: Sender<HotkeyEvent>,
    detector: DoubleTapDetector,
    double_tap: bool,
    ptt: Option<(Shortcut, u16)>,
    chord: ChordTracker,
    swallowing: bool,
}

thread_local! {
    static STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

fn now_ms() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis() as u64
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let injected = kb.flags.0 & LLKHF_INJECTED.0 != 0;
        let msg = wparam.0 as u32;
        let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
        let vk = kb.vkCode as u16;
        // Ignore our own SendInput (paste, Ctrl+C) so it cannot trigger gestures.
        if !injected && (down || up) {
            let swallow = STATE.with(|s| {
                let mut s = s.borrow_mut();
                let Some(st) = s.as_mut() else { return false };
                if st.double_tap {
                    let key = if is_ctrl(vk) { Key::Ctrl } else { Key::Other };
                    if st.detector.on_key(KeyEvent {
                        key,
                        down,
                        at_ms: now_ms(),
                    }) == Some(Gesture::Toggle)
                    {
                        let _ = st.tx.send(HotkeyEvent::DoubleTapCtrl);
                    }
                }
                let armed = PTT_ARMED.load(std::sync::atomic::Ordering::SeqCst);
                if !armed && !st.swallowing {
                    // Keep modifier tracking fresh without triggering.
                    st.chord.observe(vk, down);
                    return false;
                }
                if let Some((shortcut, target)) = &st.ptt {
                    match st.chord.on_key(shortcut, *target, vk, down) {
                        Some(ChordEvent::Pressed) => {
                            st.swallowing = true;
                            let _ = st.tx.send(HotkeyEvent::PushToTalkDown);
                            return true;
                        }
                        Some(ChordEvent::Released) => {
                            st.swallowing = false;
                            let _ = st.tx.send(HotkeyEvent::PushToTalkUp);
                            return vk == *target;
                        }
                        None => {
                            // Swallow key auto-repeat while held.
                            return st.swallowing && vk == *target;
                        }
                    }
                }
                false
            });
            if swallow {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn apply(
    config: &HotkeyConfig,
    registered: &mut Vec<u32>,
    tx: &Sender<HotkeyEvent>,
) -> Vec<RegistrationError> {
    for id in registered.drain(..) {
        unsafe {
            let _ = UnregisterHotKey(None, id as i32);
        }
    }
    let mut errors = Vec::new();
    for b in &config.bindings {
        let Some(vk) = vk_for(&b.shortcut.key) else {
            errors.push(RegistrationError {
                id: b.id,
                reason: "tecla não suportada".into(),
            });
            continue;
        };
        let mods = HOT_KEY_MODIFIERS(hotkey_modifiers(&b.shortcut.modifiers));
        match unsafe { RegisterHotKey(None, b.id as i32, mods, vk as u32) } {
            Ok(()) => registered.push(b.id),
            Err(_) => errors.push(RegistrationError {
                id: b.id,
                reason: "em uso por outro app".into(),
            }),
        }
    }
    let ptt = config
        .push_to_talk
        .as_ref()
        .and_then(|s| Some((s.clone(), vk_for(&s.key)?)));
    STATE.with(|s| {
        *s.borrow_mut() = Some(HookState {
            tx: tx.clone(),
            detector: DoubleTapDetector::new(),
            double_tap: config.double_tap_ctrl,
            ptt,
            chord: ChordTracker::default(),
            swallowing: false,
        })
    });
    errors
}

pub struct HotkeyService {
    thread_id: u32,
    commands: Sender<Command>,
    join: Option<JoinHandle<()>>,
}

impl HotkeyService {
    /// Starts the hotkey thread; events arrive on the returned receiver.
    pub fn start(
        config: HotkeyConfig,
    ) -> std::io::Result<(Self, Receiver<HotkeyEvent>, Vec<RegistrationError>)> {
        let (events_tx, events_rx) = channel();
        let (cmd_tx, cmd_rx) = channel::<Command>();
        let (ready_tx, ready_rx) = channel();
        let join = std::thread::Builder::new()
            .name("aura-hotkeys".into())
            .spawn(move || unsafe {
                let mut msg = MSG::default();
                // Create the message queue before announcing the thread id.
                let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
                let module = GetModuleHandleW(None).ok();
                let hook: Option<HHOOK> =
                    SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module.map(Into::into), 0)
                        .ok();
                let mut registered = Vec::new();
                let errors = apply(&config, &mut registered, &events_tx);
                let _ = ready_tx.send((GetCurrentThreadId(), errors, hook.is_some()));
                loop {
                    let r = GetMessageW(&mut msg, None, 0, 0);
                    if r.0 == 0 || r.0 == -1 || msg.message == WM_QUIT {
                        break;
                    }
                    match msg.message {
                        WM_HOTKEY => {
                            let _ = events_tx.send(HotkeyEvent::Triggered(msg.wParam.0 as u32));
                        }
                        WM_AURA_COMMAND => {
                            while let Ok(Command::Apply(cfg, reply)) = cmd_rx.try_recv() {
                                let _ = reply.send(apply(&cfg, &mut registered, &events_tx));
                            }
                        }
                        _ => {
                            let _ = TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                }
                for id in registered {
                    let _ = UnregisterHotKey(None, id as i32);
                }
                if let Some(h) = hook {
                    let _ = UnhookWindowsHookEx(h);
                }
                STATE.with(|s| *s.borrow_mut() = None);
            })?;
        let (thread_id, errors, hooked) = ready_rx
            .recv()
            .map_err(|_| std::io::Error::other("hotkey thread died"))?;
        if !hooked {
            tracing::warn!(
                "low-level keyboard hook unavailable: double-tap and push-to-talk disabled"
            );
        }
        Ok((
            Self {
                thread_id,
                commands: cmd_tx,
                join: Some(join),
            },
            events_rx,
            errors,
        ))
    }

    /// Replaces all bindings (settings changed); returns conflicts.
    pub fn update(&self, config: HotkeyConfig) -> Vec<RegistrationError> {
        let (tx, rx) = channel();
        if self.commands.send(Command::Apply(config, tx)).is_err() {
            return vec![];
        }
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_AURA_COMMAND, WPARAM(0), LPARAM(0));
        }
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .unwrap_or_default()
    }

    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for HotkeyService {
    fn drop(&mut self) {
        self.shutdown();
    }
}
