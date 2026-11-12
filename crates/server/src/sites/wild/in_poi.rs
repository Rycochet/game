//! Which poi's cutout a player in the wilderness stands in, told to the
//! client as it changes (`ServerMessage::InPoi`), with its name if it has one.

use shared::poi::Poi;
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::{Place, place_name_of};
use crate::world_map::learn_place_name;

/// The poi whose cutout a wilderness cell is in, if any (its chunk worked out
/// again if a road has let go of it since the window was made).
pub(super) fn poi_here(world: &mut ServerWorld, (x, y): (i64, i64)) -> Option<Poi> {
    let wild = &world.wilderness.wild;
    let (x, chunk) = (wild.wrap(x), wild.chunk_of(wild.wrap(x), y));
    let chunk = super::wild_chunk(world, chunk)?;
    let cutout = chunk.plan.cutout().filter(|cutout| cutout.contains(x, y))?;
    Some(*cutout.poi())
}

/// Tells the client of the poi a player stands in, and if it is a town they
/// learn its name for the world map. `false` if the client has gone.
pub(super) fn tell_of(client: &mut Client, world: &ServerWorld, poi: Option<Poi>) -> bool {
    if !client.link.send(&poi_message(world, poi)) {
        return false;
    }
    match poi {
        Some(poi) if place_name_of(world, &poi).is_some() => {
            learn_place_name(client, world, poi.chunk)
        }
        _ => true,
    }
}

/// What the client is told of the poi a player stands in: its kind and name.
fn poi_message(world: &ServerWorld, poi: Option<Poi>) -> ServerMessage {
    ServerMessage::InPoi {
        kind: poi.map(|poi| poi.kind),
        name: poi.and_then(|poi| place_name_of(world, &poi)),
    }
}

/// After a step in the wilderness: tells the client if the poi they stand in
/// has changed (walking into one, or back out).
pub(super) fn tell_poi(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let now = poi_here(world, visit.position());
    let Place::Wilderness { in_poi, .. } = &mut visit.place else {
        return true;
    };
    if *in_poi == now {
        return true;
    }
    *in_poi = now;
    tell_of(client, world, now)
}
