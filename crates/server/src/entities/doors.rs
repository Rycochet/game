//! Mobs and doors: a mob with hands finds its way through closed doors and
//! opens them as it comes to them; one without hands is stopped by them. Only
//! on site levels for now: in the wilderness, a cutout's closed door is a wall
//! to every mob.

use shared::body::has_hands;
use shared::door::{Side, Swing};
use shared::dungeon::{Cell, Level};
use shared::poi::Poi;
use shared::spot::Spot;

use super::{Mob, Population};

/// What a site level's mobs need to open doors.
#[derive(Debug)]
pub(crate) struct MobDoors {
    /// The poi the site is behind, to know how its doors started.
    pub(crate) poi: Poi,
    /// The level with every door open, for those with hands to find their
    /// way by.
    pub(crate) open: Level,
}

/// A door a mob opened: where, and how it hangs now.
pub(crate) struct Opened {
    pub(crate) place: (crate::storage::SiteId, u32),
    pub(crate) poi: Poi,
    pub(crate) at: (i64, i64),
    pub(crate) swing: Swing,
}

impl Population {
    /// Its level, as its mobs know it.
    pub(crate) fn level(&self) -> &Level {
        &self.level
    }

    /// A door on its level has changed. Its mobs know which are locked: they
    /// live there.
    pub(crate) fn set_door(&mut self, (x, y): (i64, i64), door: shared::door::Door) {
        self.level.set_swing(x, y, door.swing);
        self.level.set_locked(x, y, door.is_locked());
        if let Some(doors) = &mut self.doors {
            doors.open = self.level.with_doors_open();
        }
    }
}

/// The level a mob finds its way by: one with hands goes as if every door were
/// open, and opens them as it comes to them (`open_on_the_way`).
pub(super) fn way_level<'a>(mob: &Mob, level: &'a Level, doors: Option<&'a MobDoors>) -> &'a Level {
    match doors {
        Some(doors) if has_hands(mob.kind) => &doors.open,
        _ => level,
    }
}

/// A mob that has just stepped into a closed door (`way_level`) opens it
/// instead, staying where it was (`was_at`): away from it, or towards it for
/// a secret door. Where the door is and how it hangs now.
pub(super) fn open_on_the_way(
    mob: &mut Mob,
    level: &mut Level,
    was_at: Spot,
) -> Option<((i64, i64), Swing)> {
    let (x, y) = mob.position;
    if !level.is_shut(x, y) {
        return None;
    }
    mob.place(was_at);
    let was_at = was_at.cell();
    let runs = level.opening_runs_east_west(x, y);
    let from = Side::of(runs, (x, y), was_at);
    let side = if level.cell(x, y) == Cell::SecretDoor {
        from
    } else {
        from.opposite()
    };
    let swing = Swing::Open(side);
    level.set_swing(x, y, swing);
    Some(((x, y), swing))
}
