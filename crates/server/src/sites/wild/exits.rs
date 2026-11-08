//! Ways out of the wilderness: back to the world map, or down (or up) a
//! cutout's stairs into its site, and from that site back again.

use bevy::prelude::*;
use shared::protocol::ServerMessage;
use shared::site::Exit;

use super::ensure_chunk;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::enter::{enter_site, make_site};
use crate::sites::{Arrival, Visit, leave};
use crate::storage::SiteId;
use crate::world_map::look_around;

/// The Leave key: out of a Dev Area from anywhere in it, out of a site from
/// outside ground (`leave_site`), or out of the wilderness for the world map
/// where they stand: only from outside ground (not inside a building or cave),
/// onto a place the world map lets them stand now.
pub(crate) fn leave_by_key(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    if let crate::storage::SiteId::DevArea(_) = visit.id {
        return super::super::dev_area::back_from_area(client, world);
    }
    if !visit.is_wilderness() {
        return crate::sites::leave::leave_site(client, world);
    }
    let (x, y) = visit.position;
    if !visit.level.level.cell(x, y).is_outside() {
        info!("Refused leaving the wilderness from under cover at ({x}, {y})");
        return true;
    }
    let (px, py) = world.wilderness.wild.point_of(x, y);
    if world.ground.elevation(px, py, world.conditions()).is_none() {
        info!("Refused leaving the wilderness into water at ({px}, {py})");
        return true;
    }
    leave(client, world);
    client.visit = None;
    client.character.world_position = (px, py);
    client.unsaved = true;
    client.changed_map();
    let message = ServerMessage::OnWorldMap { x: px, y: py };
    client.link.send(&message) && look_around(client, world)
}

/// The player takes a cutout's stairs at `(x, y)` in the wilderness, leading
/// to level `to` of its site: they go in, and coming out leads back here.
pub(in crate::sites) fn into_site(
    client: &mut Client,
    world: &mut ServerWorld,
    (x, y): (i64, i64),
    to: Exit,
) -> bool {
    let Exit::Level(index) = to else {
        return true;
    };
    let wild = &world.wilderness.wild;
    let (wx, chunk) = (wild.wrap(x), wild.chunk_of(x, y));
    ensure_chunk(world, chunk);
    let Some(cutout) = world.wilderness.chunk(chunk).and_then(|c| c.plan.cutout()) else {
        warn!("Stairs at ({x}, {y}) in the wilderness with no cutout");
        return true;
    };
    let (poi, offset) = (*cutout.poi(), cutout.offset());
    let id = SiteId::Poi(poi.chunk);
    let known = client.character.pois.entry(poi.chunk).or_default();
    (known.known, known.visited) = (true, true);
    let arrival = match offset {
        // A building's floors have their stairs on the same cell; other
        // sites' levels have stairs back to the first.
        Some((ox, oy)) => match make_site(world, &poi, id) {
            Some(site) if site.is_building() => Arrival::At((wx - ox, y - oy)),
            _ => Arrival::From(Exit::Level(0)),
        },
        None => Arrival::From(Exit::WorldMap),
    };
    leave(client, world);
    client.visit = None;
    client.unsaved = true;
    enter_site(client, world, poi, id, index, arrival, true)
}

/// Where in the wilderness taking `exit` from `(x, y)` in a site walked into
/// from the wilderness leads, if it leads out: stairs back to a site's first
/// level that is set into the wilderness, or the way up from one that isn't.
pub(in crate::sites) fn back_to_wilderness(
    world: &mut ServerWorld,
    visit: &Visit,
    exit: Exit,
    (x, y): (i64, i64),
) -> Option<(i64, i64)> {
    let poi = *visit.poi()?;
    let site = visit.site()?;
    ensure_chunk(world, poi.chunk);
    let cutout = world.wilderness.chunk(poi.chunk)?.plan.cutout()?;
    match (exit, cutout.offset()) {
        (Exit::Level(0), Some((ox, oy))) => {
            let at = if site.is_building() {
                (x, y)
            } else {
                site.level(0).arrival(Exit::Level(visit.index))?
            };
            Some((at.0 + ox, at.1 + oy))
        }
        (Exit::WorldMap, None) => cutout.stairs().first().map(|s| (s.x, s.y)),
        (Exit::WorldMap, Some((ox, oy))) => Some((x + ox, y + oy)),
        _ => None,
    }
}
