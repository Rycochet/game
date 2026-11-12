//! Messages about part of a carried stack (`FromStack`, `PartOfStack`): the part
//! is lifted out on its own, lying where its stack does, then dealt with as
//! anything is. Dropping waits its turn with the player's steps, so a drop's part
//! is lifted only then (`ground::StackPart`).

use bevy::prelude::*;
use shared::id::Id;
use shared::protocol::ClientMessage;

use super::{Action, Client};
use crate::ground::{GroundAction, StackPart};
use crate::ids::new_id;

/// The message to answer, with any part of a stack lifted out: the same message
/// if it isn't about one. `Err` once it is dealt with here, saying whether the
/// client is still welcome.
pub(super) fn lift(client: &mut Client, message: ClientMessage) -> Result<ClientMessage, bool> {
    let (part, then) = match message {
        ClientMessage::FromStack { stack, items, then } => (StackPart::Group(stack, items), then),
        ClientMessage::PartOfStack { stack, count, then } => (StackPart::Count(stack, count), then),
        other => return Ok(other),
    };
    match *then {
        ClientMessage::DropItem { item } => {
            let part = Some(part);
            let action = Action::Ground(GroundAction::Drop { item, part });
            client.waiting.push_back(action);
            Err(true)
        }
        then @ (ClientMessage::MoveItem { .. }
        | ClientMessage::StackItem { .. }
        | ClientMessage::GiveItem { .. }
        | ClientMessage::WearItem { .. }
        | ClientMessage::BinItem { .. }) => match part {
            StackPart::Group(stack, items) => {
                client.inventory.lift_group(stack, &items);
                Ok(then)
            }
            StackPart::Count(stack, count) => {
                match client.inventory.lift_count(stack, count, &mut new_id) {
                    Some(id) => Ok(naming(then, id)),
                    None => {
                        warn!("Refused part of {stack:?}: no {count} there");
                        Err(client.send_inventory())
                    }
                }
            }
        },
        other => {
            warn!("Refused {other:?} for part of a stack");
            Err(client.send_inventory())
        }
    }
}

/// The same message about the item `id` (a part just lifted out), in place of the
/// one it named (the client's stand-in).
fn naming(message: ClientMessage, id: Id) -> ClientMessage {
    match message {
        ClientMessage::MoveItem {
            to, x, y, rotation, ..
        } => ClientMessage::MoveItem {
            item: id,
            to,
            x,
            y,
            rotation,
        },
        ClientMessage::StackItem { onto, .. } => ClientMessage::StackItem { item: id, onto },
        ClientMessage::GiveItem { to, .. } => ClientMessage::GiveItem { item: id, to },
        ClientMessage::WearItem { .. } => ClientMessage::WearItem { item: id },
        ClientMessage::BinItem { .. } => ClientMessage::BinItem { item: id },
        ClientMessage::DropItem { .. } => ClientMessage::DropItem { item: id },
        other => other,
    }
}

#[cfg(test)]
mod tests;
