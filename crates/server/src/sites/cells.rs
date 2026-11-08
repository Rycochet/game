//! Sending cells: all of a level, those beside what is seen, nearby, and level chunks.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level};
use shared::movement::{Step, step_on_level};
use shared::protocol::{LevelChunk, SeenStairs, ServerMessage, level_chunk};
use std::collections::VecDeque;

use super::Visit;
use crate::connections::Client;
use crate::storage::CellMasks;

/// Every cell of a level.
pub(super) fn all_cells(level: &Level) -> CellMasks {
    cells_in(level.bounds())
}

/// Every cell in `(min_x, min_y, width, height)`.
pub(super) fn cells_in((min_x, min_y, width, height): (i64, i64, i64, i64)) -> CellMasks {
    let mut masks = CellMasks::new();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let (chunk, bit) = level_chunk(x, y);
            *masks.entry(chunk).or_default() |= 1 << bit;
        }
    }
    masks
}

/// The cells next to any cell for which `is_in` holds, on `level`, that
/// `except` doesn't include.
pub(super) fn beside(
    level: &Level,
    is_in: impl Fn(i64, i64) -> bool,
    except: &CellMasks,
) -> CellMasks {
    let mut found = CellMasks::new();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let (chunk, bit) = level_chunk(x, y);
            let done = except.get(&chunk).is_some_and(|m| m & (1 << bit) != 0);
            let next_to = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                .iter()
                .any(|(dx, dy)| is_in(x + dx, y + dy));
            if !done && !is_in(x, y) && next_to {
                *found.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    found
}

/// Leaves out of `masks` the cells outside `bounds`: those a client lets go
/// of when the wilderness's window moves.
pub(super) fn keep_within(masks: &mut CellMasks, bounds: (i64, i64, i64, i64)) {
    let inside = cells_in(bounds);
    for (chunk, mask) in masks.iter_mut() {
        *mask &= inside.get(chunk).copied().unwrap_or(0);
    }
    masks.retain(|_, mask| *mask != 0);
}

/// Adds the cells in `more` to `masks`.
pub(super) fn add_masks(masks: &mut CellMasks, more: CellMasks) {
    for (chunk, mask) in more {
        *masks.entry(chunk).or_default() |= mask;
    }
}

/// The walkable cells the player could reach within `steps` steps (in any
/// of 8 directions, a jump over liquid counting as one), including where they
/// stand.
pub(super) fn nearby_cells(level: &Level, from: (i64, i64), steps: i64) -> Vec<(i64, i64)> {
    let mut found = vec![from];
    let mut queue = VecDeque::from([(from, 0)]);
    while let Some((at, distance)) = queue.pop_front() {
        if distance == steps {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                let Some(step) = Step::new(dx, dy) else {
                    continue;
                };
                if let Some(next) = step_on_level(level, at, step)
                    && !found.contains(&next)
                {
                    found.push(next);
                    queue.push_back((next, distance + 1));
                }
            }
        }
    }
    found
}

/// Sends the cells in `masks` as the player would see them, a chunk at a
/// time, with any stairs on them.
pub(super) fn send_cells(
    client: &Client,
    visit: &Visit,
    masks: &CellMasks,
    remembered: bool,
) -> bool {
    if !super::wild::send_flats(client, visit, masks) {
        return false;
    }
    let truesight = client.truesight;
    level_chunks(
        &visit.level.level,
        &visit.stairs,
        masks,
        remembered,
        truesight,
    )
    .into_iter()
    .all(|chunk| client.link.send(&ServerMessage::LevelCells(chunk)))
}

/// The cells in `masks` as the player would see them, a chunk at a time, with
/// any stairs on them.
pub(super) fn level_chunks(
    level: &Level,
    stairs: &[SeenStairs],
    masks: &CellMasks,
    remembered: bool,
    truesight: bool,
) -> Vec<LevelChunk> {
    let mut chunks = Vec::with_capacity(masks.len());
    for (&chunk, &mask) in masks {
        let message = LevelChunk {
            chunk,
            mask,
            cells: Vec::new(),
            stairs: Vec::new(),
            remembered,
        };
        // The positions come from the mask; fill in what is there.
        let positions: Vec<(i64, i64)> = message.positions().collect();
        let cells = positions
            .iter()
            .map(|&(x, y)| match level.cell(x, y) {
                // Hidden until found, but not from truesight.
                Cell::SecretDoor if !truesight => Cell::Wall,
                cell => cell,
            })
            .collect();
        let stairs = stairs
            .iter()
            .filter(|s| positions.contains(&(s.x, s.y)))
            .copied()
            .collect();
        chunks.push(LevelChunk {
            cells,
            stairs,
            ..message
        });
    }
    chunks
}
