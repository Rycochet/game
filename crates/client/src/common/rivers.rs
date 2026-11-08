//! The world map points rivers run through near the player, for step costs and
//! routes (a step onto one wades, as the server says).

use bevy::prelude::*;
use shared::river::RiverPoints;
use shared::world::WorldId;

/// The river points traced round where the player has been on the world map
/// (`shared::river::RiverPoints`), kept by `player`: those of `world` only.
#[derive(Resource, Default)]
pub struct NearRivers {
    pub world: Option<WorldId>,
    pub points: RiverPoints,
}
