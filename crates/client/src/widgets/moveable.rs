//! Panels that fit their contents and can be moved by their title bars, and
//! the taking of clicks and touches for widgets.

// Split into files (`cargo xtask lint` keeps files small).
mod drag;
mod place;
mod spawn;

pub(super) use drag::{Dragging, take_presses};
pub(super) use place::place_panels;
pub(super) use spawn::{MoveablePanel, PanelPart};
pub use spawn::{PanelPlace, spawn_grip_panel, spawn_moveable_panel, stops_presses};

/// Above the maps, below full-screen layers; a panel clicked on goes above the
/// others.
const FIRST_PANEL_Z: i32 = 30;

const PADDING: f32 = 8.0;
/// Above the title bar: less than at the sides, as the bar is taller than the
/// stripes in it, so the room above them matches the room beside them.
const TOP_PADDING: f32 = 4.0;
const CONTENT_GAP: f32 = 4.0;
const TITLE_GAP: f32 = 8.0;
/// The stripes either side of the title, which only panels that can be moved have:
/// how many, how thick, the gap between, and the least each side may shrink to.
const STRIPES: usize = 3;
const STRIPE_THICKNESS: f32 = 2.0;
const STRIPE_GAP: f32 = 4.0;
const MIN_STRIPES_WIDTH: f32 = 16.0;
/// A grip panel's lines (`spawn_grip_panel`): thinner and closer together than
/// a title bar's, with this much room above and below them, which grips too.
const GRIP_THICKNESS: f32 = 1.0;
const GRIP_GAP: f32 = 2.0;
const GRIP_PADDING: f32 = 3.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;

#[cfg(test)]
mod tests;
