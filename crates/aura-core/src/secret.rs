//! A wrapper that keeps secrets out of `Debug`/`Display` output (OT-003, 001).

use std::fmt;

/// Holds a sensitive value. Formatting always prints `[REDACTED]`; the value is
/// only reachable through [`Secret::expose`], which makes accidental logging
/// visible in code review. The buffer is overwritten on drop.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret<T: Wipe>(T);

impl<T: Wipe> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T: Wipe> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T: Wipe> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl<T: Wipe> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.wipe();
    }
}

impl From<String> for Secret<String> {
    fn from(value: String) -> Self {
        Secret::new(value)
    }
}

impl From<&str> for Secret<String> {
    fn from(value: &str) -> Self {
        Secret::new(value.to_string())
    }
}

/// Best-effort zeroing of the backing memory.
pub trait Wipe {
    fn wipe(&mut self);
}

impl Wipe for String {
    fn wipe(&mut self) {
        // SAFETY: writing zero bytes keeps the string valid UTF-8.
        unsafe {
            self.as_bytes_mut()
                .iter_mut()
                .for_each(|b| std::ptr::write_volatile(b, 0))
        };
        self.clear();
    }
}

impl Wipe for Vec<u8> {
    fn wipe(&mut self) {
        self.iter_mut()
            .for_each(|b| unsafe { std::ptr::write_volatile(b, 0) });
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_never_show_the_value() {
        let s = Secret::from("abc");
        assert_eq!(format!("{s:?}"), "[REDACTED]");
        assert_eq!(format!("{s}"), "[REDACTED]");
        assert_eq!(s.expose(), "abc");
    }
}
