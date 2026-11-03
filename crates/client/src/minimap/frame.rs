//! What is drawn over a map's contents: the round minimap's ridged border and
//! north notch, the full map's compass in the same style, the player's dot,
//! and the arrow pointing to a dot off the map.

// Split into files (`cargo xtask lint` keeps files small).
mod compass;
mod player_mark;
mod round;
mod secret_door_icon;
mod stairs_icon;

pub use compass::compass;
pub use player_mark::{arrow, arrow_place, floor_hint, player_dot};
use round::ridged;
pub use round::{RoundFrame, line_distance};
pub use secret_door_icon::secret_door_icon;
pub use stairs_icon::stairs_icon;

const WHITE: [u8; 3] = [255, 255, 255];
const BLACK: [u8; 3] = [0, 0, 0];

/// The ridged border's middle colour, and how far its sides go lighter and
/// darker from it.
const BORDER: [f32; 3] = [150.0, 115.0, 70.0];
const RIDGE_STRENGTH: f32 = 0.55;

/// The direction light comes from, for the ridge: the top left of the screen,
/// whichever way the map is turned.
const LIGHT: (f32, f32) = (
    -std::f32::consts::FRAC_1_SQRT_2,
    -std::f32::consts::FRAC_1_SQRT_2,
);

/// A stairs icon's steps and colour.
const STAIRS_STEPS: u32 = 4;
const STAIRS_INK: [u8; 3] = [70, 55, 40];

/// A secret door's "S": its colour, the width of its stroke against its height,
/// and the gap at each end of its curves, in radians.
const SECRET_DOOR_INK: [u8; 3] = [70, 55, 40];
const SECRET_DOOR_STROKE: f32 = 0.2;
const SECRET_DOOR_GAP: f32 = 0.35;

/// The arrow beside the player's mark on another floor, against its height: its
/// shaft's width, its head's width and height, its black edge and its gap from
/// the mark (a little more than the edge, so they nearly touch).
const HINT_SHAFT: f32 = 0.34;
const HINT_HEAD_WIDTH: f32 = 0.95;
const HINT_HEAD_HEIGHT: f32 = 0.5;
const HINT_EDGE: f32 = 0.1;
const HINT_GAP: f32 = 0.13;

/// How wide the notch's base is, against how long it is, and how far along the
/// border it spreads as it grows out of it, against the same.
const NOTCH_BASE: f32 = 0.6;
const NOTCH_SPREAD: f32 = 0.5;

#[cfg(test)]
mod tests;
