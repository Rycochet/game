//! What lies on the ground of a site's levels: bags anywhere, dropped by
//! players (or left by the dead), to be picked up by anyone within reach of
//! them (`shared::ground::in_reach`). Never on the world map. Kept in
//! `LevelSave::ground`, so everyone on a level shares it.

use bevy::prelude::*;
use shared::energy::handling_cost;
use shared::id::Id;
use shared::item::{Holder, Item, Rotation};

use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

mod bags;
mod tell;

pub(crate) use bags::{drop_at, put_down};
use bags::{leave_on_ground, lying, standing_at, take_up};
use tell::InReach;
pub(crate) use tell::sync;

/// Something a client asked to do with the ground, in turn with its steps: where the
/// player is decides what it means.
#[derive(Clone, Debug)]
pub(crate) enum GroundAction {
    Drop {
        item: Id,
        /// Part of a stack, lifted out when it is dropped.
        part: Option<StackPart>,
    },
    Take {
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    Stack {
        item: Id,
        onto: Id,
    },
    Wear {
        item: Id,
    },
    Bin {
        item: Id,
    },
}

/// Part of a carried stack, to be lifted out on its own when its turn comes.
#[derive(Clone, Debug)]
pub(crate) enum StackPart {
    /// The stack and some of its items, a group from a mixed one (`FromStack`).
    Group(Id, Vec<Id>),
    /// The stack and how many from its top (`PartOfStack`): new, for a counted one.
    Count(Id, u32),
}

/// What a client was last told about the ground in its reach.
#[derive(Default)]
pub(crate) struct GroundTold {
    /// The bags in reach and what was in them. `None` if there were none.
    reached: Option<InReach>,
    /// It guessed at something the server has now answered: it needs telling again,
    /// even if nothing changed.
    pub(crate) again: bool,
    /// It has been told the bags its character remembers on this map.
    bags_told: bool,
}

/// Moves something carried to the ground by the player: what it cost.
fn dropped(
    client: &mut Client,
    world: &mut ServerWorld,
    item: Id,
    part: Option<StackPart>,
) -> Result<u32, &'static str> {
    standing_at(client).ok_or("not in a site")?;
    // Part of a stack: see `connections::stack_parts`.
    let item = match part {
        Some(StackPart::Group(stack, items)) => {
            client.inventory.lift_group(stack, &items);
            item
        }
        Some(StackPart::Count(stack, count)) => client
            .inventory
            .lift_count(stack, count, &mut new_id)
            .ok_or("no such part of a stack")?,
        None => {
            client.inventory.lift_part(item);
            item
        }
    };
    if !client.may_take_off(item) {
        return Err("it won't come off");
    }
    let cost = client.inventory.put_down_cost(item);
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't put that down")?;
    // The client asks first: `put_down` destroys quest items.
    if item.holds_quest_item() {
        info!(
            "{} destroyed quest items with {:?}",
            client.in_log(),
            item.id
        );
    }
    put_down(client, world, item);
    Ok(cost)
}

/// Does what was asked. Whether the client is still there, and what it cost, which
/// is nothing if it was refused. Anything refused is put right by the answers.
pub(crate) fn act(
    client: &mut Client,
    world: &mut ServerWorld,
    action: GroundAction,
) -> (bool, u32) {
    let done = match action {
        GroundAction::Drop { item, ref part } => dropped(client, world, item, part.clone()),
        GroundAction::Take {
            item,
            to,
            x,
            y,
            rotation,
        } => lying(client, world, item).and_then(|found| {
            let grams = found.total_weight();
            client
                .inventory
                .add(found, to, x, y, rotation)
                .map_err(|_| "it can't go there")?;
            take_up(client, world, item);
            Ok(handling_cost(grams))
        }),
        GroundAction::Stack { item, onto } => lying(client, world, item).and_then(|found| {
            let stacks = world.stacks;
            let rest = client
                .inventory
                .add_to_stack(found.clone(), onto, stacks, &mut new_id)
                .map_err(|_| "it can't go on that")?;
            let left = rest.as_ref().map_or(0, Item::total_weight);
            leave_on_ground(client, world, item, rest);
            Ok(handling_cost(found.total_weight() - left))
        }),
        GroundAction::Wear { item } => lying(client, world, item).and_then(|mut found| {
            // Only the top of a stack goes on: the rest stays on the ground.
            let top = found.take_top(&mut new_id);
            let rest = top.is_some().then(|| found.clone());
            let wearing = top.unwrap_or(found);
            let grams = wearing.total_weight();
            // What is in its way comes off, or the player is told why not.
            let dressing = client
                .put_on_new(wearing, world.stacks)
                .map_err(|_| "it can't go on")?;
            leave_on_ground(client, world, item, rest);
            Ok(dressing + handling_cost(grams))
        }),
        GroundAction::Bin { item } => lying(client, world, item).map(|found| {
            client.inventory.to_bin_new(found);
            take_up(client, world, item);
            0
        }),
    };
    let cost = match done {
        Ok(cost) => {
            client.unsaved = true;
            cost
        }
        Err(why) => {
            warn!("Refused {action:?}: {why}");
            0
        }
    };
    client.update_speed(world.action_clock.ticks());
    client.ground_told.again = true;
    let sent = client.send_inventory();
    (sent, cost)
}

#[cfg(test)]
mod tests;
