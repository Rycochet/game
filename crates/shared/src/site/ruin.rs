//! Ruined buildings: damage done to a building's finished floors.
//!
//! The floorplan stays what it would be unruined; the damage has its own seed
//! and only ever takes cells away. Each floor depends on the one below (a floor
//! can't stand over a gap), so they are ruined from the ground up, together.
//!
//! Damage nibbles in from the edges: every cell next to a gap is eaten at
//! 10 × ruin %, then the neighbours of what was eaten at 10% less, and so on
//! until the chance is 0. That alone leaves the middle of a big building whole,
//! so the worst ruin also starts holes in the middle of its floors. On the
//! ground floor eaten cells become outside ground; above it they become holes,
//! and each storey counts as one level less ruined than the one below.

use std::collections::BTreeSet;

use super::building::Shape;
use super::grid::{self, Reach, STEPS, chance};
use super::{SiteLevel, StairsDirection};
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

/// The most ruined a building can be.
pub const MAX_RUIN: u8 = 5;

/// The chance of being eaten in the first pass, per level of ruin, in percent;
/// and how much less it is each pass after.
const PERCENT_PER_RUIN: u32 = 10;

/// Floor cells per hole started in the middle of a floor, by the ruin left on
/// that floor. Less ruined floors start none.
fn floor_cells_per_hole(ruin: u8) -> Option<i64> {
    match ruin {
        4 => Some(100),
        5.. => Some(40),
        _ => None,
    }
}

/// How far (counting diagonals as 1) a floor can be from a wall of the floor
/// below before it comes down.
const MAX_SPAN: i64 = 4;

/// Whether a cell has nothing there: outside ground, or a hole.
fn is_gap(cell: Cell) -> bool {
    cell.is_outside() || cell == Cell::Hole
}

/// Whether a staircase, or the cells round it, are at `(x, y)`.
fn near(stairs: (i64, i64), x: i64, y: i64) -> bool {
    (stairs.0 - x).abs() <= 1 && (stairs.1 - y).abs() <= 1
}

/// The cells of a level, row by row.
fn cells_of(level: &Level) -> impl Iterator<Item = (i64, i64)> + use<> {
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height).flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
}

/// Ruins a building's floors (all of them, intact, from the ground up), given
/// each floor's outline and `stairs[n]` joining floor `n` to floor `n + 1`, and
/// any stairs from the ground floor down to a basement. Floors above one with
/// no way left from its stairs down to any stairs up are dropped.
pub(super) fn ruin_floors(
    seed: Seed,
    ruin: u8,
    floors: Vec<SiteLevel>,
    shapes: &[Shape],
    stairs: &[Vec<(i64, i64)>],
    basement: Option<(i64, i64)>,
) -> Vec<SiteLevel> {
    let seed = seed.derive("ruin");
    let mut kept: Vec<SiteLevel> = Vec::new();
    for (index, (mut floor, shape)) in floors.into_iter().zip(shapes).enumerate() {
        // A staircase keeps the cells round it on its own floors and below.
        let protected = |x: i64, y: i64| {
            stairs
                .iter()
                .enumerate()
                .any(|(n, spots)| index <= n + 1 && spots.iter().any(|&s| near(s, x, y)))
                || (index == 0 && basement.is_some_and(|s| near(s, x, y)))
        };
        let below = kept.last().map(|floor| &floor.level);
        let level = &mut floor.level;
        if let Some(below) = below {
            for (x, y) in cells_of(level) {
                if !shape.contains(x, y) || is_gap(below.cell(x, y)) {
                    make_hole(level, x, y);
                }
            }
        }
        let eaten_to = if index == 0 { Cell::Ground } else { Cell::Hole };
        let left = ruin.saturating_sub(index as u8);
        let mut rng = Seed::new(seed.at(&[index as i64])).rng();
        nibble(level, &mut rng, left, eaten_to, &protected);
        open_lone_doors(level);
        if let Some(below) = below {
            for (x, y) in cells_of(level) {
                if level.is_walkable(x, y) && !protected(x, y) && !supported(below, x, y) {
                    make_hole(level, x, y);
                }
            }
        }
        let connected = stairs_connect(&floor);
        kept.push(floor);
        if !connected {
            let top = kept.last_mut().expect("just added");
            top.stairs.retain(|s| s.direction != StairsDirection::Up);
            break;
        }
    }
    uncover_ground(&mut kept);
    kept
}

/// Ground is only covered where the floor above is still there.
fn uncover_ground(floors: &mut [SiteLevel]) {
    let Some((ground, above)) = floors.split_first_mut() else {
        return;
    };
    let above = above.first();
    for (x, y) in cells_of(&ground.level) {
        if ground.level.cell(x, y) == Cell::CoveredGround
            && above.is_none_or(|floor| floor.level.cell(x, y) == Cell::Hole)
        {
            ground.level.set(x, y, Cell::Ground);
        }
    }
}

/// Turns a cell into a gap in the floor.
fn make_hole(level: &mut Level, x: i64, y: i64) {
    level.set(x, y, Cell::Hole);
    level.set_room(x, y, None);
}

/// Whether the floor below holds up a floor at `(x, y)`: it isn't over a gap,
/// and a wall (or a door in one) is near enough to carry it.
fn supported(below: &Level, x: i64, y: i64) -> bool {
    !is_gap(below.cell(x, y))
        && (-MAX_SPAN..=MAX_SPAN).any(|dy| {
            (-MAX_SPAN..=MAX_SPAN).any(|dx| {
                let (wx, wy) = (x + dx, y + dy);
                let cell = below.cell(wx, wy);
                below.contains(wx, wy) && (cell.is_wall() || cell.is_opening())
            })
        })
}

/// Eats the level's edges, and holes started in its middle, away to `eaten_to`.
/// `ruin` is the ruin left on this floor.
fn nibble(
    level: &mut Level,
    rng: &mut Rng,
    ruin: u8,
    eaten_to: Cell,
    protected: &impl Fn(i64, i64) -> bool,
) {
    let can_eat = |level: &Level, x: i64, y: i64| {
        level.contains(x, y) && !is_gap(level.cell(x, y)) && !protected(x, y)
    };
    let mut percent = u32::from(ruin.min(MAX_RUIN)) * PERCENT_PER_RUIN;
    if percent == 0 {
        return;
    }
    let beside_gap = |x: i64, y: i64| {
        STEPS
            .iter()
            .any(|(dx, dy)| level.contains(x + dx, y + dy) && is_gap(level.cell(x + dx, y + dy)))
    };
    let mut next: BTreeSet<(i64, i64)> = cells_of(level)
        .map(|(x, y)| (y, x))
        .filter(|&(y, x)| can_eat(level, x, y) && beside_gap(x, y))
        .collect();
    let mut started = start_holes(level, rng, ruin, protected);
    while percent > 0 {
        let mut eaten = std::mem::take(&mut started);
        for (y, x) in next {
            if chance(rng, percent) && !eaten.contains(&(y, x)) {
                eaten.push((y, x));
            }
        }
        for &(y, x) in &eaten {
            level.set(x, y, eaten_to);
            level.set_room(x, y, None);
        }
        next = eaten
            .iter()
            .flat_map(|&(y, x)| STEPS.map(|(dx, dy)| (y + dy, x + dx)))
            .filter(|&(y, x)| can_eat(level, x, y))
            .collect();
        percent -= PERCENT_PER_RUIN;
    }
}

/// Picks where holes start in the middle of a floor, as `(y, x)`: on floor, so
/// that what is eaten from them on the ground floor can be walked to.
fn start_holes(
    level: &Level,
    rng: &mut Rng,
    ruin: u8,
    protected: &impl Fn(i64, i64) -> bool,
) -> Vec<(i64, i64)> {
    let Some(per_hole) = floor_cells_per_hole(ruin) else {
        return Vec::new();
    };
    let mut floor: Vec<(i64, i64)> = cells_of(level)
        .filter(|&(x, y)| level.cell(x, y) == Cell::Floor && !protected(x, y))
        .map(|(x, y)| (y, x))
        .collect();
    // Part of a hole's share of the floor makes one that often.
    let area = floor.len() as i64;
    let extra = grid::roll(rng, 0, per_hole - 1) < area % per_hole;
    let holes = area / per_hole + i64::from(extra);
    let mut started = Vec::new();
    for _ in 0..holes {
        if floor.is_empty() {
            break;
        }
        started.push(floor.swap_remove(rng.range_u64(0, floor.len() as u64) as usize));
    }
    started
}

/// A door with no wall left beside it is only floor.
fn open_lone_doors(level: &mut Level) {
    for (x, y) in cells_of(level) {
        let stands = |&(dx, dy): &(i64, i64)| {
            let cell = level.cell(x + dx, y + dy);
            level.contains(x + dx, y + dy) && (cell.is_wall() || cell.is_opening())
        };
        if level.cell(x, y) == Cell::Door && !STEPS.iter().any(stands) {
            level.set(x, y, Cell::Floor);
        }
    }
}

/// Whether a way you arrive on a floor can still be walked to some stairs up.
fn stairs_connect(floor: &SiteLevel) -> bool {
    let at = |direction| {
        floor
            .stairs
            .iter()
            .filter(move |s| s.direction == direction)
            .map(|s| (s.x, s.y))
    };
    let ups: Vec<_> = at(StairsDirection::Up).collect();
    // (A basement's stairs are no way in: come by the entrance.)
    let mut ways_in: Vec<_> = at(StairsDirection::Down)
        .filter(|_| floor.entrance.is_none())
        .collect();
    ways_in.extend(floor.entrance.filter(|_| ways_in.is_empty()));
    if ups.is_empty() || ways_in.is_empty() {
        return true;
    }
    let mut reach = Reach::new(&floor.level);
    reach.spread(&floor.level, &ways_in);
    ups.iter().any(|&(x, y)| reach.is(x, y))
}

#[cfg(test)]
mod tests;
