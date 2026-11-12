//! What the player sees and remembers: truesight, revealing and forgetting a level.

use shared::dungeon::{Cell, Sight};
use shared::protocol::level_chunk;

use super::{
    Visit, add_masks, all_cells, arrive_again, beside, nearby_cells, remember_doors, send_cells,
};
use crate::connections::Client;
use crate::storage::CellMasks;
use crate::{LEVEL_PRELOAD_STEPS, ServerWorld};

/// Developer tools: truesight has just changed. If it changes how this level looks,
/// it is sent again. Only then, as that closes whatever is open on it.
pub(crate) fn truesight_changed(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let level = &visit.level.level;
    let (min_x, min_y, width, height) = level.bounds();
    let hides_something = (min_y..min_y + height)
        .any(|y| (min_x..min_x + width).any(|x| level.cell(x, y) == Cell::SecretDoor));
    !hides_something || arrive_again(client, world)
}

/// Developer tools: the character knows the whole of the level they are on. A
/// secret door stays a wall, as it does when seen, unless they have truesight.
pub(crate) fn reveal_level(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    // A town is known already.
    if !visit.is_town() {
        *memory(client, &visit) = all_cells(&visit.level.level);
        client.visit = Some(visit);
        return arrive_again(client, world);
    }
    client.visit = Some(visit);
    true
}

/// Developer tools: the character forgets the level they are on.
pub(crate) fn forget_level(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    // A town would only be sent whole again.
    if !visit.is_town() {
        memory(client, &visit).clear();
        if let Some(levels) = client.character.doors.get_mut(&visit.id) {
            levels.remove(&visit.index);
        }
        client.visit = Some(visit);
        return arrive_again(client, world);
    }
    client.visit = Some(visit);
    true
}

/// What the character remembers of the visit's level.
pub(super) fn memory<'a>(client: &'a mut Client, visit: &Visit) -> &'a mut CellMasks {
    let site = client.character.memory.entry(visit.id).or_default();
    site.entry(visit.index).or_default()
}

/// Works out what the player sees (remembered from now on) and what they
/// could see within `LEVEL_PRELOAD_STEPS`, and sends the cells of that the
/// client doesn't have yet.
pub(crate) fn update_sight(client: &mut Client) -> bool {
    let cone = client.cone();
    let Some(mut visit) = client.visit.take() else {
        return true;
    };
    if let Some(cone) = cone
        && visit.further != cone.field.further
    {
        visit.further = cone.field.further;
        visit.view_from = None;
    }
    if visit.is_town() {
        // The client has it all, except any cell that has just changed.
        let mut new = all_cells(&visit.level.level);
        for (chunk, sent) in &visit.sent {
            if let Some(mask) = new.get_mut(chunk) {
                *mask &= !sent;
            }
        }
        new.retain(|_, mask| *mask != 0);
        let sent = send_cells(client, &visit, &new, true);
        let visit = client.visit.insert(visit);
        add_masks(&mut visit.sent, new);
        return sent;
    }
    let reach = visit.reach();
    let (x, y) = visit.position();
    let (looked_from, loaded) = (&mut visit.looked_from, &mut visit.loaded);
    let level = &visit.level.level;
    // What the client draws: glimpses, the more generous sight, in the cone.
    let mut seen = Sight::new(level);
    match &cone {
        Some(cone) => seen.add_glimpses_in(level, x, y, reach, cone),
        None => seen.add_glimpses(level, x, y, reach),
    }
    for (nx, ny) in nearby_cells(level, (x, y), LEVEL_PRELOAD_STEPS) {
        if !looked_from.is_seen(nx, ny) {
            looked_from.insert(nx, ny);
            loaded.add_glimpses(level, nx, ny, reach);
        }
    }

    // Doors seen are remembered as they are, and sent again if that is news.
    let doors_seen: Vec<(i64, i64)> = (visit.doors.keys().copied())
        .filter(|&(dx, dy)| seen.is_seen(dx, dy))
        .collect();
    for (dx, dy) in doors_seen {
        if remember_doors(client, &visit, (dx, dy)) {
            let (chunk, bit) = level_chunk(dx, dy);
            if let Some(mask) = visit.sent.get_mut(&chunk) {
                *mask &= !(1 << bit);
            }
        }
    }
    let memory = memory(client, &visit);
    let mut learnt = false;
    let mut new = CellMasks::new();
    let mut shared_cells = Vec::new();
    let (min_x, min_y, width, height) = level.bounds();
    for cy in min_y..min_y + height {
        for cx in min_x..min_x + width {
            let (chunk, bit) = level_chunk(cx, cy);
            if seen.is_seen(cx, cy) {
                let (kept, _) = level_chunk(visit.kept_at(cx), cy);
                let mask = memory.entry(kept).or_default();
                learnt |= *mask & (1 << bit) == 0;
                *mask |= 1 << bit;
                if visit.shares(cx, cy) {
                    shared_cells.push((cx, cy));
                }
            }
            let sent = visit.sent.get(&chunk).copied().unwrap_or(0);
            if visit.loaded.is_seen(cx, cy) && sent & (1 << bit) == 0 {
                *new.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    client.unsaved |= learnt;
    let told = learnt_elsewhere(client, &visit, &shared_cells);
    // Their neighbours too: a wall's shape depends on what is beside it.
    let extra = beside(level, |cx, cy| visit.loaded.is_seen(cx, cy), &visit.sent);
    add_masks(&mut new, extra);
    let sent = send_cells(client, &visit, &new, false);
    let visit = client.visit.insert(visit);
    add_masks(&mut visit.sent, new);
    told && sent
}

/// Cells just seen that are another map's too, or a poi's cutout in the
/// wilderness: they are remembered there as well, and a poi whose way in is
/// among them is known (`wild::found`). `false` if the client has gone.
fn learnt_elsewhere(client: &mut Client, visit: &Visit, cells: &[(i64, i64)]) -> bool {
    for mirror in &visit.mirrors {
        let memory = client.character.memory.entry(mirror.id).or_default();
        let memory = memory.entry(mirror.index).or_default();
        for &(x, y) in cells {
            if let Some((x, y)) = mirror.of(x, y) {
                let (chunk, bit) = level_chunk(x, y);
                *memory.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    super::wild::found_by_sight(client, visit, cells)
}
