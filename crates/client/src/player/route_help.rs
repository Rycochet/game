//! Helpers for journeys: moving towards a point, enemies in the way, and the
//! cell a point is in.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::sheet::Feeling;

use super::Player;
use crate::common::{GodMode, MapPosition, Other};

/// `distance` along the straight line from one point to another, stopping
/// exactly on it.
pub(super) fn towards(from: DVec2, to: DVec2, distance: f64) -> DVec2 {
    if from.distance(to) <= distance {
        to
    } else {
        from + (to - from).normalize() * distance
    }
}

/// The cells of mobs that won't move aside for the player: none in god mode.
pub(super) fn enemy_cells(
    others: &Query<(&Other, &MapPosition), Without<Player>>,
    god: GodMode,
) -> Vec<(i64, i64)> {
    others
        .iter()
        .filter(|(other, _)| !god.0 && other.feeling == Feeling::Unfriendly)
        .map(|(_, at)| (at.x, at.y))
        .collect()
}

/// The grid point a position is nearest: its cell, in a dungeon.
pub(super) fn nearest_cell(position: DVec2) -> MapPosition {
    MapPosition {
        x: position.x.round() as i64,
        y: position.y.round() as i64,
    }
}
