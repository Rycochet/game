//! Where exploring from the world map arrives: outside a poi the player is
//! at, where going into it would start, else the middle of their point.

use shared::poi::Poi;
use shared::protocol::ENTER_REACH;

use super::ensure_chunk;
use crate::ServerWorld;
use crate::connections::Client;
use crate::world_map::known_poi;

/// Exploring from a poi entered from above arrives this many cells south of
/// its stairs down: near enough to see them clearly, not on them (owner).
pub(super) const FROM_STAIRS: i64 = 2;

/// Where in the wilderness exploring from the player's place on the world
/// map arrives: the entrance of the nearest poi they know of within
/// `ENTER_REACH` (as going into it needs), else the middle of their point.
pub(super) fn arrival(client: &Client, world: &mut ServerWorld) -> (i64, i64) {
    let (px, py) = client.world_position();
    let centre = world.wilderness.wild.centre_of(px, py);
    let Some(poi) = poi_at(client, world, (px, py)) else {
        return centre;
    };
    ensure_chunk(world, poi.chunk);
    world
        .wilderness
        .chunk(poi.chunk)
        .and_then(|chunk| chunk.entrance)
        .unwrap_or(centre)
}

/// The nearest poi the player knows of that they could go into from `(x, y)`.
fn poi_at(client: &Client, world: &ServerWorld, (x, y): (i64, i64)) -> Option<Poi> {
    let reach = ENTER_REACH;
    // East-west, the nearer way round the planet.
    let apart = |poi: &Poi| {
        let dx = (poi.x - x).rem_euclid(world.width);
        (dx.min(world.width - dx), (poi.y - y).abs())
    };
    world
        .pois
        .chunks_in_area((x - reach, y - reach), (x + reach, y + reach))
        .into_iter()
        .filter_map(|chunk| known_poi(client, world, chunk))
        .filter(|poi| {
            let (dx, dy) = apart(poi);
            dx <= reach && dy <= reach
        })
        .min_by_key(|poi| {
            let (dx, dy) = apart(poi);
            (dx * dx + dy * dy, poi.chunk)
        })
}
