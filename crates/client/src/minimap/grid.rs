//! The map of a site level: remembered walkable cells as squares coloured by
//! their ground, with doors and shop signs, and hatching in the wall cells
//! along what they touch. Everything else is left as bare parchment.

// Split into files (`cargo xtask lint` keeps files small).
mod colour;
mod looks;
mod paint;

pub use colour::colour_at;
use colour::ground_colour;
pub use looks::{CellLook, Looks, Sides, look_of};
pub use paint::draw;

const WHITE: [u8; 3] = [255, 255, 255];
const LINE: [u8; 3] = [200, 200, 200];
const HATCH: [u8; 3] = [70, 55, 40];
const DOOR: [u8; 3] = [150, 95, 45];
const HOUSE_DOOR: [u8; 3] = [175, 60, 45];
const HOLE: [u8; 3] = [0, 0, 0];

/// How far hatching reaches into a wall from its edge, in cells, and its
/// strokes per cell.
const HATCH_REACH: f64 = 0.25;
const HATCH_STROKES: f64 = 3.0;
/// How much of a stroke's width is drawn.
const HATCH_SOLID: f64 = 0.4;

/// How far along each side a corner is cut away, in cells.
const CORNER_CUT: f64 = 0.4;

/// A door's rectangle: half its length along the wall, half its width across,
/// and the half width of the line it covers, all in cells.
const DOOR_HALF_LENGTH: f64 = 0.28;
const DOOR_HALF_WIDTH: f64 = 0.16;
const DOOR_LINE: f64 = 0.05;
/// How thick the rectangle's outline is.
const DOOR_OUTLINE: f64 = 0.07;
/// A house door is a diamond on its line, this far from its middle each way.
const HOUSE_DOOR_REACH: f64 = 0.28;

#[cfg(test)]
mod tests;
