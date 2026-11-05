//! Moved to another world: the colours and rivers worked out for the last one go.

use bevy::prelude::*;

use super::lines::MapRivers;
use super::world::WorldTiles;

/// Starts the maps' tiles and rivers again for the new world.
pub(super) fn forget_world(mut commands: Commands) {
    commands.insert_resource(WorldTiles::default());
    commands.insert_resource(MapRivers::default());
}
