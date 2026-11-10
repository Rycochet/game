//! What places are called, for the client to show: towns' names, from the
//! poi's map seed, which only the server knows, and how high it stands.

use shared::poi::Poi;
use shared::town_name::town_name;

use crate::ServerWorld;

/// The name of the town behind a poi, or `None` if it isn't a town.
pub(super) fn town_name_of(world: &ServerWorld, poi: &Poi) -> Option<String> {
    town_name(poi.kind, poi.seed, world.ground.metres(poi.x, poi.y))
}
