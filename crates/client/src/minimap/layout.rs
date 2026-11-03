//! Where the minimap and full map go and how big they are, and where the mouse is over the full map.

use bevy::prelude::*;

use super::{
    CLOSE_REACH, FULL_MAP_MARGIN, FULL_MAP_OF_WINDOW, MINIMAP_BASE_HEIGHT, MINIMAP_OF_HEIGHT,
    NOTCH, Room, TITLE_HEIGHT,
};
use crate::widgets::CLOSE_PIXELS;

/// The minimap's round part across, in the interface's pixels.
pub(super) fn minimap_across(window: &Room) -> f32 {
    // Never nothing: a minimised window has no height, and the scale of the
    // maps is worked out from this.
    (window.height() * MINIMAP_OF_HEIGHT).round().max(1.0)
}

/// How much bigger than given the sizes round the minimap are in this window.
pub(super) fn minimap_trim(window: &Room) -> f32 {
    minimap_across(window) / (MINIMAP_BASE_HEIGHT * MINIMAP_OF_HEIGHT)
}

/// The picture holding it, with the suns and moons round it (more than the
/// notch needs).
pub(super) fn minimap_picture_size(window: &Room) -> f32 {
    minimap_across(window) + 2.0 * (super::sky::BAND.max(NOTCH) * minimap_trim(window) + 1.0)
}

/// The full map's parchment, across, in logical pixels.
pub(super) fn full_map_side(window: &Room) -> f32 {
    (window.width().min(window.height()) * FULL_MAP_OF_WINDOW).round()
}

/// The part of it the map is drawn on, in logical pixels.
pub(super) fn full_map_area(window: &Room) -> Vec2 {
    let side = full_map_side(window);
    Vec2::new(
        side - 2.0 * FULL_MAP_MARGIN,
        side - 2.0 * FULL_MAP_MARGIN - TITLE_HEIGHT,
    )
    .max(Vec2::ONE)
}

/// The top-left corner of the full map's parchment, in logical pixels.
pub(super) fn full_map_corner(window: &Room) -> Vec2 {
    let side = full_map_side(window);
    Vec2::new(window.width() - side, window.height() - side) / 2.0
}

/// The top-left corner of the full map's picture, in logical pixels.
pub(super) fn full_map_picture_corner(window: &Room) -> Vec2 {
    full_map_corner(window) + Vec2::new(FULL_MAP_MARGIN, FULL_MAP_MARGIN + TITLE_HEIGHT)
}

/// Where the mouse is over the full map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OverFullMap {
    Outside,
    CloseButton,
    Parchment,
}

pub(super) fn over_full_map(window: &Room, at: Vec2) -> OverFullMap {
    let side = full_map_side(window);
    let at = at - full_map_corner(window);
    let inside = |from: f32, to: f32, at: f32| (from..to).contains(&at);
    if !inside(0.0, side, at.x) || !inside(0.0, side, at.y) {
        OverFullMap::Outside
    } else if inside(
        side - FULL_MAP_MARGIN - CLOSE_PIXELS - CLOSE_REACH,
        side,
        at.x,
    ) && inside(0.0, FULL_MAP_MARGIN + CLOSE_PIXELS + CLOSE_REACH, at.y)
    {
        OverFullMap::CloseButton
    } else {
        OverFullMap::Parchment
    }
}
