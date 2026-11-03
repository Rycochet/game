//! Draws water and lava as channels cut down into the floor.
//!
//! As with walls, a cell is drawn in four quarters around its centre. What a
//! quarter looks like depends only on the four cells round the grid corner it
//! touches, so neighbouring quarters always meet: a quarter of liquid with
//! floor to both sides is rounded; with liquid on one side it is half a cell
//! wide, running that way; with liquid on both it fills the quarter, except a
//! round bite of floor at the corner if the cell diagonally across is floor.
//!
//! Walls count as liquid here. Liquid runs up to the foot of a wall (over the
//! floor of the wall's cell, like liquid in its own quarters), and the wall
//! carries on down to the bed (`walls`).
//!
//! Depth (the owner's rule): a cell with floor on two opposite sides is half a
//! cell deep, and stays so where it meets another such cell, so a narrow
//! channel's bed is flat. Where it meets a wider liquid cell, or a wall, it
//! slopes to a full cell deep. Any other cell is a full cell deep all over.

use super::CELL_SIZE;

// Split into files (`cargo xtask lint` keeps files small).
mod bed;
mod near;
mod outline;
mod pieces;
mod quarter;
mod shade;

#[cfg(test)]
use bed::is_narrow;
use bed::{Bed, bed_at};
use near::{around, cell_light, wet};
pub use near::{outline_at, surfaces_near};
pub use outline::Outline;
use outline::{Point, Rings};
pub use quarter::{Quarter, build_cell};
use shade::bed_shade;
pub use shade::{depth_shade, opacity};

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// Half the width of a narrow channel, so also how far a shore stands back
/// from the edge of the cell.
pub const CHANNEL_HALF: f32 = CELL_SIZE / 4.0;

/// How far the surface of a liquid is below the floor.
const SURFACE_DROP: f32 = CELL_SIZE * 0.1;

/// How deep a narrow channel is in the middle of its cell, and how deep liquid
/// gets anywhere else.
const NARROW_DEPTH: f32 = CELL_SIZE * 0.5;
pub const FULL_DEPTH: f32 = CELL_SIZE;

/// Lengths a curved shore is cut into, in a quarter. Even, so that a corner of
/// the quarter is reached half way round.
const CURVE_PIECES: usize = 6;

/// How much darker the sides are at the floor, how much more they darken by a
/// full cell deep (evenly on the way down), and how much darker again the bed
/// is at any depth, so it stands out from them.
const RIM_DARKENING: f32 = 0.2;
const SIDE_DARKENING: f32 = 0.4;
const BED_DARKENING: f32 = 0.2;

#[cfg(test)]
mod tests;
