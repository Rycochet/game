//! Steps and teleports within a site, and setting a cell.

use bevy::prelude::*;
use shared::dungeon::{Cell, Sight};
use shared::movement::{Step, step_on_level};
use shared::protocol::{ServerMessage, level_chunk};

use super::update_sight;
use crate::ServerWorld;
use crate::connections::Client;

/// The player takes a step on their level: taken if it leads onto a
/// walkable cell (otherwise `Position` puts the client right), then sends
/// what they now see or are about to.
pub(crate) fn stepped(client: &mut Client, step: Step) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let Some(position) = step_on_level(&visit.level.level, visit.position, step) else {
        let (x, y) = step.from(visit.position);
        info!("Refused a step onto an unwalkable cell ({x}, {y})");
        return true;
    };
    visit.position = position;
    client.unsaved = true;
    update_sight(client)
}

/// Developer tools: puts the player on `(x, y)` on their level, if it is
/// walkable (through walls, unlike a step).
pub(crate) fn teleport(client: &mut Client, x: i64, y: i64) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    if !visit.level.level.is_walkable(x, y) {
        info!("Refused a teleport onto an unwalkable cell ({x}, {y})");
        return true;
    }
    visit.position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y }) && update_sight(client)
}

/// Developer tools: changes a cell of the player's level (kept as a player's
/// change), and sends it again with what can be seen now.
pub(crate) fn set_cell(
    client: &mut Client,
    world: &mut ServerWorld,
    x: i64,
    y: i64,
    cell: Cell,
) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    if !visit.level.level.contains(x, y) || visit.position == (x, y) {
        return true;
    }
    visit.level.level.set(x, y, cell);
    let site = world.site_mut(visit.id, visit.poi.kind);
    let saved = site.levels.entry(visit.index).or_default();
    saved.changes.insert((x, y), cell);
    // Everything may look different from everywhere now.
    visit.view_from = None;
    visit.looked_from = Sight::new(&visit.level.level);
    visit.loaded = Sight::new(&visit.level.level);
    let (chunk, bit) = level_chunk(x, y);
    if let Some(mask) = visit.sent.get_mut(&chunk) {
        *mask &= !(1 << bit);
    }
    update_sight(client)
}
