//! Double-tap Ctrl gesture (AC-006 of `specs/001-fundacao-overlay`).
//!
//! The detector is pure: the Windows low-level keyboard hook only forwards
//! key events with timestamps and acts on the returned [`Gesture`].

/// Maximum time between the first release and the second release.
pub const WINDOW_MS: u64 = 400;
/// Taps within this period after an activation are ignored.
pub const COOLDOWN_MS: u64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Ctrl,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub down: bool,
    /// Milliseconds from any monotonic origin.
    pub at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Toggle,
}

#[derive(Debug, Default)]
pub struct DoubleTapDetector {
    ctrl_down: bool,
    /// Set when Ctrl went down and nothing else was pressed since.
    clean_press: bool,
    last_clean_tap_ms: Option<u64>,
    cooldown_until_ms: u64,
}

impl DoubleTapDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_key(&mut self, event: KeyEvent) -> Option<Gesture> {
        match (event.key, event.down) {
            (Key::Other, true) => {
                // Any other key invalidates the current and the pending tap.
                self.clean_press = false;
                self.last_clean_tap_ms = None;
                None
            }
            (Key::Other, false) => None,
            (Key::Ctrl, true) => {
                if !self.ctrl_down {
                    self.ctrl_down = true;
                    self.clean_press = true;
                }
                None
            }
            (Key::Ctrl, false) => {
                self.ctrl_down = false;
                if !self.clean_press {
                    return None;
                }
                self.clean_press = false;
                let now = event.at_ms;
                if now < self.cooldown_until_ms {
                    self.last_clean_tap_ms = None;
                    return None;
                }
                match self.last_clean_tap_ms {
                    Some(prev) if now.saturating_sub(prev) <= WINDOW_MS => {
                        self.last_clean_tap_ms = None;
                        self.cooldown_until_ms = now + COOLDOWN_MS;
                        Some(Gesture::Toggle)
                    }
                    _ => {
                        self.last_clean_tap_ms = Some(now);
                        None
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctrl(down: bool, at_ms: u64) -> KeyEvent {
        KeyEvent {
            key: Key::Ctrl,
            down,
            at_ms,
        }
    }
    fn other(down: bool, at_ms: u64) -> KeyEvent {
        KeyEvent {
            key: Key::Other,
            down,
            at_ms,
        }
    }
    fn run(events: &[KeyEvent]) -> Vec<Gesture> {
        let mut d = DoubleTapDetector::new();
        events.iter().filter_map(|e| d.on_key(*e)).collect()
    }

    #[test]
    fn two_clean_taps_within_window_toggle() {
        let g = run(&[
            ctrl(true, 0),
            ctrl(false, 60),
            ctrl(true, 200),
            ctrl(false, 260),
        ]);
        assert_eq!(g, vec![Gesture::Toggle]);
    }

    #[test]
    fn ctrl_c_is_not_a_tap() {
        let g = run(&[
            ctrl(true, 0),
            other(true, 50),
            other(false, 90),
            ctrl(false, 120),
            ctrl(true, 250),
            ctrl(false, 300),
        ]);
        assert!(g.is_empty());
    }

    #[test]
    fn taps_too_far_apart_do_nothing() {
        let g = run(&[
            ctrl(true, 0),
            ctrl(false, 50),
            ctrl(true, 500),
            ctrl(false, 560),
        ]);
        assert!(g.is_empty());
    }

    #[test]
    fn taps_during_cooldown_are_ignored() {
        let g = run(&[
            ctrl(true, 0),
            ctrl(false, 60),
            ctrl(true, 200),
            ctrl(false, 260),
            ctrl(true, 400),
            ctrl(false, 450),
            ctrl(true, 600),
            ctrl(false, 650),
        ]);
        assert_eq!(g, vec![Gesture::Toggle]);
    }

    #[test]
    fn auto_repeat_while_holding_ctrl_is_a_single_press() {
        let g = run(&[
            ctrl(true, 0),
            ctrl(true, 30),
            ctrl(true, 60),
            ctrl(false, 90),
            ctrl(true, 200),
            ctrl(false, 250),
        ]);
        assert_eq!(g, vec![Gesture::Toggle]);
    }
}
