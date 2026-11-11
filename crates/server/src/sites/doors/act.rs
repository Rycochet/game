//! Opening and closing doors: what a player asks, checked here.

use bevy::prelude::*;
use shared::door::{Side, Swing, carries_key, door_cost};
use shared::dungeon::{Cell, Sight};
use shared::protocol::{ServerMessage, level_chunk};

use super::{VisitDoor, keep_door};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::{Visit, update_sight};

/// The player opens the closed door at `(x, y)` beside them: away from them,
/// or towards them if it is a secret door, which they must have found. Whether
/// the client is still there, and what it cost. Refused with `DoorRefused`.
pub(crate) fn open_door(
    client: &mut Client,
    world: &mut ServerWorld,
    x: i64,
    y: i64,
) -> (bool, u32) {
    let Some(visit) = &client.visit else {
        return (true, 0);
    };
    let (px, py) = visit.position;
    let beside = (x - px).abs().max((y - py).abs()) == 1;
    let known = client.character.doors.get(&visit.id);
    let known = known.and_then(|levels| levels.get(&visit.index));
    let found = client.truesight || known.is_some_and(|d| d.contains_key(&(visit.kept_at(x), y)));
    let secret = visit.level.level.cell(x, y) == Cell::SecretDoor;
    let door = visit.doors.get(&(x, y)).copied().filter(|_| beside);
    let Some(mut door) = door.filter(|_| found || !secret) else {
        // Not beside them, or not known to be a door.
        info!("Refused opening a door at ({x}, {y})");
        let refused = ServerMessage::DoorRefused {
            x,
            y,
            locked: false,
        };
        return (client.link.send(&refused), 0);
    };
    if !door.door.swing.is_shut() {
        // Already open: the client is told again how it is.
        let visit = client.visit.as_mut().expect("checked above");
        changed_at(visit, x, y);
        return (update_sight(client), 0);
    }
    // Locked: unlocked first, with its key, or it stays shut.
    let mut cost = door_cost();
    if let Some(lock) = door.door.lock.as_mut().filter(|lock| lock.locked) {
        if !carries_key(&client.inventory, lock.id) {
            info!("A locked door at ({x}, {y}) stays shut");
            let refused = ServerMessage::DoorRefused { x, y, locked: true };
            return (client.link.send(&refused), door_cost());
        }
        lock.locked = false;
        cost += door_cost();
    }
    let runs = visit.level.level.opening_runs_east_west(x, y);
    let from = Side::of(runs, (x, y), (px, py));
    let side = if secret { from } else { from.opposite() };
    door.door.swing = Swing::Open(side);
    let visit = client.visit.as_mut().expect("checked above");
    set_here(visit, (x, y), door);
    keep_door(world, door.key, door.door);
    (update_sight(client), cost)
}

/// The player closes every open door beside them with nothing in its doorway
/// (broken ones stay open), and locks every closed one whose key they carry.
/// What it cost: nothing if nothing changed.
pub(crate) fn close_near(client: &mut Client, world: &mut ServerWorld) -> (bool, u32) {
    let Some(visit) = &client.visit else {
        return (true, 0);
    };
    let (px, py) = visit.position;
    let near: Vec<((i64, i64), VisitDoor)> = (visit.doors.iter())
        .filter(|((x, y), _)| (x - px).abs().max((y - py).abs()) == 1)
        .filter(|(_, door)| match door.door.swing {
            Swing::Open(_) => true,
            Swing::Closed => door
                .door
                .lock
                .is_some_and(|lock| !lock.locked && carries_key(&client.inventory, lock.id)),
            Swing::Broken(_) => false,
        })
        .map(|(&at, &door)| (at, door))
        .collect();
    let mut closed = false;
    for (at, mut door) in near {
        if door.door.swing.is_shut() {
            if let Some(lock) = &mut door.door.lock {
                lock.locked = true;
            }
        } else if world.someone_in_doorway(client, at, door.key) {
            continue;
        }
        door.door.swing = Swing::Closed;
        let visit = client.visit.as_mut().expect("checked above");
        set_here(visit, at, door);
        keep_door(world, door.key, door.door);
        closed = true;
    }
    if !closed {
        return (true, 0);
    }
    (update_sight(client), door_cost())
}

/// Developer tools: opens (away from the player), or closes, every door on the
/// player's level that can be: broken doors stay open, and doorways with
/// someone or something in them stay as they are.
pub(crate) fn dev_all(client: &mut Client, world: &mut ServerWorld, open: bool) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let position = visit.position;
    let doors: Vec<((i64, i64), VisitDoor)> = (visit.doors.iter())
        .map(|(&at, &door)| (at, door))
        .collect();
    for (at, mut door) in doors {
        let visit = client.visit.as_ref().expect("checked above");
        let runs = visit.level.level.opening_runs_east_west(at.0, at.1);
        let swing = match (open, door.door.swing) {
            (true, Swing::Closed) => Swing::Open(Side::of(runs, at, position).opposite()),
            (false, Swing::Open(_)) if !world.someone_in_doorway(client, at, door.key) => {
                Swing::Closed
            }
            _ => continue,
        };
        door.door.swing = swing;
        let visit = client.visit.as_mut().expect("checked above");
        set_here(visit, at, door);
        keep_door(world, door.key, door.door);
    }
    update_sight(client)
}

/// Sets a door on the visit's own level, to be sent again.
pub(super) fn set_here(visit: &mut Visit, (x, y): (i64, i64), door: VisitDoor) {
    visit.doors.insert((x, y), door);
    visit.level.level.set_swing(x, y, door.door.swing);
    visit.level.level.set_locked(x, y, door.door.is_locked());
    changed_at(visit, x, y);
}

/// A cell of the visit's level has changed: it is sent again, and sight worked
/// out afresh (a door may hide or show anything).
pub(in crate::sites) fn changed_at(visit: &mut Visit, x: i64, y: i64) {
    visit.view_from = None;
    visit.looked_from = Sight::new(&visit.level.level);
    visit.loaded = Sight::new(&visit.level.level);
    let (chunk, bit) = level_chunk(x, y);
    if let Some(mask) = visit.sent.get_mut(&chunk) {
        *mask &= !(1 << bit);
    }
}

/// Developer tools: the player is given the key to every lock on a door beside
/// them, or if none, to the nearest lock on their level (into their pack, or
/// onto the ground under them if it has no room).
pub(crate) fn dev_key(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let (px, py) = visit.position;
    let apart = |&(x, y): &(i64, i64)| (x - px).abs().max((y - py).abs());
    let locks: Vec<((i64, i64), shared::id::Id)> = (visit.doors.iter())
        .filter_map(|(&at, door)| door.door.lock.map(|lock| (at, lock.id)))
        .collect();
    let beside: Vec<_> = locks.iter().filter(|(at, _)| apart(at) == 1).collect();
    let chosen: Vec<shared::id::Id> = if beside.is_empty() {
        locks
            .iter()
            .min_by_key(|(at, _)| apart(at))
            .map(|&(_, id)| id)
            .into_iter()
            .collect()
    } else {
        beside.iter().map(|&&(_, id)| id).collect()
    };
    if chosen.is_empty() {
        info!("No lock on this level to give a key for");
        return true;
    }
    for lock in chosen {
        let mut key = shared::item::Item::new(crate::ids::new_id(), shared::item::ItemKind::Key);
        key.form = Some(shared::item_form::ItemForm::Key(lock));
        if let Err(key) = client.inventory.give(key, world.stacks) {
            crate::ground::put_down(client, world, key);
        }
    }
    client.unsaved = true;
    client.send_inventory()
}
