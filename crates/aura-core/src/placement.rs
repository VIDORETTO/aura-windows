//! Overlay placement across monitors (AC-012 of `specs/001-fundacao-overlay`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        (r > x && b > y).then(|| Rect::new(x, y, r - x, b - y))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    /// Work area in physical pixels (excludes the taskbar).
    pub work_area: Rect,
    pub dpi: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayMode {
    Compact,
    Expanded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPlacement {
    pub monitor_id: String,
    pub rect: Rect,
    pub dpi: u32,
}

/// Default logical sizes at 96 DPI.
pub const COMPACT_SIZE: (i32, i32) = (640, 64);
pub const EXPANDED_WIDTH: i32 = 640;
pub const MIN_EXPANDED: (i32, i32) = (480, 320);

fn scale(v: i32, dpi: u32) -> i32 {
    ((v as i64 * dpi as i64) as f64 / 96.0).round() as i32
}

/// Computes where the Overlay goes. Pure and total: always returns a rectangle
/// fully inside the target monitor work area.
pub fn place_overlay(
    monitors: &[Monitor],
    target_monitor: &str,
    saved: Option<&SavedPlacement>,
    mode: OverlayMode,
) -> Option<Rect> {
    let target = monitors
        .iter()
        .find(|m| m.id == target_monitor)
        .or_else(|| monitors.first())?;
    let area = target.work_area;

    if let Some(saved) = saved.filter(|s| s.monitor_id == target.id) {
        let mut rect = saved.rect;
        if saved.dpi != target.dpi && saved.dpi > 0 {
            let f = target.dpi as f64 / saved.dpi as f64;
            let ox = area.x as f64 + (rect.x - area.x) as f64 * f;
            let oy = area.y as f64 + (rect.y - area.y) as f64 * f;
            rect = Rect::new(
                ox.round() as i32,
                oy.round() as i32,
                (rect.w as f64 * f).round() as i32,
                (rect.h as f64 * f).round() as i32,
            );
        }
        return Some(clamp_into(rect, area));
    }

    let (w, h) = match mode {
        OverlayMode::Compact => (
            scale(COMPACT_SIZE.0, target.dpi),
            scale(COMPACT_SIZE.1, target.dpi),
        ),
        OverlayMode::Expanded => {
            let w = scale(EXPANDED_WIDTH, target.dpi);
            let h = ((area.h as f64) * 0.70).round() as i32;
            (w, h.max(scale(MIN_EXPANDED.1, target.dpi)))
        }
    };
    let compact_h = scale(COMPACT_SIZE.1, target.dpi);
    let x = area.x + (area.w - w) / 2;
    let y = area.y + (area.h as f64 / 3.0 - compact_h as f64 / 2.0).round() as i32;
    Some(clamp_into(Rect::new(x, y, w, h), area))
}

/// Moves and, if needed, shrinks `rect` so it is fully inside `area`.
pub fn clamp_into(rect: Rect, area: Rect) -> Rect {
    let w = rect.w.min(area.w).max(1);
    let h = rect.h.min(area.h).max(1);
    let x = rect.x.clamp(area.x, area.right() - w);
    let y = rect.y.clamp(area.y, area.bottom() - h);
    Rect::new(x, y, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitors() -> Vec<Monitor> {
        vec![
            Monitor {
                id: "A".into(),
                work_area: Rect::new(0, 0, 1920, 1040),
                dpi: 96,
            },
            Monitor {
                id: "B".into(),
                work_area: Rect::new(1920, 0, 2560, 1400),
                dpi: 144,
            },
        ]
    }

    #[test]
    fn saved_placement_on_present_monitor_is_restored() {
        let saved = SavedPlacement {
            monitor_id: "B".into(),
            rect: Rect::new(2100, 100, 700, 500),
            dpi: 144,
        };
        let r = place_overlay(&monitors(), "B", Some(&saved), OverlayMode::Expanded).unwrap();
        assert_eq!(r, Rect::new(2100, 100, 700, 500));
    }

    #[test]
    fn default_compact_is_centered_on_upper_third() {
        let r = place_overlay(&monitors(), "A", None, OverlayMode::Compact).unwrap();
        // x = (1920-640)/2 = 640 ; y = round(1040/3 - 32) = 315
        assert_eq!(r, Rect::new(640, 315, 640, 64));
    }

    #[test]
    fn saved_on_missing_monitor_falls_back_to_default() {
        let saved = SavedPlacement {
            monitor_id: "C".into(),
            rect: Rect::new(10, 10, 10, 10),
            dpi: 96,
        };
        let r = place_overlay(&monitors(), "A", Some(&saved), OverlayMode::Compact).unwrap();
        assert_eq!(r, Rect::new(640, 315, 640, 64));
    }

    #[test]
    fn partially_outside_rect_is_moved_inside() {
        let saved = SavedPlacement {
            monitor_id: "A".into(),
            rect: Rect::new(1800, 10, 400, 64),
            dpi: 96,
        };
        let r = place_overlay(&monitors(), "A", Some(&saved), OverlayMode::Compact).unwrap();
        assert_eq!(r, Rect::new(1520, 10, 400, 64));
    }

    #[test]
    fn dpi_change_scales_the_saved_rect() {
        let saved = SavedPlacement {
            monitor_id: "B".into(),
            rect: Rect::new(1920, 0, 640, 64),
            dpi: 96,
        };
        let r = place_overlay(&monitors(), "B", Some(&saved), OverlayMode::Compact).unwrap();
        assert_eq!(r, Rect::new(1920, 0, 960, 96));
    }

    #[test]
    fn no_monitors_yields_none() {
        assert!(place_overlay(&[], "A", None, OverlayMode::Compact).is_none());
    }
}
