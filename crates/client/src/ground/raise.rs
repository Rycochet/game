//! Bags stand on the ground where they are, raised where it slopes
//! (`common::DrawnGround`).

use bevy::prelude::*;

use super::{Bags, in_cells};
use crate::common::DrawnGround;
use crate::dungeon_map::CELL_SIZE;

/// Puts each bag on the ground under it whenever bags or heights change.
pub(super) fn raise_bags(
    heights: Res<DrawnGround>,
    bags: Res<Bags>,
    mut places: Query<&mut Transform>,
) {
    if !heights.is_changed() && !bags.is_changed() {
        return;
    }
    for (&at, bag) in &bags.0 {
        if let Ok(mut place) = places.get_mut(bag.root) {
            let (x, y) = in_cells(at);
            let ground = heights.drawn_between(x, y) * CELL_SIZE;
            if place.translation.y != ground {
                place.translation.y = ground;
            }
        }
    }
}
