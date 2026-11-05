//! Give Item (dev tools): new items as ordered, into the inventory, and in a site
//! what doesn't fit onto the ground; on the world map, all of it or nothing.

use bevy::prelude::*;
use shared::item::Item;
use shared::item_form::{ItemOrder, batch_seed, make_item};

use super::Client;
use crate::ServerWorld;
use crate::ids::new_id;

/// Makes and gives `count` of what is ordered. Whether the client is still welcome.
pub(super) fn give_items(
    client: &mut Client,
    world: &mut ServerWorld,
    order: &ItemOrder,
    seed: Option<u64>,
    count: u32,
    identical: bool,
) -> bool {
    let items = made(client, world, order, seed, count, identical);
    let mut inventory = client.inventory.clone();
    let mut left = Vec::new();
    for item in items {
        if let Err(item) = inventory.give(item, world.stacks) {
            left.push(item);
        }
    }
    // Only a site has ground to put the rest on.
    let has_ground = client.visit.is_some();
    if !left.is_empty() && !has_ground {
        info!("No room for {} new {:?}", left.len(), order.kind);
        return client.send_inventory();
    }
    client.inventory = inventory;
    for item in on_the_ground(left, identical) {
        crate::ground::put_down(client, world, item);
    }
    client.unsaved = true;
    client.update_speed(world.action_clock.ticks());
    client.send_inventory()
}

/// The items, as many as the server allows: up to a full stack of one alike, or up
/// to the most in any stack of different ones.
fn made(
    client: &Client,
    world: &ServerWorld,
    order: &ItemOrder,
    seed: Option<u64>,
    count: u32,
    identical: bool,
) -> Vec<Item> {
    let seed = seed.unwrap_or_else(|| new_id().seed().value());
    let made_at = Some(world.made_now());
    let wearer = client.inventory.creature;
    let one = |n: u32| {
        let seed = batch_seed(seed, if identical { 0 } else { n });
        make_item(order, wearer, new_id(), seed, made_at, &mut new_id)
    };
    let first = one(0);
    let most = match identical {
        true => world.stacks.most_of(&first),
        false => world.stacks.most,
    };
    let count = count.clamp(1, most.max(1));
    std::iter::once(first).chain((1..count).map(one)).collect()
}

/// What is left over, as it goes on the ground: items alike as one stack.
fn on_the_ground(mut left: Vec<Item>, identical: bool) -> Vec<Item> {
    if !identical || left.len() < 2 {
        return left;
    }
    let mut bottom = left.remove(0);
    bottom.above.extend(left);
    vec![bottom]
}

#[cfg(test)]
mod tests;
