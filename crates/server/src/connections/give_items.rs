//! Give Item (dev tools): new items as ordered, made for whoever gets them: into the
//! player's inventory, or with a target, a mob's pockets or another player's
//! inventory (`give_items/to_players.rs`). Into an inventory, in a site what doesn't
//! fit goes onto the ground under them; on the world map, all of it or nothing.
//! Nothing at all if any can't be made as asked.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::Item;
use shared::item_form::{ItemOrder, Refused, batch_seed, try_make_item};

use super::Client;
use crate::ServerWorld;
use crate::entities::{give_to_mob, mob_in_sight};
use crate::ids::new_id;

mod to_players;

pub(super) use to_players::give_to_player;

/// What Give Item asked for, apart from who gets it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Batch {
    pub order: ItemOrder,
    pub seed: Option<u64>,
    pub count: u32,
    pub identical: bool,
}

/// Makes and gives a batch, to the player or to `to`: a mob in sight at once,
/// another player once every message this tick has been read. Whether the client
/// is still welcome.
pub(super) fn give_items(
    client: &mut Client,
    world: &mut ServerWorld,
    batch: Batch,
    to: Option<Id>,
) -> bool {
    let Some(to) = to else {
        let kind = client.inventory.creature;
        if let Some(items) = made(world, &batch, kind) {
            into_inventory(client, world, items, batch.identical);
        }
        return client.send_inventory();
    };
    if let Some(kind) = mob_in_sight(client, world, to) {
        if let Some(items) = made(world, &batch, kind)
            && let Err(why) = give_to_mob(client, world, to, items)
        {
            warn!("Couldn't give {:?} to {to:?}: {why}", batch.order.kind);
        }
        return true;
    }
    super::for_others::ask(client, to, super::for_others::ForOther::Gift(batch));
    true
}

/// Puts new items in `owner`'s inventory, and in a site what doesn't fit on the
/// ground under them; on the world map nothing unless it all fits.
fn into_inventory(owner: &mut Client, world: &mut ServerWorld, items: Vec<Item>, identical: bool) {
    let mut inventory = owner.inventory.clone();
    let mut left = Vec::new();
    for item in items {
        if let Err(item) = inventory.give(item, world.stacks, &mut new_id) {
            left.push(item);
        }
    }
    // Only a site has ground to put the rest on.
    if !left.is_empty() && owner.visit.is_none() {
        info!("No room for {} new items", left.len());
        return;
    }
    owner.inventory = inventory;
    for item in on_the_ground(left, identical) {
        crate::ground::put_down(owner, world, item);
    }
    owner.unsaved = true;
    owner.update_speed(world.action_clock.ticks());
}

/// The items for `wearer`, as many as the server allows (`count_made`); none, and
/// said why, if any can't be made as asked.
fn made(world: &ServerWorld, batch: &Batch, wearer: CreatureKind) -> Option<Vec<Item>> {
    let made = count_made(world, batch, wearer);
    if made.is_err() {
        info!(
            "Can't make {:?} as asked: {:?}",
            batch.order.kind, batch.order.request
        );
    }
    made.ok()
}

/// Up to a full stack of one alike, or up to the most in any stack of different
/// ones, made to fit `wearer`. Refused if any can't be made as asked.
fn count_made(
    world: &ServerWorld,
    batch: &Batch,
    wearer: CreatureKind,
) -> Result<Vec<Item>, Refused> {
    let seed = batch.seed.unwrap_or_else(|| new_id().seed().value());
    let made_at = Some(world.made_now());
    let one = |n: u32| {
        let seed = batch_seed(seed, if batch.identical { 0 } else { n });
        try_make_item(&batch.order, wearer, new_id(), seed, made_at, &mut new_id)
    };
    let first = one(0)?;
    let most = match batch.identical {
        true => world.stacks.most_of(&first),
        false => world.stacks.most,
    };
    let count = batch.count.clamp(1, most.max(1));
    let rest: Result<Vec<Item>, Refused> = (1..count).map(one).collect();
    Ok(std::iter::once(first).chain(rest?).collect())
}

/// What is left over, as it goes on the ground: items alike as one stack.
fn on_the_ground(mut left: Vec<Item>, identical: bool) -> Vec<Item> {
    if !identical || left.len() < 2 {
        return left;
    }
    let mut bottom = left.remove(0);
    for one in left {
        bottom.put_on(one);
    }
    vec![bottom]
}

#[cfg(test)]
mod tests;
