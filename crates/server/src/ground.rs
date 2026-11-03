//! What lies on the ground of a site's levels: dropped by players, to be picked up
//! by anyone standing on the same cell. Never on the world map. Kept in
//! `LevelSave::ground`, so everyone on a level shares it.

use bevy::prelude::*;
use shared::energy::{dressing_cost, handling_cost};
use shared::ground::Load;
use shared::id::Id;
use shared::item::{Holder, Item, Rotation};
use shared::poi::PoiKind;
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::{Client, Phase};
use crate::storage::{LevelSave, SiteId};

/// Something a client asked to do with the ground, in turn with its steps: where the
/// player is decides what it means.
#[derive(Clone, Copy, Debug)]
pub(crate) enum GroundAction {
    Drop {
        item: Id,
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

/// What lay on a cell: each thing's id, with how many there were of it.
type Pile = Vec<(Id, u32)>;

/// What a client was last told about the ground under it.
#[derive(Default)]
pub(crate) struct GroundTold {
    /// The cell, and what lay there, with how many of each. `None` if nothing did.
    pile: Option<((i64, i64), Pile)>,
    /// It guessed at something the server has now answered: it needs telling again,
    /// even if nothing changed.
    pub(crate) again: bool,
    /// It has been told the bags its character remembers on this map.
    bags_told: bool,
}

/// The cell the player stands on, and which level it is on.
fn standing_on(client: &Client) -> Option<(SiteId, PoiKind, u32, (i64, i64))> {
    let visit = client.visit.as_ref()?;
    Some((visit.id(), visit.kind(), visit.index(), visit.position()))
}

/// The level the player is on, to change.
fn level_mut<'a>(world: &'a mut ServerWorld, client: &Client) -> Option<&'a mut LevelSave> {
    let (id, kind, index, _) = standing_on(client)?;
    Some(world.site_mut(id, kind).levels.entry(index).or_default())
}

/// Takes this off the ground under the player. What is left empty goes.
fn take_up(client: &Client, world: &mut ServerWorld, item: Id) -> Option<Item> {
    let (.., at) = standing_on(client)?;
    let level = level_mut(world, client)?;
    let pile = level.ground.get_mut(&at)?;
    let place = pile.iter().position(|one| one.id == item)?;
    let taken = pile.remove(place);
    if pile.is_empty() {
        level.ground.remove(&at);
    }
    Some(taken)
}

/// What is left on the ground of something partly taken up: `None` if all of it was.
fn leave_on_ground(client: &Client, world: &mut ServerWorld, item: Id, rest: Option<Item>) {
    let Some(rest) = rest else {
        take_up(client, world, item);
        return;
    };
    let Some((.., at)) = standing_on(client) else {
        return;
    };
    let lying = level_mut(world, client).and_then(|level| level.ground.get_mut(&at));
    if let Some(one) = lying.and_then(|pile| pile.iter_mut().find(|one| one.id == item)) {
        *one = rest;
    }
}

/// Puts this on the ground under the player, as the newest there.
fn put_down(client: &Client, world: &mut ServerWorld, item: Item) {
    if let Some((.., at)) = standing_on(client)
        && let Some(level) = level_mut(world, client)
    {
        level.ground.entry(at).or_default().push(item);
    }
}

/// Moves something carried to the ground under the player: what it cost.
fn dropped(client: &mut Client, world: &mut ServerWorld, item: Id) -> Result<u32, &'static str> {
    standing_on(client).ok_or("not in a site")?;
    // Part of a stack: see `answer_items`.
    client.inventory.lift_part(item);
    if !client.may_take_off(item) {
        return Err("it won't come off");
    }
    let cost = client.inventory.put_down_cost(item);
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't put that down")?;
    put_down(client, world, item);
    Ok(cost)
}

/// Something on the ground under the player, as it is: it stays there until `take_up`.
fn lying(client: &Client, world: &mut ServerWorld, item: Id) -> Result<Item, &'static str> {
    let (id, kind, index, at) = standing_on(client).ok_or("not in a site")?;
    let level = world.site(id, kind).levels.get(&index);
    let pile = level.and_then(|level| level.ground.get(&at));
    let found = pile.and_then(|pile| pile.iter().find(|one| one.id == item));
    found.cloned().ok_or("that isn't on the ground here")
}

/// Does what was asked. Whether the client is still there, and what it cost, which
/// is nothing if it was refused. Anything refused is put right by the answers.
pub(crate) fn act(
    client: &mut Client,
    world: &mut ServerWorld,
    action: GroundAction,
) -> (bool, u32) {
    let done = match action {
        GroundAction::Drop { item } => dropped(client, world, item),
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
                .add_to_stack(found.clone(), onto, stacks)
                .map_err(|_| "it can't go on that")?;
            let left = rest.as_ref().map_or(0, Item::total_weight);
            leave_on_ground(client, world, item, rest);
            Ok(handling_cost(found.total_weight() - left))
        }),
        GroundAction::Wear { item } => lying(client, world, item).and_then(|mut found| {
            // Only the top of a stack goes on: the rest stays on the ground.
            let top = found.take_top();
            let rest = top.is_some().then(|| found.clone());
            let wearing = top.unwrap_or(found);
            let (worn, grams) = (wearing.id, wearing.total_weight());
            client
                .inventory
                .wear_new(wearing)
                .map_err(|_| "it can't go on")?;
            client.inventory.magic_found_out(worn);
            leave_on_ground(client, world, item, rest);
            Ok(dressing_cost(grams) + handling_cost(grams))
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

/// Tells each player what is under them, when it isn't what they were last told. A
/// client that has gone is noticed when its messages are next read.
pub(crate) fn sync(clients: &mut [Client], world: &mut ServerWorld) {
    for client in clients {
        if client.phase != Phase::Playing {
            continue;
        }
        let Some((id, kind, index, at)) = standing_on(client) else {
            continue;
        };
        let level = world.site(id, kind).levels.get(&index);
        let items = level.and_then(|level| level.ground.get(&at));
        let pile = items.map(|items| {
            let counted = items.iter().map(|item| (item.id, item.count()));
            (at, counted.collect())
        });
        if pile != client.ground_told.pile || client.ground_told.again {
            let items = client.as_seen(items.cloned().unwrap_or_default());
            client.link.send(&ServerMessage::GroundItems {
                x: at.0,
                y: at.1,
                items,
            });
            client.ground_told.pile = pile;
            client.ground_told.again = false;
        }
        tell_bags(client, world);
    }
}

/// Tells a client about bags: on arriving, those its character remembers; after that
/// only what has just been seen to differ, which is all its character remembers too.
fn tell_bags(client: &mut Client, world: &mut ServerWorld) {
    let Some((id, kind, index, _)) = standing_on(client) else {
        return;
    };
    let level = world.site(id, kind).levels.get(&index);
    let ground = level.map(|level| &level.ground);
    let piles: Vec<((i64, i64), Load)> = ground
        .into_iter()
        .flatten()
        .map(|(&cell, items)| (cell, Load::of(items)))
        .collect();
    let levels = client.character.bags.get(&id);
    let mut known = levels
        .and_then(|levels| levels.get(&index))
        .cloned()
        .unwrap_or_default();
    if !client.ground_told.bags_told {
        client.ground_told.bags_told = true;
        for (&(x, y), &load) in &known {
            client.link.send(&ServerMessage::GroundSeen { x, y, load });
        }
    }
    let Some(visit) = client.visit.as_mut() else {
        return;
    };
    let mut changed = false;
    for &(cell, load) in &piles {
        if known.get(&cell) != Some(&load) && visit.can_see(cell.0, cell.1) {
            known.insert(cell, load);
            changed = true;
            let (x, y) = cell;
            client.link.send(&ServerMessage::GroundSeen { x, y, load });
        }
    }
    let gone: Vec<(i64, i64)> = known
        .keys()
        .filter(|cell| !piles.iter().any(|(pile, _)| pile == *cell))
        .filter(|cell| visit.can_see(cell.0, cell.1))
        .copied()
        .collect();
    for (x, y) in gone {
        known.remove(&(x, y));
        changed = true;
        client.link.send(&ServerMessage::GroundGone { x, y });
    }
    if changed {
        let levels = client.character.bags.entry(id).or_default();
        levels.insert(index, known);
        client.unsaved = true;
    }
}

#[cfg(test)]
mod tests;
