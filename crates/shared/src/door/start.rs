//! How a door starts out, rolled from its poi's seed and where it stands.

use super::{Door, DoorStyle, Side, Swing};
use crate::dungeon::Level;
use crate::item::Quality;
use crate::material::Material;
use crate::poi::{Poi, PoiKind};

/// Out of 100: how many of a building's doors start closed, inside and on the
/// way out. Outside doors are shut more often.
fn closed_percent(kind: PoiKind, outside: bool) -> u64 {
    match (kind, outside) {
        // Herbalists keep their doors shut.
        (PoiKind::HerbalistHut, false) => 85,
        (PoiKind::HerbalistHut, true) => 95,
        (_, false) => 70,
        (_, true) => 90,
    }
}

/// Out of 100: how many doors of a ruin start broken. Of the rest, half as
/// many as usual are closed.
const RUINED_BROKEN_PERCENT: u64 = 30;

/// The door at cell `at` of level `index` of the site behind `poi` (`ruined`
/// if it is a ruin), as it is before anyone has touched it. `outside`: it leads
/// out (`door_is_outside`). Secret doors always start closed.
pub fn starting_door(
    poi: &Poi,
    ruined: bool,
    index: u32,
    at: (i64, i64),
    secret: bool,
    outside: bool,
    runs_east_west: bool,
) -> Door {
    let roll = poi.seed.derive("door").at(&[i64::from(index), at.0, at.1]);
    let percent = roll % 100;
    let side = if (roll / 100).is_multiple_of(2) {
        if runs_east_west {
            Side::North
        } else {
            Side::West
        }
    } else if runs_east_west {
        Side::South
    } else {
        Side::East
    };
    let mut closed = closed_percent(poi.kind, outside);
    let swing = if secret {
        Swing::Closed
    } else if ruined && percent < RUINED_BROKEN_PERCENT {
        Swing::Broken(side)
    } else {
        if ruined {
            closed /= 2;
        }
        // A second roll, so ruins' broken doors don't skew the rest.
        if (roll / 200) % 100 < closed {
            Swing::Closed
        } else {
            Swing::Open(side)
        }
    };
    Door {
        swing,
        quality: Quality::Common,
        material: Material::Wood,
        style: DoorStyle::Plain,
    }
}

/// Whether the door at `(x, y)` leads outside: outside ground is beside it.
pub fn door_is_outside(level: &Level, x: i64, y: i64) -> bool {
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .iter()
        .any(|(dx, dy)| level.cell(x + dx, y + dy).is_outside())
}
