//! World Gates in the wilderness stand on their cell's ground, raised by its
//! height (`common::WildCellHeights`; none in the Hall).

use bevy::prelude::*;

use super::place::Gates;
use crate::common::WildCellHeights;
use crate::dungeon_map::CELL_SIZE;

/// Puts each gate on its cell's ground whenever gates or heights change.
pub(super) fn raise_gates(
    heights: Res<WildCellHeights>,
    gates: Res<Gates>,
    mut places: Query<&mut Transform>,
) {
    if !heights.is_changed() && !gates.is_changed() {
        return;
    }
    for (&(x, y), &frame) in gates.standing() {
        if let Ok(mut place) = places.get_mut(frame) {
            let ground = heights.drawn(x, y) * CELL_SIZE;
            if place.translation.y != ground {
                place.translation.y = ground;
            }
        }
    }
}
