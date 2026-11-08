//! Bags stand on their cell's ground, raised in the wilderness
//! (`common::WildCellHeights`).

use bevy::prelude::*;

use super::Bags;
use crate::common::WildCellHeights;
use crate::dungeon_map::CELL_SIZE;

/// Puts each bag on its cell's ground whenever bags or heights change.
pub(super) fn raise_bags(
    heights: Res<WildCellHeights>,
    bags: Res<Bags>,
    mut places: Query<&mut Transform>,
) {
    if !heights.is_changed() && !bags.is_changed() {
        return;
    }
    for (&(x, y), bag) in &bags.0 {
        if let Ok(mut place) = places.get_mut(bag.root) {
            let ground = heights.drawn(x, y) * CELL_SIZE;
            if place.translation.y != ground {
                place.translation.y = ground;
            }
        }
    }
}
