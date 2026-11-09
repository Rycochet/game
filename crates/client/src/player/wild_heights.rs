//! Keeping the wilderness level's cell heights (`common::WildCellHeights`) up
//! to date, for step costs and routes.

use bevy::prelude::*;
use shared::movement::WildHeights;

use crate::common::{CurrentDungeon, CurrentWorld, WildCellHeights};

/// Works the land's heights out again whenever the level is the wilderness
/// and its window has moved (sampling a few hundred world map points), and
/// forgets them anywhere else. Pois' covered ground is levelled again as the
/// flats or the cells change (a cell's kind decides whether it is levelled),
/// marking the heights changed only when a height did.
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
    let moved = heights.heights.is_none() || heights.bounds != bounds;
    if moved {
        let found = WildHeights::new(&world.ground, bounds);
        let (min_x, min_y, width, height) = bounds;
        heights.bounds = bounds;
        heights.reference = found.at(min_x + width / 2, min_y + height / 2);
        heights.heights = Some(found);
    }
    let flats_changed = heights.flats != dungeon.flats;
    if !moved && !flats_changed && !(dungeon.is_changed() && !dungeon.flats.is_empty()) {
        return;
    }
    let cells = heights.bypass_change_detection();
    cells.flats.clone_from(&dungeon.flats);
    let changed = match &mut cells.heights {
        Some(found) => found.relevel(&dungeon.flats, &dungeon.level),
        None => false,
    };
    if moved || changed {
        heights.set_changed();
    }
}
