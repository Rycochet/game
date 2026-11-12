//! What each cell of a level puts in a creature's way (owner, 2026-11-10:
//! whole wall cells as now, plus thin blockers; playtest will tell).
//!
//! - Walls, windows (natural ones' columns leave gaps far too narrow for
//!   anyone) and closed doors fill their cell, less a notch at each corner
//!   with no wall beside it, so two walls touching at a corner leave a
//!   squeeze between them.
//! - An open (or broken) door leaves its frame: a post either side of the
//!   doorway, which is `2 × DOOR_HALF_GAP` wide, as drawn. Its leaf doesn't
//!   block (owner).
//! - A fence is a thin bar through its cell towards the fences and gates
//!   beside it, shaped as walls are; it can be jumped.
//! - A tree is its trunk, round, at its cell's middle.

use super::{Blocker, Shape};
use crate::dungeon::{CELL_UNITS, Cell, Level, WallShape};
use crate::spot::{CELL_SPAN, HALF_SPAN, Spot};

/// How far a wall's free corner is notched, each way, in units: a quarter of a
/// cell. Between two walls touching at a corner it leaves `CORNER_CUT / √2`
/// (42 units) either side of the middle: a Medium creature squeezes through,
/// a Large one can't.
pub const CORNER_CUT: i64 = 60;

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
    if fills(level, x, y) {
        out.extend(notched(level, x, y).map(solid));
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
        let [north, east, south, west] = shape.reach().map(|r| r * REACH_SCALE);
        let t = FENCE_HALF;
        let bars = [((-west, -t), (east, t)), ((-t, -north), (t, south))];
        out.extend(bars.map(|(low, high)| Blocker {
            shape: boxed(middle, low, high),
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

/// Whether the cell is filled: a wall, a window or a closed door.
fn fills(level: &Level, x: i64, y: i64) -> bool {
    level.is_wall(x, y) || level.cell(x, y) == Cell::Window || level.is_shut(x, y)
}

/// Whether a corner beside this cell stays square: walls, windows and
/// doorways run on into it.
fn holds_corner(level: &Level, x: i64, y: i64) -> bool {
    fills(level, x, y) || level.is_opening(x, y)
}

/// A filled cell as boxes: whole, or with its free corners notched.
fn notched(level: &Level, x: i64, y: i64) -> impl Iterator<Item = Shape> {
    let free = |dx, dy| !holds_corner(level, x + dx, y) && !holds_corner(level, x, y + dy);
    let cut = |is_free: bool| if is_free { CORNER_CUT } else { 0 };
    let (nw, ne, sw, se) = (free(-1, -1), free(1, -1), free(-1, 1), free(1, 1));
    let middle = Spot::middle_of((x, y));
    let (h, n) = (HALF_SPAN, CORNER_CUT);
    let whole = !(nw || ne || sw || se);
    let boxes = if whole {
        vec![((-h, -h), (h, h))]
    } else {
        vec![
            ((-h + n, -h), (h - n, h)),
            ((-h, -h + cut(nw)), (-h + n, h - cut(sw))),
            ((h - n, -h + cut(ne)), (h, h - cut(se))),
        ]
    };
    boxes
        .into_iter()
        .map(move |(low, high)| boxed(middle, low, high))
}

/// A box from offsets round `middle`.
fn boxed(middle: Spot, low: (i64, i64), high: (i64, i64)) -> Shape {
    Shape::Box {
        low: middle.offset(low.0, low.1),
        high: middle.offset(high.0, high.1),
    }
}
