//! A real poi's site generator, with the sides a town's roads come in from.

use shared::poi::Poi;
use shared::protocol::SeenPoi;
use shared::road::town_sides;
use shared::site::Generator;

use crate::ServerWorld;

/// The generator of `poi`'s site (`Generator::for_site`), a town's main roads
/// facing the towns the road network joins it to.
pub(crate) fn site_generator(world: &ServerWorld, poi: &Poi) -> Generator {
    let sides = town_sides(&world.pois, poi);
    Generator::for_site(poi.kind, poi.seed, poi.desert, sides)
}

/// What anyone may see of `poi` on the world map, a town's wall's gates
/// facing its roads.
pub(crate) fn seen_poi(world: &ServerWorld, poi: Poi) -> SeenPoi {
    let sides = town_sides(&world.pois, &poi);
    SeenPoi::new(poi, sides)
}
