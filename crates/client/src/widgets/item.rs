//! How items look anywhere in the interface: cells with edges, grids to put them
//! in, and the name beside the pointer. What pressing or dropping them does stays
//! with the module showing them.

use bevy::prelude::*;

use super::TextSize;

// Split into files (`cargo xtask lint` keeps files small).
mod cells;
mod count;
mod look;
mod mixed;
mod name;
mod nodes;
mod picture;

pub use cells::{spawn_item_cells, spawn_item_grid};
pub use look::{
    Floating, ItemLook, ItemName, ItemTip, ItemTooltip, item_colour, item_edge, item_edge_width,
    item_label,
};
pub(super) use look::{NameBox, NameEdge, NameText};
pub(super) use name::{name_item_at_pointer, show_item_tooltip, spawn_name_box};
pub use nodes::{at_cell, covering, logical_rect, shown_at};
pub use picture::{item_picture, picture_in_square};
use picture::{picture_node, picture_turned};

/// A grid cell, across, in logical pixels.
pub const CELL: f32 = 32.0;
/// How many pixels a picture has for each cell it covers: twice what it is shown at,
/// for sharpness.
pub const PICTURE_PIXELS_PER_CELL: u32 = 64;
/// What is written on an item.
pub const ITEM_LABEL_FONT: TextSize = TextSize::Small;
/// Above moveable panels, below full-screen layers.
pub const HELD_ITEM_Z: i32 = 900_000;

const CELL_BORDER: f32 = 1.0;
/// How thick the metal edge of a set item's name is.
const NAME_EDGE: f32 = 3.0;
/// How far from the pointer an item's name goes.
const TOOLTIP_OFFSET: Vec2 = Vec2::new(16.0, 16.0);
/// How thick an edge showing quality is, to be seen beside the kind's colour.
const QUALITY_BORDER: f32 = 2.0;

#[cfg(test)]
mod tests;
