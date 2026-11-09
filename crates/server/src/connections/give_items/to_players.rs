//! Give Item with another player as the target: made for them and put in their
//! inventory (`for_others` finds them, in sight, once messages are read).

use super::super::Client;
use super::{Batch, into_inventory, made};
use crate::ServerWorld;

/// Makes the batch for `other` and puts it in their inventory.
pub(in crate::connections) fn give_to_player(
    other: &mut Client,
    world: &mut ServerWorld,
    batch: &Batch,
) {
    if let Some(items) = made(world, batch, other.inventory.creature) {
        into_inventory(other, world, items, batch.identical);
        other.send_inventory();
    }
}
