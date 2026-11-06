//! Which poi's cutout a player in the wilderness stands in, told to the
//! client as it changes (`ServerMessage::InPoi`).

use shared::poi::PoiKind;
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::Place;

/// The kind of poi whose cutout a wilderness cell is in, if any (its chunk
/// worked out already, as the window round it has).
pub(super) fn poi_here(world: &ServerWorld, (x, y): (i64, i64)) -> Option<PoiKind> {
    let wild = &world.wilderness.wild;
    let x = wild.wrap(x);
    let chunk = world.wilderness.chunk(wild.chunk_of(x, y))?;
    let cutout = chunk.plan.cutout().filter(|cutout| cutout.contains(x, y))?;
    Some(cutout.poi().kind)
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
    client.link.send(&ServerMessage::InPoi(now))
}
