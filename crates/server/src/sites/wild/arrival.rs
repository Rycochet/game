//! Where going into the wilderness from the world map arrives: at a poi the
//! player is at (by Down, or a click on it), where its own visit would start
//! or where they last walked out of it, else the middle of their point.

use shared::poi::{ENTER_DISTANCE, Poi, can_enter_from};

use super::ensure_chunk;
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::SiteId;
use crate::world_map::known_poi;

/// Exploring from a poi entered from above arrives this many cells south of
/// its stairs down: near enough to see them clearly, not on them (owner).
pub(super) const FROM_STAIRS: i64 = 2;

/// Where in the wilderness exploring from the player's place on the world
/// map arrives: at the nearest poi they know of near enough to go into
/// (`poi_arrival`), else the middle of their point.
pub(super) fn arrival(client: &Client, world: &mut ServerWorld) -> (i64, i64) {
    let (px, py) = client.world_position();
    match poi_at(client, world, (px, py)) {
        Some(poi) => poi_arrival(client, world, &poi),
        None => world.wilderness.wild.centre_of(px, py),
    }
}

/// Where going into a poi arrives in the wilderness: where they last walked
/// out of its first level set in (if it is still walkable there and their
/// map of it is still the same), else its `WildChunk::entrance`, else the middle of its point.
pub(in crate::sites) fn poi_arrival(
    client: &Client,
    world: &mut ServerWorld,
    poi: &Poi,
) -> (i64, i64) {
    ensure_chunk(world, poi.chunk);
    let centre = world.wilderness.wild.centre_of(poi.x, poi.y);
    let Some(chunk) = world.wilderness.chunk(poi.chunk) else {
        return centre;
    };
    let entrance = chunk.entrance.unwrap_or(centre);
    let id = SiteId::Poi(poi.chunk);
    let Some(cutout) = chunk.plan.cutout() else {
        return entrance;
    };
    let (Some((ox, oy)), Some(&(0, (x, y)))) = (cutout.offset(), client.character.left_at.get(&id))
    else {
        return entrance;
    };
    let (x, y) = (x + ox, y + oy);
    // Somewhere in it still, on the map of it they knew.
    let inside = cutout.contains(x, y);
    let block = world.wilderness.wild.block_of(x, y);
    let open = inside && world.wilderness.block(block).is_walkable(x, y);
    let knew = client.character.site_versions.get(&id).copied();
    if !open || knew != Some(world.site(id).version) {
        return entrance;
    }
    (x, y)
}

/// The nearest poi the player knows of that they could go into from `(x, y)`.
fn poi_at(client: &Client, world: &ServerWorld, (x, y): (i64, i64)) -> Option<Poi> {
    let reach = ENTER_DISTANCE;
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
        .filter(|poi| can_enter_from(poi, (x, y), world.width))
        .min_by_key(|poi| {
            let (dx, dy) = apart(poi);
            (dx * dx + dy * dy, poi.chunk)
        })
}
