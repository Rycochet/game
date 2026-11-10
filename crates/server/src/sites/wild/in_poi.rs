//! Which poi's cutout a player in the wilderness stands in, told to the
//! client as it changes (`ServerMessage::InPoi`), with its name if it has one.

use shared::poi::Poi;
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::{Place, town_name_of};

/// The poi whose cutout a wilderness cell is in, if any (its chunk worked out
/// already, as the window round it has).
pub(super) fn poi_here(world: &ServerWorld, (x, y): (i64, i64)) -> Option<Poi> {
    let wild = &world.wilderness.wild;
    let x = wild.wrap(x);
    let chunk = world.wilderness.chunk(wild.chunk_of(x, y))?;
    let cutout = chunk.plan.cutout().filter(|cutout| cutout.contains(x, y))?;
    Some(*cutout.poi())
}

/// What the client is told of the poi a player stands in: its kind and name.
pub(super) fn poi_message(world: &ServerWorld, poi: Option<Poi>) -> ServerMessage {
    ServerMessage::InPoi {
        kind: poi.map(|poi| poi.kind),
        name: poi.and_then(|poi| town_name_of(world, &poi)),
    }
}

/// After a step in the wilderness: tells the client if the poi they stand in
/// has changed (walking into one, or back out).
pub(super) fn tell_poi(client: &mut Client, world: &ServerWorld) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let now = poi_here(world, visit.position);
    let Place::Wilderness { in_poi, .. } = &mut visit.place else {
        return true;
    };
    if *in_poi == now {
        return true;
    }
    *in_poi = now;
    client.link.send(&poi_message(world, now))
}
