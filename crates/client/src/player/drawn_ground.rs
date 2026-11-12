//! Keeping the current level's drawn ground (`common::DrawnGround`) up to
//! date: the wilderness's from its window's heights, a site's from the kind of
//! ground the server said it has (`shared::relief::LevelGround`).

use bevy::prelude::*;
use shared::movement::WildHeights;
use shared::relief::{LevelGround, Relief, UNITS_PER_16TH};

use crate::common::{CurrentDungeon, CurrentWorld, DrawnGround, WildCellHeights};

/// A wilderness window's bounds and its stairs' cells.
type WildKey = ((i64, i64, i64, i64), Vec<(i64, i64)>);

/// Works the ground out again when the level, its stairs or (in the
/// wilderness) its heights change, marking it changed only when it did.
pub(super) fn keep_drawn_ground(
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    wild: Res<WildCellHeights>,
    mut drawn: ResMut<DrawnGround>,
    // Where the land kept in `drawn.land` was sampled.
    mut land_for: Local<Option<(i64, i64, i64, i64)>>,
    // The window and stairs the wilderness's corners were worked out for.
    mut wild_for: Local<Option<WildKey>>,
) {
    if !dungeon.is_changed() && !wild.is_changed() && world.as_ref().is_none_or(|w| !w.is_changed())
    {
        return;
    }
    let stairs: Vec<_> = dungeon.stairs.iter().map(|s| (s.x, s.y)).collect();
    let ground = dungeon.info.as_ref().map(|info| &info.ground);
    let bounds = dungeon.level.bounds();
    let (relief, reference) = match (ground, &world, &wild.heights) {
        (Some(LevelGround::Wilderness), _, Some(heights)) => {
            // The window's corners only change with its heights or stairs:
            // cells coming in don't change them.
            let key = (bounds, stairs.clone());
            if !wild.is_changed() && *wild_for == Some(key.clone()) && drawn.relief.is_some() {
                return;
            }
            *wild_for = Some(key);
            (
                Some(Relief::of_land(heights, bounds, (0, 0), &stairs)),
                wild.reference * UNITS_PER_16TH,
            )
        }
        (Some(LevelGround::Land { offset, flats }), Some(world), _) => {
            let (ox, oy) = *offset;
            let (min_x, min_y, width, height) = bounds;
            let shifted = (min_x + ox, min_y + oy, width, height);
            let cells = drawn.bypass_change_detection();
            if *land_for != Some(shifted) || cells.land.is_none() {
                cells.land = Some(WildHeights::new(&world.ground, shifted));
                *land_for = Some(shifted);
            }
            let land = cells.land.as_mut().expect("just made");
            let level = &dungeon.level;
            land.relevel_by(flats, |x, y| !level.cell(x - ox, y - oy).is_outside());
            let relief = Relief::of_land(land, bounds, *offset, &stairs);
            let middle = relief.centre(min_x + width / 2, min_y + height / 2);
            (Some(relief), middle)
        }
        (Some(other @ LevelGround::Cave { .. }), Some(world), _) => {
            (other.relief(&world.ground, &dungeon.level, &stairs), 0)
        }
        _ => (None, 0),
    };
    if !matches!(ground, Some(LevelGround::Land { .. })) && drawn.land.is_some() {
        drawn.bypass_change_detection().land = None;
    }
    if drawn.relief != relief || drawn.reference != reference {
        drawn.relief = relief;
        drawn.reference = reference;
    }
}
