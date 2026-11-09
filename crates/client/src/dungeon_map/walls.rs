//! Draws walls, doors and windows as thin 3D shapes, by the rules in
//! `shared::dungeon` (`WallShape`, how walls join).
//!
//! A wall cell is drawn in four quarters around its centre. Each quarter is
//! part of the open space round a corner of the cell, and has its own piece of
//! floor, up to two wall faces and a piece of the wall's top. A remembered
//! wall draws a quarter if an open cell known to the client shares that corner,
//! so a wall seen from one side doesn't vanish when the camera is behind it,
//! and the insides of solid rock stay dark. The floor stops at the wall's foot,
//! and beside natural walls the wall's picture fades out across it.

use shared::dungeon::{CELL_UNITS, WALL_HALF_UNITS};

use super::{CELL_SIZE, WALL_HEIGHT};

// Split into files (`cargo xtask lint` keeps files small).
mod build;
mod door_leaf;
mod faces;
mod floor;
mod layers;
mod opening;
mod opening_frame;
mod opening_wall;
mod patch;
mod pillars;
mod scene;
mod texture;
mod thickness;
mod tile;
mod under_liquid;
mod wall_face;
mod wall_quarter;

use build::Tile;
pub use build::{build_opening, build_wall};
pub use door_leaf::build_secret_door;
pub use layers::{Layer, Layers};
use opening::{Opening, RISING_SHADE};
use patch::{Patch, cuts};
pub use scene::Scene;
use texture::{face_uv, floor_near};
pub(super) use texture::{flat_uv, noise};
pub use thickness::half_thickness;
use thickness::{height_shade, heights, reach_at, units_to_size};

/// Half a cell.
const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// Half a wall's thickness at its narrowest.
const WALL_HALF: f32 = WALL_HALF_UNITS as f32 / CELL_UNITS as f32 * CELL_SIZE;

/// Half a natural wall's thickness at the floor and at its top, where the
/// curve like a letter C is widest.
const NATURAL_WALL_HALF: f32 = CELL_SIZE * 0.2;

/// How finely a natural wall is cut up, for its curve and rough surface: up
/// its height and (on faces long enough) along it.
const NATURAL_BANDS: usize = 6;
const NATURAL_SEGMENTS: usize = 3;

/// How far a natural wall's surface wanders in and out, and how much that
/// darkens or lightens it.
const ROUGHNESS: f32 = 0.45;
const ROUGH_SHADE: f32 = 0.12;

/// How far a natural wall's picture fades out across the floor from its foot:
/// no further than a wall end's distance from the edge of its cell, so it
/// never stops abruptly at the next cell. It is cut into steps this long, and
/// lifted this far off the floor to stay in front of it.
const BLEND_WIDTH: f32 = CELL_SIZE * 0.25;
const BLEND_STEP: f32 = BLEND_WIDTH / 3.0;
const BLEND_LIFT: f32 = 0.02;

/// Half the width of the hole for a door or window (with the frame round it,
/// the cell's width).
const OPENING_HALF: f32 = CELL_SIZE * 0.4;

/// A door's height, and where an arched door's curve starts.
const DOOR_TOP: f32 = WALL_HEIGHT * 0.78;
const ARCH_START: f32 = WALL_HEIGHT * 0.5;

/// The bottom and top of a window's hole: the frame below is taller than the
/// lintel above.
const WINDOW_SILL: f32 = WALL_HEIGHT * 0.42;
const WINDOW_TOP: f32 = WALL_HEIGHT * 0.82;

/// Natural windows are a line of thin columns, as wide as the gaps between
/// them, each this wide at its narrowest and at its foot.
const PILLARS: i64 = 5;
const PILLAR_SPACING: f32 = CELL_SIZE * 0.2;
const PILLAR_HALF: f32 = CELL_SIZE * 0.05;
const PILLAR_FOOT_HALF: f32 = CELL_SIZE * 0.08;

/// Shading, since the walls are unlit: faces running north-south are a little
/// darker than those running east-west, which shows corners; the tops of walls
/// are darker still. A natural wall is lighter low down, where it leans towards
/// the light, and darker where it overhangs.
const SIDEWAYS_SHADE: f32 = 0.85;
const TOP_SHADE: f32 = 0.55;
const FRAME_SHADE: f32 = 0.7;
const NATURAL_LOW_SHADE: f32 = 1.0;
const NATURAL_HIGH_SHADE: f32 = 0.6;

/// The quarters of a cell, as the directions they lie in (`x` east, `z` south).
const QUARTERS: [(f32, f32); 4] = [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)];

#[cfg(test)]
mod tests;
