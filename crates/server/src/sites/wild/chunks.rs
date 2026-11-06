//! Working out a wilderness chunk: its poi's site as the server has it, the
//! cutout made from it and its stairs as players see them.

use shared::poi::{ChunkPos, Poi};
use shared::protocol::SeenStairs;
use shared::site::{
    BuildingLook, Exit, FieldLook, Generator, Rect, Site, SiteLevel, StairsDirection,
};
use shared::wilderness::{ChunkPlan, Cutout, gates_of};

use super::super::levels::seen_stairs;
use super::arrival::FROM_STAIRS;
use crate::ServerWorld;
use crate::storage::SiteId;
use crate::wilderness::WildChunk;

/// Makes sure the server has worked out a chunk of the wilderness.
pub(in crate::sites) fn ensure_chunk(world: &mut ServerWorld, chunk: ChunkPos) {
    if world.wilderness.chunk(chunk).is_some() {
        return;
    }
    let poi = world.pois.in_chunk(chunk);
    let site = poi.and_then(|poi| site_as_saved(world, &poi));
    let maps = &world.wilderness;
    let roads = maps.roads_near(chunk);
    let gates = gates_of(&maps.wild, &world.pois);
    let plan = ChunkPlan::new(&maps.wild, chunk, poi.zip(site.as_ref()), &gates, &roads);
    let (mut stairs, mut buildings, mut fields) = (Vec::new(), Vec::new(), Vec::new());
    let mut entrance = None;
    if let (Some(cutout), Some(site)) = (plan.cutout(), &site) {
        let first = site.level(0);
        stairs = cutout_stairs(cutout, site, &first);
        entrance = entrance_of(cutout, &first);
        if let Some(offset) = cutout.offset() {
            (buildings, fields) = looks(&first, offset);
        }
    }
    world.wilderness.keep_chunk(WildChunk {
        plan,
        stairs,
        buildings,
        fields,
        entrance,
    });
}

/// Where exploring from a cutout's poi arrives: its first level's entrance
/// from the world map, or `FROM_STAIRS` south of the way down into a site
/// entered from above.
fn entrance_of(cutout: &Cutout, first: &SiteLevel) -> Option<(i64, i64)> {
    match cutout.offset() {
        Some((ox, oy)) => first.arrival(Exit::WorldMap).map(|(x, y)| (x + ox, y + oy)),
        None => cutout.stairs().first().map(|s| (s.x, s.y + FROM_STAIRS)),
    }
}

/// A site's first level's town buildings and farm fields, moved by `offset`.
fn looks(first: &SiteLevel, (ox, oy): (i64, i64)) -> (Vec<BuildingLook>, Vec<FieldLook>) {
    let moved = |rect: Rect| Rect {
        x: rect.x + ox,
        y: rect.y + oy,
        ..rect
    };
    let buildings = first
        .buildings
        .iter()
        .map(|building| BuildingLook {
            rects: building.look.rects.iter().copied().map(moved).collect(),
            door: (building.look.door.0 + ox, building.look.door.1 + oy),
            ..building.look.clone()
        })
        .collect();
    let fields = first
        .fields
        .iter()
        .map(|field| FieldLook {
            rect: moved(field.rect),
            ..*field
        })
        .collect();
    (buildings, fields)
}

/// A poi's site made with the generator version saved with it (the current one
/// if nobody has been in), without starting a save for it.
fn site_as_saved(world: &mut ServerWorld, poi: &Poi) -> Option<Site> {
    let id = SiteId::Poi(poi.chunk);
    let generator = Generator::for_site(poi.kind, poi.seed, poi.desert);
    let version = match world.sites.get(&id) {
        Some(save) => save.version,
        None => match world.storage.load_site(id) {
            Ok(Some(save)) => {
                let version = save.version;
                world.sites.insert(id, save);
                version
            }
            _ => generator.version(),
        },
    };
    Site::with_generator(poi.kind, poi.seed, version, generator)
        .or_else(|| Site::with_generator(poi.kind, poi.seed, generator.version(), generator))
}

/// A cutout's stairs as players see them: the site's first level's, moved, or
/// the way down into a site entered from above, built as the stairs it arrives
/// at.
fn cutout_stairs(cutout: &Cutout, site: &Site, first: &SiteLevel) -> Vec<SeenStairs> {
    match cutout.offset() {
        Some((ox, oy)) => cutout
            .stairs()
            .iter()
            .map(|&stairs| {
                let own = shared::site::Stairs {
                    x: stairs.x - ox,
                    y: stairs.y - oy,
                    ..stairs
                };
                SeenStairs {
                    x: stairs.x,
                    y: stairs.y,
                    ..seen_stairs(site, 0, &first.level, own)
                }
            })
            .collect(),
        None => {
            let below = first.stairs.iter().find(|s| s.to == Exit::WorldMap);
            let Some(&below) = below else {
                return Vec::new();
            };
            let seen = seen_stairs(site, 0, &first.level, below);
            cutout
                .stairs()
                .iter()
                .map(|stairs| SeenStairs {
                    x: stairs.x,
                    y: stairs.y,
                    direction: StairsDirection::Down,
                    ..seen
                })
                .collect()
        }
    }
}
