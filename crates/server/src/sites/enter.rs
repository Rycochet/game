//! Entering a site: from a poi, by a dev tool, or again after loading a save.

use bevy::prelude::*;
use shared::poi::{ChunkPos, Poi, PoiKind, can_enter_from};
use shared::rng::Seed;
use shared::site::{Exit, Generator, MAX_RUIN, Site};
use shared::spot::CELL_SPAN;

use super::{Arrival, NewVisit, enter_level, leave, leave_for_poi};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{SiteId, VisitSave};
use crate::world_map::{self};

/// Goes into the poi in `chunk`, if the player is near enough to it on the
/// world map (`poi::can_enter_from`): into the wilderness, where its first
/// level is set in, at its entrance or where they last walked out of it
/// (`wild::poi_arrival`).
pub(crate) fn enter_poi(client: &mut Client, world: &mut ServerWorld, chunk: ChunkPos) -> bool {
    let Some(poi) = world_map::known_poi(client, world, chunk) else {
        warn!("A client tried to enter {chunk:?}, with no poi it knows of");
        return true;
    };
    let near = can_enter_from(&poi, client.world_position(), world.width);
    if client.visit.is_some() || client.character.hall.is_some() || !near {
        warn!("A client tried to enter {chunk:?} from elsewhere");
        return true;
    }
    client.character.pois.entry(chunk).or_default().visited = true;
    client.unsaved = true;
    let at = super::wild::poi_arrival(client, world, &poi);
    super::wild::enter_wilderness(client, world, shared::spot::Spot::middle_of(at))
}

/// Developer tools: enters a made-up site at any level, where the player
/// stands on the world map.
pub(crate) fn dev_enter(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: PoiKind,
    seed: u64,
    level: u32,
    ruin: u8,
) -> bool {
    leave(client, world);
    client.visit = None;
    let poi = dev_poi(client, kind, seed);
    let id = SiteId::Dev { kind, seed, ruin };
    let arrival = Arrival::From(Exit::WorldMap);
    enter_site(client, world, poi, id, level, arrival, false)
}

/// Tests: enters a real poi's site straight onto level `level`.
#[cfg(test)]
pub(crate) fn test_enter_poi(
    client: &mut Client,
    world: &mut ServerWorld,
    poi: Poi,
    level: u32,
) -> bool {
    leave(client, world);
    client.visit = None;
    let id = SiteId::Poi(poi.chunk);
    let arrival = Arrival::From(Exit::WorldMap);
    enter_site(client, world, poi, id, level, arrival, false)
}

/// Developer tools: a made-up site as if new, whatever anyone has done there or
/// remembers of it, and the player in it. It is on no map: leaving it comes back
/// to where they stand now.
pub(crate) fn generate(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: PoiKind,
    seed: u64,
    ruin: u8,
) -> bool {
    let ruin = if Generator::can_be_ruined(kind) {
        ruin.min(MAX_RUIN)
    } else {
        0
    };
    leave(client, world);
    client.visit = None;
    let id = SiteId::Dev { kind, seed, ruin };
    world.forget_site(id);
    client.character.forget_site(&id);
    dev_enter(client, world, kind, seed, 0, ruin)
}

/// The made-up poi of a dev tools site: where the player stands.
pub(super) fn dev_poi(client: &Client, kind: PoiKind, seed: u64) -> Poi {
    let (x, y) = client.world_position();
    Poi {
        kind,
        chunk: ChunkPos { x: 0, y: 0 },
        x,
        y,
        seed: Seed::new(seed),
        desert: false,
    }
}

/// A client coming back: puts the player back on the level they were on
/// when they left, if it is still there.
pub(crate) fn resume(client: &mut Client, world: &mut ServerWorld, saved: VisitSave) -> bool {
    let id = saved.site;
    let poi = match id {
        SiteId::Poi(chunk) => world.pois.in_chunk(chunk),
        SiteId::Dev { kind, seed, .. } => Some(dev_poi(client, kind, seed)),
        SiteId::Wilderness => return super::wild::enter_wilderness(client, world, saved.position),
        // Never kept: back where they came from.
        SiteId::DevArea(_) => return super::dev_area::back_from_area(client, world),
    };
    let Some(poi) = poi else {
        warn!("A character was in {id:?}, which isn't there");
        client.unsaved = true;
        return world_map::look_around(client, world);
    };
    if let Err(error) = world.open_site(id) {
        return client.refuse_for_saves(&error);
    }
    // Their map is the version they knew (someone new to it knows the one saved
    // with the site).
    let saved_version = world.site(id).version;
    let knew = client.character.site_versions.get(&id).copied();
    let knew = knew.unwrap_or(saved_version);
    let site = make_site(world, &poi, id);
    if site.is_none() || world.site(id).version != knew {
        info!("A character was in {id:?}, which has changed: putting them out");
        return leave_for_poi(client, world, poi, id);
    }
    // Saved before entering a poi went into the wilderness: a first level
    // set in is the wilderness's now, and leaving any other goes there.
    if let SiteId::Poi(chunk) = id
        && saved.level == 0
    {
        super::wild::ensure_chunk(world, chunk);
        let cutout = world.wilderness.chunk(chunk).and_then(|c| c.plan.cutout());
        if let Some((ox, oy)) = cutout.and_then(|cutout| cutout.offset()) {
            let at = saved.position.offset(ox * CELL_SPAN, oy * CELL_SPAN);
            return super::wild::enter_wilderness(client, world, at);
        }
    }
    let from_wilderness = saved.from_wilderness || matches!(id, SiteId::Poi(_));
    let at = Arrival::At(saved.position);
    enter_site(client, world, poi, id, saved.level, at, from_wilderness)
}

/// Generates a site with the generator version it was first made with. If this
/// server can't run that one any more, the site is forgotten and made anew.
pub(super) fn make_site(world: &mut ServerWorld, poi: &Poi, id: SiteId) -> Option<Site> {
    let generator = match id {
        SiteId::Dev { ruin, .. } => Generator::for_poi(poi.kind).with_ruin(ruin),
        SiteId::DevArea(area) => Generator::DevArea(area),
        SiteId::Poi(_) | SiteId::Wilderness => Generator::for_site(poi.kind, poi.seed, poi.desert),
    };
    let version = world.site(id).version;
    let site = Site::with_generator(poi.kind, poi.seed, version, generator);
    if site.is_some() {
        return site;
    }
    warn!("{id:?} needs generator version {version}, which this server can't run: remaking it");
    world.forget_site(id);
    let version = world.site(id).version;
    Site::with_generator(poi.kind, poi.seed, version, generator)
}

/// Generates the site and puts the player on level `index` (or its last level,
/// if there are fewer), `from_wilderness` if they walked in from it. What they
/// remember of it goes if its map has changed.
pub(super) fn enter_site(
    client: &mut Client,
    world: &mut ServerWorld,
    poi: Poi,
    id: SiteId,
    index: u32,
    arrival: Arrival,
    from_wilderness: bool,
) -> bool {
    if let Err(error) = world.open_site(id) {
        return client.refuse_for_saves(&error);
    }
    let Some(site) = make_site(world, &poi, id) else {
        error!("{id:?} can't be made by this server");
        return true;
    };
    let version = world.site(id).version;
    let known = client.character.site_versions.insert(id, version);
    if known.is_some_and(|known| known != version) {
        client.character.memory.remove(&id);
        client.character.bags.remove(&id);
        client.character.doors.remove(&id);
        client.character.left_at.remove(&id);
    }
    client.unsaved = true;
    let index = index.min(site.levels() - 1);
    let new = NewVisit {
        poi,
        id,
        site,
        from_wilderness,
    };
    enter_level(client, world, new, index, arrival, true)
}
