//! Helpers for journeys: routes through a level's doors, moving towards a
//! point, enemies in the way, and the cell a point is in.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::dungeon::Level;
use shared::movement::{Gait, STRAIGHT_COST};
use shared::pathfinding::{Path, find_path_by, straighten_by};
use shared::sheet::Feeling;

use super::Player;
use crate::common::{GodMode, MapPosition, Other};

/// What a door known to be locked adds to a route: as much as a long way round.
const LOCKED_COST: u32 = 100 * STRAIGHT_COST;

/// The quickest route on a level from `from` to `to` through the cells `known`
/// accepts, by `cost_of`: through closed doors as if open (they are opened on
/// the way), each costing a step more, so an open way round is taken if it is
/// no longer; through one known to be locked only if there is no other way.
pub(super) fn dungeon_route(
    level: &Level,
    (from, to): ((i64, i64), (i64, i64)),
    known: impl Fn(i64, i64) -> bool + Copy,
    cost_of: impl Fn((i64, i64), (i64, i64)) -> u32 + Copy,
    gait: Gait,
) -> Path {
    // Locked ones too, if there is no other way: they are tried.
    let open = level.with_all_doors_open();
    let cost = |a, b: (i64, i64)| {
        let door = if level.is_locked(b.0, b.1) {
            LOCKED_COST
        } else if level.is_shut(b.0, b.1) {
            STRAIGHT_COST
        } else {
            0
        };
        cost_of(a, b) + door
    };
    let path = find_path_by(&open, from, to, None, known, cost, gait);
    Path {
        cells: straighten_by(&open, from, &path, known, cost, gait).cells,
        ..path
    }
}

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
