//! The terrain around the player, drawn as a square window lined up with the
//! camera, with flat faces down its edges.
//!
//! Two stages, so stepping and turning the view stay cheap:
//! - `TerrainGrid::fill` works out heights, normals and colours for the grid
//!   points the window covers. Points already worked out are kept, so a step
//!   or a turn only adds the newly covered ones.
//! - `build_window` and `build_edges` then cut the window out of that grid:
//!   whole triangles inside it are kept, ones crossing its border are clipped
//!   to it, so its outline is exact.

// Split into files (`cargo xtask lint` keeps files small).
mod canopy;
mod edges;
mod grid;
mod height_cache;
mod rivers;
mod roads;
mod square;
mod window;

use canopy::{CANOPY_SHADING, linear};
pub use canopy::{canopy_lift, shown_height_at};
pub use edges::build_edges;
pub use grid::TerrainGrid;
pub use height_cache::HeightCache;
pub use rivers::build_rivers;
use rivers::clip;
pub use roads::build_roads;
use square::Corner;
pub use square::ViewSquare;
pub use window::build_window;

use super::MAX_HALF;

/// The furthest grid units from the player `TerrainGrid` may need. The
/// window's corners are at most `MAX_HALF × √2` (about 283) away, so this
/// covers it at any angle, with a little to spare for `ViewSquare::reach`.
pub const REACH: i64 = (MAX_HALF * std::f32::consts::SQRT_2) as i64 + 5;

/// Side length of `TerrainGrid`'s wrapping store: more than the `2 × REACH`
/// it must hold, so moving never evicts points still in view.
const STORE_SIDE: i64 = 640;

/// Bottom of the edge faces, as a fraction of the water level (0 = height 0,
/// as in the prototype; raised to make the faces shorter).
const FRONT_BASE: f32 = 0.3;

/// Undoes the sun's dimming of flat frozen water (lit flat ice measured at
/// ~71% of its colour in linear light).
const ICE_BOOST: f32 = 1.41;

/// Cache side length: larger than `TerrainGrid`'s reach (plus a 1-unit
/// border for normals) so moving never evicts samples still in view.
const CACHE_SIDE: i64 = 640;

// Checked when compiling: the wrapping stores must be big enough.
const _: () = assert!(STORE_SIDE > 2 * REACH + 1);
const _: () = assert!(CACHE_SIDE > 2 * REACH + 3);

const BANK_ALPHA: f32 = 0.6;

const CANOPY_LUSH: f32 = 1.5;
const CANOPY_FULL: f32 = 0.7;

#[cfg(test)]
mod tests;
