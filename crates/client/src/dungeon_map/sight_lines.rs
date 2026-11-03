//! The real edge of what is in sight, for drawing only: a polygon round each
//! viewer, cut by the same thin walls as `shared::dungeon`'s sight, drawn into
//! a mask the floors' shader reads, so the edge falls part way across a cell.
//! The game's own sight stays on the cell grid (`Sight`).
//!
//! The polygon is found by casting rays at the corners of walls, and it uses
//! floats, which is fine as nothing but the picture depends on it.

// Split into files (`cargo xtask lint` keeps files small).
mod area;
mod cells;
mod fan;
mod mask;
mod material;
mod outlines;
mod wall_map;

use area::{Area, view_area};
pub use cells::add_cells;
use cells::{cross_cells, touch_cells};
use fan::{add_fan, fan_mesh};
pub use mask::{MaskCamera, SightLines, spawn_mask};
pub use material::{MaskSide, SightMask, SurfaceMaterial};
pub use outlines::{OutlineMesh, place_mask, set_mask_active, update_outlines};
pub use wall_map::{Vec2d, WallMap};

/// The mask's pixels each way, however much of the level it covers.
const MASK_PIXELS: u32 = 2048;

/// The layer only the mask's camera draws.
const MASK_LAYER: usize = 1;

/// A ray is cast this far (radians) to each side of a wall's corner, so one
/// passes it and the other stops on it.
const BESIDE_CORNER: f64 = 1.0e-5;

/// Rays also go out this many ways, for open space with no corners in it.
const EVEN_RAYS: usize = 72;

/// The mask covers this much more than the view each way, so it needn't move
/// whenever the camera does.
const MASK_SPARE: f32 = 0.25;

/// A new mask area is worked out when the view has shrunk to this share of it.
const MASK_TOO_BIG: f32 = 0.4;

#[cfg(test)]
mod tests;
