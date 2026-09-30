//! Remembers Overlay position/size per monitor and mode (AC-012, 001).

use crate::{Result, Store};
use aura_core::placement::{OverlayMode, Rect, SavedPlacement};
use rusqlite::{OptionalExtension, params};

fn mode_key(mode: OverlayMode) -> &'static str {
    match mode {
        OverlayMode::Compact => "compact",
        OverlayMode::Expanded => "expanded",
    }
}

pub struct PlacementRepo<'a> {
    store: &'a Store,
}

impl<'a> PlacementRepo<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn get(&self, monitor_id: &str, mode: OverlayMode) -> Result<Option<SavedPlacement>> {
        self.store.with_conn(|c| {
            Ok(c.query_row(
                "SELECT x, y, w, h, dpi FROM overlay_placements WHERE monitor_id = ?1 AND mode = ?2",
                params![monitor_id, mode_key(mode)],
                |r| {
                    Ok(SavedPlacement {
                        monitor_id: monitor_id.to_string(),
                        rect: Rect::new(r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?),
                        dpi: r.get(4)?,
                    })
                },
            )
            .optional()?)
        })
    }

    pub fn save(&self, placement: &SavedPlacement, mode: OverlayMode) -> Result<()> {
        let r = placement.rect;
        self.store.with_conn(|c| {
            c.execute(
                "INSERT INTO overlay_placements(monitor_id, mode, x, y, w, h, dpi) VALUES (?1,?2,?3,?4,?5,?6,?7)
                 ON CONFLICT(monitor_id, mode) DO UPDATE SET x=excluded.x, y=excluded.y, w=excluded.w, h=excluded.h, dpi=excluded.dpi",
                params![placement.monitor_id, mode_key(mode), r.x, r.y, r.w, r.h, placement.dpi],
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_round_trip_per_mode() {
        let store = Store::open_in_memory().unwrap();
        let repo = PlacementRepo::new(&store);
        let p = SavedPlacement {
            monitor_id: "B".into(),
            rect: Rect::new(1, 2, 3, 4),
            dpi: 144,
        };
        repo.save(&p, OverlayMode::Expanded).unwrap();
        assert_eq!(repo.get("B", OverlayMode::Expanded).unwrap(), Some(p));
        assert_eq!(repo.get("B", OverlayMode::Compact).unwrap(), None);
    }
}
