//! Moved to another world: what was worked out for the last one goes.

use bevy::prelude::*;

use super::rivers::KnownRivers;
use super::terrain::{HeightCache, TerrainGrid};

/// Starts the terrain and rivers again for the new world (a new `HeightCache`
/// also redraws everything). Its pois go with `pois::despawn_all`.
pub(super) fn forget_world(mut commands: Commands) {
    commands.insert_resource(HeightCache::default());
    commands.insert_resource(TerrainGrid::default());
    commands.insert_resource(KnownRivers::default());
}
