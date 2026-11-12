//! Moves and teleports within a site, and setting a cell.

use bevy::prelude::*;
use shared::clearance::check_segment;
use shared::dungeon::{Cell, Sight};
use shared::protocol::{ServerMessage, level_chunk};
use shared::spot::Spot;

use super::doors::changed_at;
use super::update_sight;
use crate::ServerWorld;
use crate::connections::Client;

/// The player moves to `to` on their level, already checked
/// (`Visit::landing`), then is sent what they now see or are about to.
pub(crate) fn moved(client: &mut Client, to: Spot) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    visit.position = to;
    client.unsaved = true;
    update_sight(client)
}

/// A move to `to` was refused (`Position` puts the client right). A door the
/// client took for open is told again as it hangs.
pub(crate) fn refused(client: &mut Client, to: Spot) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let (x, y) = to.cell();
    info!("Refused a move to {to:?}, in cell ({x}, {y})");
    if visit.doors.contains_key(&(x, y)) {
        changed_at(visit, x, y);
        return update_sight(client);
    }
    true
}

/// Developer tools: puts the player at `to` (units) on their level, anywhere
/// with footing and room (through walls, unlike a move).
pub(crate) fn teleport(client: &mut Client, to: Spot) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let level = &visit.level.level;
    let size = client.inventory.creature.def().size;
    // Room to stand: a move along a hair's breadth from it would be allowed.
    let room = check_segment(level, size, to, to.offset(1, 0)).is_some()
        || check_segment(level, size, to, to.offset(-1, 0)).is_some();
    if !room {
        info!("Refused a teleport to {to:?}, with no room to stand");
        return true;
    }
    visit.position = to;
    client.unsaved = true;
    client.changed_map();
    client
        .link
        .send(&ServerMessage::Teleported { x: to.x, y: to.y })
        && update_sight(client)
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
    if !visit.level.level.contains(x, y) || visit.position() == (x, y) {
        return true;
    }
    visit.level.level.set(x, y, cell);
    // A cutout's cell in the wilderness is its site's.
    let mut mirrored = visit.mirrors.iter().filter(|_| visit.is_wilderness());
    let mirrored = mirrored.find_map(|m| m.of(x, y).map(|at| (m.id, m.index, at)));
    let (id, index, at) = mirrored.unwrap_or((visit.id, visit.index, (visit.kept_at(x), y)));
    let saved = world.site_mut(id).levels.entry(index).or_default();
    saved.changes.insert(at, cell);
    let key = super::doors::DoorKey {
        site: id,
        index,
        at,
    };
    super::doors::cell_changed(world, visit, (x, y), key);
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
