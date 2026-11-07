//! How widgets answer the pointer: looks for being hovered and pressed, a ripple
//! from the point pressed, and the names of buttons after a short wait.

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod looks;
mod pointer;
mod show;

pub use looks::{ButtonStyle, ButtonTooltip};
pub(super) use looks::{Pointed, Ripple};
use looks::{Touch, look, ripple_shape};
pub use pointer::OverInterface;
pub(super) use pointer::{spread_ripples, track_pointer};
pub(super) use show::{show_button_looks, show_button_tooltips, show_edges};

const LIGHTER_WHEN_HOVERED: f32 = 0.06;
const DARKER_WHEN_PRESSED: f32 = 0.08;
/// How long a ripple takes to spread and fade, in seconds.
const RIPPLE_SECONDS: f32 = 0.35;
/// How long the pointer stays on a button before its name shows, in seconds.
const TOOLTIP_DELAY: f32 = 0.5;
const TOOLTIP_OFFSET: Vec2 = Vec2::new(16.0, 16.0);

#[cfg(test)]
mod tests;
