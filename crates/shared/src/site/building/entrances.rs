//! Doors in the outer wall, and the cells either side of one.

use super::super::grid::{STEPS, roll};
use super::{CELLS_PER_EXTRA_ENTRANCE, MIN_ENTRANCE_GAP, Shape, Spot};
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// Doors in the outer wall: one, preferably facing south (the side seen from
/// the world map), and up to three more for bigger buildings. Inside rooms
/// are only joined up to what these doors already reach, so a side door may
/// be the only way into part of the building.
/// Returns each door and the ground outside it.
pub(super) fn add_entrances(
    level: &mut Level,
    shape: &Shape,
    rng: &mut Rng,
) -> Vec<((i64, i64), (i64, i64))> {
    // Outer wall cells with ground straight out, floor straight in, and wall
    // either side.
    let mut candidates: Vec<(Spot, Spot, usize)> = Vec::new();
    for y in shape.y..shape.y + shape.height {
        for x in shape.x..shape.x + shape.width {
            if !shape.contains(x, y) || shape.is_interior(x, y) {
                continue;
            }
            for (side, (dx, dy)) in STEPS.into_iter().enumerate() {
                let outside = (x + dx, y + dy);
                let inside = (x - dx, y - dy);
                if level.cell(outside.0, outside.1) == Cell::Ground
                    && level.cell(inside.0, inside.1) == Cell::Floor
                    && level.cell(x + dy, y + dx) == Cell::Wall
                    && level.cell(x - dy, y - dx) == Cell::Wall
                {
                    candidates.push(((x, y), outside, side));
                }
            }
        }
    }
    let mut chosen = Vec::new();
    // STEPS[2] is south.
    let south: Vec<_> = candidates.iter().filter(|c| c.2 == 2).copied().collect();
    let pool = if south.is_empty() {
        &candidates
    } else {
        &south
    };
    if pool.is_empty() {
        return Vec::new();
    }
    let first = pool[rng.range_u64(0, pool.len() as u64) as usize];
    chosen.push(first);
    // More doors for bigger buildings, on sides without one if possible and
    // never close to another.
    let extra = roll(
        rng,
        0,
        ((shape.width + shape.height) / CELLS_PER_EXTRA_ENTRANCE).min(3),
    );
    for _ in 0..extra {
        let apart = |c: &(Spot, Spot, usize)| {
            chosen.iter().all(|d: &(Spot, Spot, usize)| {
                (d.0.0 - c.0.0).abs() + (d.0.1 - c.0.1).abs() >= MIN_ENTRANCE_GAP
            })
        };
        let new_side: Vec<_> = candidates
            .iter()
            .filter(|c| apart(c) && chosen.iter().all(|d| d.2 != c.2))
            .copied()
            .collect();
        let any_side: Vec<_> = candidates.iter().filter(|c| apart(c)).copied().collect();
        let pool = if new_side.is_empty() {
            any_side
        } else {
            new_side
        };
        if pool.is_empty() {
            break;
        }
        chosen.push(pool[rng.range_u64(0, pool.len() as u64) as usize]);
    }
    for &((x, y), _, _) in &chosen {
        level.set(x, y, Cell::Door);
    }
    chosen
        .into_iter()
        .map(|(door, outside, _)| (door, outside))
        .collect()
}

/// A wall cell inside the building between two walkable cells (north and
/// south, or east and west).
pub(super) fn between(level: &Level, x: i64, y: i64) -> Option<((i64, i64), (i64, i64))> {
    [(0, 1), (1, 0)].into_iter().find_map(|(dx, dy)| {
        let (a, b) = ((x - dx, y - dy), (x + dx, y + dy));
        (level.cell(a.0, a.1) == Cell::Floor && level.cell(b.0, b.1) == Cell::Floor)
            .then_some((a, b))
    })
}
