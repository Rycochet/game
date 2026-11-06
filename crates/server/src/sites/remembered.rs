//! Sending what a player remembers of the level they are on.

use shared::protocol::{LEVEL_CHUNK_SIZE, level_chunk};
use shared::site::SiteLevel;
use shared::wilderness::Cutout;

use super::{Mirror, NewVisit, Visit, add_masks, all_cells, beside, memory, send_cells};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{CellMasks, SiteId};

/// Sends what the player remembers of the visit's level that the client hasn't
/// been sent on this visit (a town's layout is known from the start, and so
/// isn't kept as memory), and the cells beside it, which shape it.
pub(super) fn send_remembered(client: &mut Client, visit: &mut Visit) -> bool {
    let mut remembered = if visit.is_town() {
        all_cells(&visit.level.level)
    } else {
        remembered_here(client, visit)
    };
    for (chunk, sent) in &visit.sent {
        if let Some(mask) = remembered.get_mut(chunk) {
            *mask &= !sent;
        }
    }
    remembered.retain(|_, mask| *mask != 0);
    if !send_cells(client, visit, &remembered, true) {
        return false;
    }
    // Their neighbours too, not as remembered: they shape what is.
    let level = &visit.level.level;
    let is_remembered = |x, y| {
        let (chunk, bit) = level_chunk(x, y);
        remembered.get(&chunk).is_some_and(|m| m & (1 << bit) != 0)
    };
    let neighbours = beside(level, is_remembered, &visit.sent);
    let neighbours = without(neighbours, &remembered);
    if !send_cells(client, visit, &neighbours, false) {
        return false;
    }
    add_masks(&mut visit.sent, remembered);
    add_masks(&mut visit.sent, neighbours);
    true
}

/// What the character remembers of the cells of the visit's level, by its cells.
fn remembered_here(client: &mut Client, visit: &Visit) -> CellMasks {
    let Some(width) = visit.wraps_at() else {
        return memory(client, visit).clone();
    };
    // The wilderness: only its window, kept by where it is round the planet.
    let memory = memory(client, visit);
    let (min_x, min_y, columns, rows) = visit.level.level.bounds();
    let (first, _) = level_chunk(min_x, min_y);
    let (last, _) = level_chunk(min_x + columns - 1, min_y + rows - 1);
    let chunks_round = width / LEVEL_CHUNK_SIZE;
    let mut found = CellMasks::new();
    for cy in first.1..=last.1 {
        for cx in first.0..=last.0 {
            if let Some(&mask) = memory.get(&(cx.rem_euclid(chunks_round), cy)) {
                found.insert((cx, cy), mask);
            }
        }
    }
    found
}

/// `masks` without the cells in `except`.
fn without(mut masks: CellMasks, except: &CellMasks) -> CellMasks {
    for (chunk, mask) in except {
        if let Some(found) = masks.get_mut(chunk) {
            *found &= !mask;
        }
    }
    masks.retain(|_, mask| *mask != 0);
    masks
}

/// A poi site's first level, entered from the world map, is set into the
/// wilderness too: what is seen of it is kept for both.
pub(super) fn mirror_of_first(
    world: &ServerWorld,
    new: &NewVisit,
    index: u32,
    level: &SiteLevel,
) -> Vec<Mirror> {
    if index != 0 || new.from_wilderness || !matches!(new.id, SiteId::Poi(_)) {
        return Vec::new();
    }
    let cutout = Cutout::new(&world.wilderness.wild, new.poi, level);
    let Some(offset) = cutout.offset() else {
        return Vec::new();
    };
    vec![Mirror {
        id: SiteId::Wilderness,
        index: 0,
        offset,
        bounds: level.level.bounds(),
    }]
}
