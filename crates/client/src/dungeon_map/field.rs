//! What only farms have: fields, what grows in them, and their fences and gates.
//!
//! What stage a field is at is purely for looks, so it is chosen here. For now
//! that is from the number the server sends with it; the time of year is to
//! come into it. Crops are thin upright pictures, standing in a `#` in each cell,
//! with their own mesh, which only changes with what is in sight.

use shared::rng::Seed;

use super::CELL_SIZE;

// Split into files (`cargo xtask lint` keeps files small).
mod crops;
mod fence;
mod ground;
mod stage;
mod wood;

pub use crops::build_crops;
pub use fence::build_fence;
pub use ground::build_field;
#[cfg(test)]
use ground::ruined_tile;
pub use stage::{CROP_STAGES, Stage};
use wood::{post, rails, wood};

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// The picture of crops as they grow: one tile per stage, side by side.
pub const CROPS_PICTURE: &str = "field_crops.png";
const CROP_TILES: f32 = 3.0;

/// Pixels across a cell, for crops' heights.
const CELL_PIXELS: f32 = 32.0;

/// Where a cell's crops stand, each way, as a share of the cell from its centre.
const CROP_ROWS: [f32; 2] = [-0.25, 0.25];

/// A fence: its height, its rails' heights, how thick posts and rails are.
const FENCE_HEIGHT: f32 = CELL_SIZE * 0.6;
const RAIL_HEIGHTS: [f32; 2] = [CELL_SIZE * 0.25, CELL_SIZE * 0.5];
const POST_HALF: f32 = CELL_SIZE * 0.05;
const RAIL_HALF: f32 = CELL_SIZE * 0.025;

/// How far a gate's posts stand from the middle of its cell. Its leaf is as long
/// as the gap between them, and has a bar this thick upright at its open end.
const GATE_POST: f32 = CELL_SIZE * 0.42;
const GATE_END_HALF: f32 = CELL_SIZE * 0.035;

/// The sides of a fence's wood are darker than its top.
const WOOD_SIDE_SHADE: f32 = 0.8;

/// For choosing what each cell of a ruined field has gone back to.
const RUINED_TILES: Seed = Seed::new(0x6669_656c_6473);

#[cfg(test)]
mod tests;
