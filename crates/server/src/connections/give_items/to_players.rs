//! Give Item with another player as the target: made for them and put in their
//! inventory, once every client's messages this tick have been read, as only then
//! can two clients be reached at once. Only if they are on the same level and in
//! the giver's sight; otherwise nothing is given.

use bevy::prelude::*;

use super::super::Client;
use super::super::heal::pair;
use super::{into_inventory, made};
use crate::ServerWorlds;

/// Gives what each client asked for another player.
pub(in crate::connections) fn give_to_players(clients: &mut [Client], worlds: &mut ServerWorlds) {
    for i in 0..clients.len() {
        let gifts = std::mem::take(&mut clients[i].dev_gifts);
        for (to, batch) in gifts {
            let world = clients[i].world;
            let found = clients
                .iter()
                .position(|other| !other.gone && other.world == world && other.entity_id() == to);
            let Some(j) = found.filter(|&j| j != i) else {
                warn!("Nobody to give {:?} to: {to:?}", batch.order.kind);
                continue;
            };
            let (giver, other) = pair(clients, i, j);
            if !in_sight(giver, other) {
                warn!("Can't give {:?} to {to:?}: out of sight", batch.order.kind);
                continue;
            }
            let world = worlds.of(world);
            if let Some(items) = made(world, &batch, other.inventory.creature) {
                into_inventory(other, world, items, batch.identical);
                other.send_inventory();
            }
        }
    }
}

/// Whether `other` stands on the giver's level where the giver can see.
fn in_sight(giver: &mut Client, other: &Client) -> bool {
    let at = other
        .visit
        .as_ref()
        .zip(giver.visit.as_ref())
        .filter(|(theirs, mine)| theirs.population() == mine.population())
        .map(|(theirs, _)| theirs.position());
    let visit = giver.visit.as_mut();
    at.zip(visit)
        .is_some_and(|((x, y), visit)| visit.can_see(x, y))
}
