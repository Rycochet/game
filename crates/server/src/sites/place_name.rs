//! What places are called, for the client to show: towns', castles' and
//! forts' names, from the poi's map seed, which only the server knows, and how
//! high it stands.

use shared::place_name::place_name;
use shared::poi::Poi;

use crate::ServerWorld;

/// The name of the place behind a poi, or `None` if its kind has no name of
/// its own (only towns, castles and forts do). A ruin keeps its name.
pub(crate) fn place_name_of(world: &ServerWorld, poi: &Poi) -> Option<String> {
    place_name(poi.kind, poi.seed, world.ground.metres(poi.x, poi.y))
}
