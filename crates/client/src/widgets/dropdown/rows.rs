//! Which options show, which row is first, and where the list goes.

use bevy::prelude::*;

use super::super::filter_list::matches_filter;
use super::{GAP, ROWS};

/// The options matching what has been typed, by index.
pub(super) fn on_show(options: &[String], filter: &str) -> Vec<usize> {
    (0..options.len())
        .filter(|&at| matches_filter(&options[at], filter))
        .collect()
}

/// The first row to draw so that `highlight` shows, from `first`, among `count`.
pub(super) fn window_start(first: usize, highlight: Option<usize>, count: usize) -> usize {
    let most = count.saturating_sub(ROWS);
    let first = match highlight {
        Some(at) if at < first => at,
        Some(at) if at >= first + ROWS => at + 1 - ROWS,
        _ => first,
    };
    first.min(most)
}

/// Where a list goes under a button: on the screen, or above it if there is no
/// room below.
pub(super) fn list_corner(button: Rect, height: f32, width: f32, window: Vec2) -> Vec2 {
    let below = button.max.y + GAP;
    let y = if below + height <= window.y {
        below
    } else {
        (button.min.y - GAP - height).max(0.0)
    };
    Vec2::new(button.min.x.min(window.x - width).max(0.0), y)
}
