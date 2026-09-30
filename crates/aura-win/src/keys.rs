//! Canonical key names (`aura_core::settings::Shortcut`) → Windows virtual-key
//! codes and `RegisterHotKey` modifier flags.

use aura_core::settings::{Modifiers, Shortcut};

pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;
pub const MOD_NOREPEAT: u32 = 0x4000;

pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;
pub const VK_LSHIFT: u16 = 0xA0;
pub const VK_RSHIFT: u16 = 0xA1;
pub const VK_LCONTROL: u16 = 0xA2;
pub const VK_RCONTROL: u16 = 0xA3;
pub const VK_LMENU: u16 = 0xA4;
pub const VK_RMENU: u16 = 0xA5;
pub const VK_LWIN: u16 = 0x5B;
pub const VK_RWIN: u16 = 0x5C;

pub fn hotkey_modifiers(m: &Modifiers) -> u32 {
    let mut f = MOD_NOREPEAT;
    if m.ctrl {
        f |= MOD_CONTROL;
    }
    if m.alt {
        f |= MOD_ALT;
    }
    if m.shift {
        f |= MOD_SHIFT;
    }
    if m.win {
        f |= MOD_WIN;
    }
    f
}

pub fn vk_for(key: &str) -> Option<u16> {
    let k = key.trim();
    if k.len() == 1 {
        let c = k.chars().next()?.to_ascii_uppercase();
        if c.is_ascii_uppercase() || c.is_ascii_digit() {
            return Some(c as u16);
        }
    }
    if let Some(n) = k.strip_prefix('F').and_then(|n| n.parse::<u16>().ok())
        && (1..=24).contains(&n)
    {
        return Some(0x70 + n - 1);
    }
    Some(match k {
        "Space" => 0x20,
        "Enter" => 0x0D,
        "Tab" => 0x09,
        "Escape" => 0x1B,
        "Backspace" => 0x08,
        "Delete" => 0x2E,
        "Insert" => 0x2D,
        "Home" => 0x24,
        "End" => 0x23,
        "PageUp" => 0x21,
        "PageDown" => 0x22,
        "Up" => 0x26,
        "Down" => 0x28,
        "Left" => 0x25,
        "Right" => 0x27,
        // US-layout OEM keys; ABNT2 differs for some (see HANDOFF).
        "Backquote" => 0xC0,
        "Minus" => 0xBD,
        "Equal" => 0xBB,
        "Comma" => 0xBC,
        "Period" => 0xBE,
        "Slash" => 0xBF,
        "Semicolon" => 0xBA,
        "Quote" => 0xDE,
        "BracketLeft" => 0xDB,
        "BracketRight" => 0xDD,
        "Backslash" => 0xDC,
        _ => return None,
    })
}

pub fn is_ctrl(vk: u16) -> bool {
    matches!(vk, VK_CONTROL | VK_LCONTROL | VK_RCONTROL)
}

pub fn is_modifier(vk: u16) -> bool {
    matches!(
        vk,
        VK_SHIFT
            | VK_CONTROL
            | VK_MENU
            | VK_LSHIFT
            | VK_RSHIFT
            | VK_LCONTROL
            | VK_RCONTROL
            | VK_LMENU
            | VK_RMENU
            | VK_LWIN
            | VK_RWIN
    )
}

/// Tracks held modifiers from low-level hook events, to recognise a
/// push-to-talk chord press and its release.
#[derive(Debug, Default, Clone)]
pub struct ChordTracker {
    held: Modifiers,
    key_down: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordEvent {
    Pressed,
    Released,
}

impl ChordTracker {
    /// Tracks modifiers only (push-to-talk disarmed): never triggers.
    pub fn observe(&mut self, vk: u16, down: bool) {
        self.update_modifiers(vk, down);
        self.key_down = false;
    }

    fn update_modifiers(&mut self, vk: u16, down: bool) {
        match vk {
            VK_CONTROL | VK_LCONTROL | VK_RCONTROL => self.held.ctrl = down,
            VK_MENU | VK_LMENU | VK_RMENU => self.held.alt = down,
            VK_SHIFT | VK_LSHIFT | VK_RSHIFT => self.held.shift = down,
            VK_LWIN | VK_RWIN => self.held.win = down,
            _ => {}
        }
    }

    /// Feeds one key event; returns chord transitions for `shortcut`.
    pub fn on_key(
        &mut self,
        shortcut: &Shortcut,
        target_vk: u16,
        vk: u16,
        down: bool,
    ) -> Option<ChordEvent> {
        self.update_modifiers(vk, down);
        let mods_ok = self.held == shortcut.modifiers;
        if vk == target_vk && down && !self.key_down && mods_ok {
            self.key_down = true;
            return Some(ChordEvent::Pressed);
        }
        // Releasing the key or any required modifier ends the chord.
        let required_released = !down && (vk == target_vk || (is_modifier(vk) && !mods_ok));
        if self.key_down && required_released {
            self.key_down = false;
            return Some(ChordEvent::Released);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_keys_and_modifiers() {
        assert_eq!(vk_for("Space"), Some(0x20));
        assert_eq!(vk_for("a"), Some(b'A' as u16));
        assert_eq!(vk_for("7"), Some(b'7' as u16));
        assert_eq!(vk_for("F1"), Some(0x70));
        assert_eq!(vk_for("F24"), Some(0x87));
        assert_eq!(vk_for("F25"), None);
        assert_eq!(vk_for("Nope"), None);
        let s = Shortcut::parse("Ctrl+Shift+Space").unwrap();
        assert_eq!(
            hotkey_modifiers(&s.modifiers),
            MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT
        );
    }

    #[test]
    fn push_to_talk_chord() {
        let s = Shortcut::parse("Ctrl+Space").unwrap();
        let mut t = ChordTracker::default();
        assert_eq!(t.on_key(&s, 0x20, VK_LCONTROL, true), None);
        assert_eq!(t.on_key(&s, 0x20, 0x20, true), Some(ChordEvent::Pressed));
        // Auto-repeat does not re-trigger.
        assert_eq!(t.on_key(&s, 0x20, 0x20, true), None);
        // Releasing Ctrl first also ends the chord.
        assert_eq!(
            t.on_key(&s, 0x20, VK_LCONTROL, false),
            Some(ChordEvent::Released)
        );
        assert_eq!(t.on_key(&s, 0x20, 0x20, false), None);
        // Space without Ctrl does nothing.
        assert_eq!(t.on_key(&s, 0x20, 0x20, true), None);
        // While disarmed only modifiers are tracked.
        let mut t = ChordTracker::default();
        t.observe(VK_LCONTROL, true);
        t.observe(0x20, true);
        assert_eq!(t.on_key(&s, 0x20, 0x20, true), Some(ChordEvent::Pressed));
    }
}
