//! Draws what only towns have, all from one picture (`assets/town_atlas.png`,
//! made by `tools/make_town_atlas.py`): roads, gardens with hedges, and solid
//! buildings with roofs.
//!
//! A building's walls stand down the middle of its outermost cells, as in
//! `walls`, but it is drawn as a whole: walls of different buildings don't
//! join, so a strip of ground shows between them. Its roof slopes up from the
//! walls across those cells, fading to black, and is flat and black inside.
//! A building in the way of a viewer goes half see-through.

use shared::dungeon::{CELL_UNITS, WALL_HALF_UNITS};

use super::{CELL_SIZE, WALL_HEIGHT};

// Split into files (`cargo xtask lint` keeps files small).
mod atlas;
mod building_cell;
mod buildings;
mod door_window;
mod emit;
mod faces;
mod flat;
mod hedge;
mod hedges;
mod opening;
mod roof;

pub(super) use atlas::atlas_uv;
pub use atlas::door_picture;
use atlas::{flat_uv, kind_index, road_tile, roof_tile, wall_tile};
use building_cell::build_cell;
pub use buildings::build_buildings;
use door_window::build_door_or_window;
pub(super) use door_window::window_faces;
pub(super) use emit::{cell_centre, emit};
use emit::{emit_as_given, light_at};
use faces::{Piece, face_layer, side_shade, strip, upright, wall_face};
pub use flat::build_flat;
pub(super) use flat::{flat_layer, flat_patch, layer_colour};
use hedge::Hedge;
pub use hedges::build_hedges;
use opening::Opening;
use roof::{Roof, roof_centre};

/// Tiles across and down the atlas.
const COLUMNS: f32 = 8.0;
const ROWS: f32 = 6.0;
/// Pixels across a tile of the atlas.
const ATLAS_TILE_PIXELS: u32 = 32;

/// Where things are in the atlas: the order `make_town_atlas.py` puts them.
const GARDEN_TILE: usize = 6;
pub(super) const HEDGE_TILE: usize = 7;
const FIRST_WALL_TILE: usize = 8;
const FIRST_ROOF_TILE: usize = 13;
const WINDOW_TILE: usize = 18;
const LIT_WINDOW_TILE: usize = 19;
const FIRST_DOOR_TILE: usize = 20;
pub(super) const PLOUGHED_TILE: usize = 43;
pub(super) const GRASS_TILE: usize = 44;
pub(super) const FENCE_TILE: usize = 45;

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// How far a wall stands from the centre of its cell.
const WALL_OUT: f32 = WALL_HALF_UNITS as f32 / CELL_UNITS as f32 * CELL_SIZE;

/// How high a roof rises above the walls, over how far in from them: the rest
/// of the outermost cells, so what is inside those is flat.
const ROOF_RISE: f32 = CELL_SIZE * 0.5;
const ROOF_RAMP: f32 = HALF_CELL + WALL_OUT;

/// Shading, since it is all unlit: faces running north-south are a little
/// darker, which shows corners.
const SIDEWAYS_SHADE: f32 = 0.85;

/// A window's half width, and its bottom and top.
const WINDOW_HALF: f32 = CELL_SIZE * 0.25;
const WINDOW_SILL: f32 = WALL_HEIGHT * 0.4;
const WINDOW_TOP: f32 = WALL_HEIGHT * 0.8;

/// A hedge's distance in from the garden's edge, its thickness and height.
const HEDGE_INSET: f32 = CELL_SIZE * 0.2;
const HEDGE_THICK: f32 = CELL_SIZE * 0.2;
const HEDGE_HEIGHT: f32 = CELL_SIZE * 0.2;

/// The hedge's sides are darker than its top.
const HEDGE_SIDE_SHADE: f32 = 0.7;

/// An arch's half width (to the inside of its posts) and height.
const ARCH_HALF: f32 = CELL_SIZE * 0.25;
const ARCH_HEIGHT: f32 = CELL_SIZE * 1.5;

/// How likely a stretch of hedge at least `ARCH_RUN` long has an arch, in
/// percent.
const ARCH_PERCENT: u64 = 50;
const ARCH_RUN: i64 = 3;

#[cfg(test)]
mod tests;
