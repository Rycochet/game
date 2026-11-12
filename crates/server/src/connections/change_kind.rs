//! A character becoming another kind of creature, for good or shapeshifted (for now
//! only `DevSetCreature`).

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::protocol::{KindChange, ServerMessage};
use shared::sheet::Sheet;

use super::Client;
use crate::ServerWorld;

/// Changes the character's kind: what the new body still takes stays on, and what
/// it can't wear or has no room for drops on the ground under it. Only where there
/// is ground: never on the world map (owner, 2026-11-06: changing shape waits for
/// the wilderness, which is entered anywhere) nor in the Hall. For good, it is a
/// new creature of that kind; shapeshifted, only its body and Body row change
/// (`Sheet::shapeshift`). Whether the client is still welcome.
pub(super) fn change_kind(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: CreatureKind,
    how: KindChange,
) -> bool {
    if client.visit.is_none() {
        warn!("Refused becoming {kind:?}: there is no ground here");
        // Put right whatever the client guessed.
        return client
            .link
            .send(&ServerMessage::Sheet(client.sheet.as_sent()))
            && client.send_inventory();
    }
    let was = client.inventory.creature;
    for item in client.inventory.change_body(kind, world.stacks) {
        crate::ground::put_down(client, world, item);
    }
    client.ground_told.again = true;
    // Health and Mana stay as they were; how much there can be follows the
    // new attributes (`keep_vitals` tells the client).
    match how {
        // Its body only: the form's Body row, the rest its own.
        KindChange::Temporary => client.sheet.shapeshift(was, kind),
        KindChange::Permanent => {
            let vitals = std::mem::take(&mut client.sheet.vitals);
            let alignment = client.sheet.alignment;
            client.sheet = Sheet::new(kind);
            client.sheet.vitals = vitals;
            client.sheet.alignment = alignment;
        }
    }
    client.unsaved = true;
    client
        .link
        .send(&ServerMessage::Sheet(client.sheet.as_sent()))
        && {
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
}
