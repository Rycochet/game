//! How many rows and columns fit.

use super::{COLUMN_GAP, COLUMN_WIDTH, FILTER_HEIGHT, GAP, MARGIN, ROW, TITLE_ROW};
use crate::widgets::{PANEL_EDGE, layer_share};

/// How many entries fit in a column of this layer, in a window this tall (in the
/// interface's pixels).
pub(super) fn rows_in(window_height: f32, layer: usize) -> usize {
    let share = layer_share(layer as i32);
    let inside = window_height * share - 2.0 * (PANEL_EDGE + MARGIN);
    let room = inside - TITLE_ROW - FILTER_HEIGHT - 2.0 * GAP * 3.0;
    (((room + GAP) / (ROW + GAP)).floor() as usize).max(1)
}

/// How many columns of this layer fit, in a window this wide (in the interface's
/// pixels).
pub(super) fn columns_in(window_width: f32, layer: usize) -> usize {
    let share = layer_share(layer as i32);
    let room = window_width * share - 2.0 * (PANEL_EDGE + MARGIN);
    (((room + COLUMN_GAP) / (COLUMN_WIDTH + COLUMN_GAP)).floor() as usize).max(1)
}
