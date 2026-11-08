//! The wilderness on the client: keeping the place on the world map
//! (`WorldReturn`, for the weather and the way out) with the player. Exploring
//! and leaving are `buttons`.
//! The server sends it as a level like any site's, a window round the player
//! that `WildernessWindow` moves (`CurrentDungeon::move_window`).

use bevy::prelude::*;
use shared::wilderness::SCALE;

use crate::common::{CurrentDungeon, CurrentWorld, MapPosition, WorldReturn};
use crate::player::Player;

/// In the wilderness, the place on the world map is the point the player is
/// in: the weather and the land's colours outside go by it, and leaving starts
/// from it.
pub(super) fn follow_point(
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    player: Single<&MapPosition, (With<Player>, Changed<MapPosition>)>,
    mut world_return: ResMut<WorldReturn>,
) {
    let Some(world) = world else {
        return;
    };
    if !dungeon.is_wilderness() {
        return;
    }
    let width = world.surface.width() * SCALE;
    let point = MapPosition {
        x: player.x.rem_euclid(width).div_euclid(SCALE),
        y: player.y.div_euclid(SCALE),
    };
    if world_return.0 != point {
        world_return.0 = point;
    }
}
