//! The terrain around the player, drawn as a square window lined up with the
//! camera, with flat faces down its edges.
//!
//! Two stages, so stepping and turning the view stay cheap:
//! - `TerrainGrid::fill` works out heights, normals and colours for the grid
//!   points the window covers. Points already worked out are kept, so a step
//!   or a turn only adds the newly covered ones.
//! - `build_chunk` makes the meshes of square chunks of that grid, each built
//!   once while in reach (`chunk.rs`); the terrain's shaders cut the window
//!   out of them, so its outline is exact. `build_edges` cuts the edge faces
//!   to the window on the CPU, as do the rivers and roads.

// Split into files (`cargo xtask lint` keeps files small).
mod canopy;
mod chunk;
mod edges;
mod grid;
mod height_cache;
mod rivers;
mod roads;
mod square;

use canopy::{CANOPY_SHADING, linear};
pub use canopy::{canopy_lift, shown_height_at};
pub use chunk::{CHUNK, GridArea, build_chunk, chunks_area, chunks_needed};
pub use edges::build_edges;
pub use grid::TerrainGrid;
pub use height_cache::HeightCache;
pub use rivers::build_rivers;
pub use roads::build_roads;
use square::Corner;
pub use square::ViewSquare;

use super::MAX_HALF;

/// The furthest grid units from the player `TerrainGrid` may need. The
/// window's corners are at most `MAX_HALF × √2` (about 283) away, so this
/// covers it at any angle, with a little to spare for `ViewSquare::reach`.
pub const REACH: i64 = (MAX_HALF * std::f32::consts::SQRT_2) as i64 + 5;

/// Side length of `TerrainGrid`'s wrapping store: more than the
/// `2 × (REACH + CHUNK)` it must hold (whole chunks round the window), so
/// moving never evicts points still in view.
const STORE_SIDE: i64 = 672;

/// How far the edge faces reach below the lowest drawn point in the window, in
/// the starting view: measured from the ground in view, not the sea, so
/// highlands far above it still stand on a frame of the same depth.
const FRAME_DEPTH: f32 = 25.0;

/// Undoes the sun's dimming of flat frozen water (lit flat ice measured at
/// ~71% of its colour in linear light).
const ICE_BOOST: f32 = 1.41;

/// Cache side length: larger than `TerrainGrid`'s reach (plus a 1-unit
/// border for normals) so moving never evicts samples still in view.
const CACHE_SIDE: i64 = 672;

// Checked when compiling: the wrapping stores must be big enough.
const _: () = assert!(STORE_SIDE > 2 * (REACH + CHUNK) + 1);
const _: () = assert!(CACHE_SIDE > 2 * (REACH + CHUNK) + 3);

const BANK_ALPHA: f32 = 0.6;

const CANOPY_LUSH: f32 = 1.5;
const CANOPY_FULL: f32 = 0.7;

#[cfg(test)]
mod tests;
