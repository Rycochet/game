//! What each cell of a level puts in a creature's way: what is drawn there
//! (owner, 2026-11-14: walls block only where they are drawn).
//!
//! - Walls (building doors, blind windows and closed secret doors too) are
//!   thin bars through their cell, as `WallShape::reach` draws them: out to
//!   the walls, doors and windows they join, otherwise a little past the
//!   middle. Natural walls (crypts, caves, ice) are wider, as they are at the
//!   foot. Windows and closed doors are a bar the width of their cell, along
//!   their wall. None of these cells gives footing, so a creature's middle
//!   still stops at the edge of a wall's cell.
//! - An open (or broken) door leaves its frame: a post either side of the
//!   doorway, which is `2 × DOOR_HALF_GAP` wide, as drawn. Its leaf doesn't
//!   block (owner).
//! - A fence is a thin bar through its cell towards the fences and gates
//!   beside it, shaped as walls are; it can be jumped.
//! - A tree is its trunk, round, at its cell's middle.

use super::{Blocker, Shape};
use crate::dungeon::{CELL_UNITS, Cell, Level, WALL_HALF_UNITS, WallShape};
use crate::spot::{CELL_SPAN, HALF_SPAN, Spot};

/// Half a built wall's thickness, in units: 12, as drawn (45% to 55% of its
/// cell).
pub const WALL_HALF: i64 = WALL_HALF_UNITS * REACH_SCALE;

/// Half a natural wall's thickness at its foot, in units: 0.2 of a cell, as
/// `dungeon_map` draws it (its curve like a letter C is widest there).
pub const NATURAL_WALL_FOOT: i64 = 48;

/// Half a doorway's width, in units: 0.4 of a cell, as `dungeon_map` draws it,
/// so the doorway is 192 units (0.8 cell) between its frame's posts.
pub const DOOR_HALF_GAP: i64 = 96;

/// Half a fence's thickness, in units.
pub const FENCE_HALF: i64 = 12;

/// A tree trunk's radius, in units.
pub const TRUNK_RADIUS: i64 = 30;

/// Units here for one of `WallShape::reach`'s.
const REACH_SCALE: i64 = CELL_SPAN / CELL_UNITS;

/// Adds what cell `(x, y)` puts in the way to `out`.
pub fn cell_blockers(level: &Level, x: i64, y: i64, out: &mut Vec<Blocker>) {
    let middle = Spot::middle_of((x, y));
    let solid = |shape| Blocker { shape, fence: None };
    let cell = level.cell(x, y);
    let half = if level.natural_walls() {
        NATURAL_WALL_FOOT
    } else {
        WALL_HALF
    };
    if let Some(shape) = level.wall_shape(x, y) {
        // Ends and sides reach as far past the narrowest wall as it is wider.
        let reach = shape.reach().map(|r| {
            let r = r * REACH_SCALE;
            if r >= HALF_SPAN {
                r
            } else {
                r + half - WALL_HALF
            }
        });
        out.extend(bars(middle, reach, half).map(solid));
    } else if cell == Cell::Window || level.is_shut(x, y) {
        // Along its wall, the width of its cell.
        let (h, t) = (HALF_SPAN, half);
        let (low, high) = if level.opening_runs_east_west(x, y) {
            ((-h, -t), (h, t))
        } else {
            ((-t, -h), (t, h))
        };
        out.push(solid(boxed(middle, low, high)));
    } else if level.is_opening(x, y) {
        // A doorway: its frame's posts, along its wall.
        let (h, g) = (HALF_SPAN, DOOR_HALF_GAP);
        let posts = if level.opening_runs_east_west(x, y) {
            [((-h, -h), (-g, h)), ((g, -h), (h, h))]
        } else {
            [((-h, -h), (h, -g)), ((-h, g), (h, h))]
        };
        out.extend(posts.map(|(low, high)| solid(boxed(middle, low, high))));
    } else if cell == Cell::Fence {
        let joins = |x, y| matches!(level.cell(x, y), Cell::Fence | Cell::FieldGate);
        let shape = WallShape {
            north: joins(x, y - 1),
            east: joins(x + 1, y),
            south: joins(x, y + 1),
            west: joins(x - 1, y),
        };
        let reach = shape.reach().map(|r| r * REACH_SCALE);
        out.extend(bars(middle, reach, FENCE_HALF).map(|shape| Blocker {
            shape,
            fence: Some((x, y)),
        }));
    } else if cell == Cell::Tree {
        out.push(solid(Shape::Post {
            at: middle,
            radius: TRUNK_RADIUS,
        }));
    }
}

/// Everything in the way in the cells round a box of spots, `reach` units
/// beyond it each way.
pub fn blockers_near(level: &Level, low: Spot, high: Spot, reach: i64) -> Vec<Blocker> {
    let (min_x, min_y) = low.offset(-reach, -reach).cell();
    let (max_x, max_y) = high.offset(reach, reach).cell();
    let mut out = Vec::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            cell_blockers(level, x, y, &mut out);
        }
    }
    out
}

/// The two bars of a wall or fence, `half` thick each side of its middle line,
/// reaching `[north, east, south, west]` from `middle`.
fn bars(middle: Spot, [north, east, south, west]: [i64; 4], half: i64) -> [Shape; 2] {
    let t = half;
    [((-west, -t), (east, t)), ((-t, -north), (t, south))]
        .map(|(low, high)| boxed(middle, low, high))
}

/// A box from offsets round `middle`.
fn boxed(middle: Spot, low: (i64, i64), high: (i64, i64)) -> Shape {
    Shape::Box {
        low: middle.offset(low.0, low.1),
        high: middle.offset(high.0, high.1),
    }
}
