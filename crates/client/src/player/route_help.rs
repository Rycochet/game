//! Helpers for journeys: moving towards a point, enemies in the way, and the
//! cell a point is in.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::clearance::{Crowding, Other as Crowded};
use shared::sheet::Feeling;

use super::Player;
use crate::common::{GodMode, MapPosition, Other, SpotPosition};

/// `distance` along the straight line from one point to another, stopping
/// exactly on it.
pub(super) fn towards(from: DVec2, to: DVec2, distance: f64) -> DVec2 {
    if from.distance(to) <= distance {
        to
    } else {
        from + (to - from).normalize() * distance
    }
}

/// Everyone else on the map, where they are.
pub(super) type Others<'w, 's> =
    Query<'w, 's, (&'static Other, &'static MapPosition, &'static SpotPosition), Without<Player>>;

/// The cells of mobs that won't let the player past: none in god mode.
pub(super) fn enemy_cells(others: &Others, god: GodMode) -> Vec<(i64, i64)> {
    others
        .iter()
        .filter(|(other, ..)| !god.0 && other.feeling == Feeling::Unfriendly)
        .map(|(_, at, _)| (at.x, at.y))
        .collect()
}

/// Everyone in the player's way on a level, as the server counts them
/// (`shared::clearance::crowd`): enemies block, other players are walked
/// through, the rest are squeezed past. Nobody in god mode.
pub(super) fn crowd(others: &Others, god: GodMode) -> Vec<Crowded> {
    if god.0 {
        return Vec::new();
    }
    others
        .iter()
        .filter(|(other, ..)| other.feeling != Feeling::None)
        .map(|(other, _, spot)| Crowded {
            at: spot.0,
            size: other.kind.def().size,
            crowding: match other.feeling {
                Feeling::Unfriendly => Crowding::Block,
                _ => Crowding::Squeeze,
            },
        })
        .collect()
}

/// The grid point a position is nearest: its cell, in a dungeon.
pub(super) fn nearest_cell(position: DVec2) -> MapPosition {
    MapPosition {
        x: position.x.round() as i64,
        y: position.y.round() as i64,
    }
}
