//! The shapes of a bag on the ground and of its sparkle, without Bevy so they can be
//! tested alone: a canvas sack turned about its middle, tied near the top, as full as
//! its `Load` looks. Everything stands on `y = 0`, in 3D units.

// Split into files (`cargo xtask lint` keeps files small).
mod model;
mod outlines;
mod shape;
mod size;

pub(super) use model::Model;
pub(super) use outlines::{hover_outline, outline, star};
pub(super) use shape::bag;
pub(super) use size::size_of;
use size::{TIED_AT, on_curve, profile};

/// How many ways round the bag goes, and up it.
const SEGMENTS: usize = 24;
const RINGS: usize = 22;
/// How many ways round the rope's own thickness goes, and along it.
const ROPE_SIDES: usize = 6;
const ROPE_SEGMENTS: usize = 16;
/// How far the outline stands out from a bag, always and while the pointer is on it.
const OUTLINE_WIDTH: f32 = 0.4;
const HOVER_OUTLINE_WIDTH: f32 = 0.9;
/// How many times the weave's picture goes round and up a bag.
const WEAVE_ROUND: f32 = 4.0;
const WEAVE_UP: f32 = 3.0;
const CANVAS: [f32; 3] = [0.74, 0.63, 0.45];
const ROPE: [f32; 3] = [0.3, 0.2, 0.11];
/// How dark the folds go, and memory's shade.
const FOLD_SHADE: f32 = 0.72;
const MEMORY_SHADE: f32 = 0.22;

/// How many different bags of each look there are.
pub(super) const VARIETIES: u8 = 3;

#[cfg(test)]
mod tests;
