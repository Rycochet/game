//! Keeping the river points round the player known (`common::NearRivers`), as
//! the server does round every step, so steps across rivers cost the same here.

use bevy::prelude::*;

use super::Player;
use crate::common::{CurrentWorld, GameView, MapPosition, NearRivers};

/// On the world map, traces the rivers of any chunks round the player not
/// traced yet (a new world starts again).
pub(super) fn keep_river_points(
    view: Res<State<GameView>>,
    world: Option<Res<CurrentWorld>>,
    mut near: ResMut<NearRivers>,
    player: Single<&MapPosition, With<Player>>,
) {
    let Some(world) = world.filter(|_| *view.get() == GameView::WorldMap) else {
        return;
    };
    if near.world != Some(world.id) {
        *near = NearRivers {
            world: Some(world.id),
            ..default()
        };
    }
    near.points.cover(&world.rivers, (player.x, player.y));
}
