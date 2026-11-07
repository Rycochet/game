//! Developer tools: Dev Areas (`shared::site::DevArea`), made to order, and the
//! way back from them to wherever the player was before the first one.

use shared::protocol::ServerMessage;
use shared::site::{DEV_AREA_KIND, DevArea, Exit};

use super::enter::dev_poi;
use super::{Arrival, Visit, enter_site, leave, resume};
use crate::connections::Client;
use crate::storage::{DevReturn, SiteId};
use crate::world_map::look_around;
use crate::{ServerWorld, hall};

/// Makes a Dev Area as if new and puts the player on its stairs. Where they
/// were is kept from before the first, so making another from inside one
/// still goes back there.
pub(crate) fn generate_area(client: &mut Client, world: &mut ServerWorld, area: DevArea) -> bool {
    let area = area.checked();
    if client.character.dev_return.is_none() {
        client.character.dev_return = Some(DevReturn {
            world_position: client.world_position(),
            visit: client.visit.as_ref().map(Visit::save),
            hall: client.character.hall,
        });
    }
    leave(client, world);
    if let Some(visit) = client.visit.take() {
        forget_if_area(client, world, visit.id);
    }
    let id = SiteId::DevArea(area);
    forget_if_area(client, world, id);
    let poi = dev_poi(client, DEV_AREA_KIND, area.seed);
    let arrival = Arrival::From(Exit::WorldMap);
    enter_site(client, world, poi, id, 0, arrival, false)
}

/// Out of the Dev Area they are in (forgotten, as none is kept) and back to
/// where they were before it: a site level, the wilderness, the Hall or the
/// world map. `false` if the client has gone.
pub(crate) fn back_from_area(client: &mut Client, world: &mut ServerWorld) -> bool {
    leave(client, world);
    if let Some(visit) = client.visit.take() {
        forget_if_area(client, world, visit.id);
    }
    let back = client.character.dev_return.take().unwrap_or(DevReturn {
        world_position: client.world_position(),
        visit: None,
        hall: None,
    });
    client.character.world_position = back.world_position;
    client.unsaved = true;
    match (back.visit, back.hall) {
        (Some(visit), _) => resume(client, world, visit),
        (None, Some(at)) => hall::enter(client, world, at),
        (None, None) => {
            let (x, y) = back.world_position;
            client.changed_map();
            client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
        }
    }
}

/// Forgets a Dev Area, on the server and by the character; other sites stay.
fn forget_if_area(client: &mut Client, world: &mut ServerWorld, id: SiteId) {
    if let SiteId::DevArea(_) = id {
        world.forget_site(id);
        client.character.forget_site(&id);
    }
}
