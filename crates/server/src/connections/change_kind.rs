//! A character becoming another kind of creature (for now only `DevSetCreature`).

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::protocol::ServerMessage;
use shared::sheet::Sheet;

use super::Client;
use crate::ServerWorld;

/// Changes the character's kind: what the new body still takes stays on, and what
/// it can't wear or has no room for drops on the ground under it. Only where there
/// is ground: never on the world map (owner, 2026-11-06: changing shape waits for
/// the wilderness, which is entered anywhere) nor in the Hall. Whether the client
/// is still welcome.
pub(super) fn change_kind(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: CreatureKind,
) -> bool {
    if client.visit.is_none() {
        warn!("Refused becoming {kind:?}: there is no ground here");
        // Put right whatever the client guessed.
        return client
            .link
            .send(&ServerMessage::Sheet(client.sheet.clone()))
            && client.send_inventory();
    }
    for item in client.inventory.change_body(kind, world.stacks) {
        crate::ground::put_down(client, world, item);
    }
    client.ground_told.again = true;
    client.sheet = Sheet::new(kind);
    client.unsaved = true;
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.clone()))
        && {
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
}
