//! Keeping the wilderness level's cell heights (`common::WildCellHeights`) up
//! to date, for step costs and routes.

use bevy::prelude::*;
use shared::movement::WildHeights;

use crate::common::{CurrentDungeon, CurrentWorld, WildCellHeights};

/// Works the heights out again whenever the level is the wilderness and its
/// window has moved (sampling a few hundred world map points), and forgets them
/// anywhere else.
pub(super) fn keep_wild_heights(
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    mut heights: ResMut<WildCellHeights>,
) {
    let Some(world) = world.filter(|_| dungeon.is_wilderness()) else {
        if heights.heights.is_some() {
            heights.heights = None;
        }
        return;
    };
    let bounds = dungeon.level.bounds();
    if heights.heights.is_none() || heights.bounds != bounds || heights.flats != dungeon.flats {
        heights.bounds = bounds;
        heights.flats.clone_from(&dungeon.flats);
        let found = WildHeights::new(&world.ground, bounds).with_flats(dungeon.flats.clone());
        let (min_x, min_y, width, height) = bounds;
        heights.reference = found.at(min_x + width / 2, min_y + height / 2);
        heights.heights = Some(found);
    }
}
