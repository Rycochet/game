//! Dungeons and buildings: a grid of floor and wall cells, and what can be
//! seen (or lit) from a cell.
//!
//! A wall takes up only a thin slice of its cell, running through the middle
//! towards the walls next to it (`WallShape`). It blocks movement in the whole
//! cell, but sight and light only where it really stands, so they pass beside
//! a column or the end of a wall. Sight follows straight lines in exact
//! integers, so the client and the server always agree on what a character can
//! see. Levels are made by the generators in `crate::site`.

// Split into files (`cargo xtask lint` keeps files small).
mod bars;
mod cell;
mod doors;
mod level;
mod line;
mod sight;
mod wall_shape;

pub use bars::WallBar;
use bars::bars_of;
pub(crate) use bars::line_hits_box;
pub use cell::Cell;
pub use level::{Level, RoomId};
use line::clear_line;
pub use line::{line_is_clear, spots_are_clear};
pub use sight::Sight;
pub use wall_shape::WallShape;

/// Cells measured in units for sight's exact arithmetic: a cell is this wide,
/// with its centre on a multiple of it.
pub const CELL_UNITS: i64 = 40;

/// Half a wall's thickness, in units: it covers 45% to 55% of its cell.
pub const WALL_HALF_UNITS: i64 = 2;

/// How far past the centre a wall joined to only one other goes, in units: it
/// stops 60% of the way across its cell.
pub const WALL_END_UNITS: i64 = 4;

/// Half a cell, in units.
const HALF_CELL: i64 = CELL_UNITS / 2;

#[cfg(test)]
mod tests;
