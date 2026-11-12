//! Helpers for moving on a level: following a journey's route, closed doors
//! and jumps ahead.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::clearance::{JUMP_REACH, check_segment};
use shared::dungeon::Level;
use shared::spot::{CELL_SPAN, Spot};

use super::Travel;
use super::doors::{AskDoor, wait_at_door};
use super::stride::{Stride, Walker, to_vector, unit_distance};

/// How far jumps are tried, in units, nearest the usual two cells first.
const JUMP_LENGTHS: [f64; 5] = [480.0, 400.0, 560.0, 320.0, 640.0];

/// `from` moved `length` units `way`.
fn ahead_of(from: Spot, way: DVec2, length: f64) -> Spot {
    let by = way * length;
    from.offset(by.x.round() as i64, by.y.round() as i64)
}

/// A closed door just ahead of `at` going `way`, if any: the cell walked into
/// next.
pub(super) fn door_ahead(level: &Level, at: Spot, way: DVec2) -> Option<(i64, i64)> {
    [120.0, 200.0, 280.0]
        .into_iter()
        .map(|length| ahead_of(at, way, length).cell())
        .find(|&cell| cell != at.cell() && level.is_shut(cell.0, cell.1))
}

/// Where a jump from `at` going `way` lands, if one can be taken: over one
/// cell of liquid, a hole or a fence (`check_segment`).
pub(super) fn jump_ahead<F: Fn((i64, i64)) -> bool>(
    walker: &Walker<F>,
    at: Spot,
    way: DVec2,
) -> Option<Spot> {
    JUMP_LENGTHS
        .into_iter()
        .map(|length| ahead_of(at, way, length))
        .filter(|to| at.distance(*to) <= JUMP_REACH && (walker.free)(to.cell()))
        .find(|&to| {
            check_segment(walker.level, walker.size, at, to)
                .is_some_and(|passage| passage.jumped.is_some())
        })
}

/// Goes `distance` units (on flat ground; `slowness` says how much longer
/// a step from one cell into another takes) along a journey's route, adding
/// each straight move to `moves`. Waits at a closed door in the way (asked
/// open once); finds a new way from where it is if something else is in
/// the way, giving up if that found way is blocked at once. Whether it has
/// arrived where it was asked to go.
pub(super) fn follow_route<F: Fn((i64, i64)) -> bool>(
    travel: &mut Travel,
    walker: &Walker<F>,
    mut distance: f64,
    slowness: impl Fn((i64, i64), (i64, i64)) -> f64,
    moves: &mut Vec<(Spot, Spot)>,
    asks: &mut MessageWriter<AskDoor>,
) -> bool {
    while let Some(&next) = travel.spots.front() {
        let at = travel.stride.at;
        if at == next {
            travel.spots.pop_front();
            if travel.spots.is_empty() {
                moves.extend(travel.stride.finish());
                return travel.route_arrives;
            }
            continue;
        }
        if distance <= 0.0 {
            break;
        }
        // A jump on the route: taken whole, and shown.
        let jumps = check_segment(walker.level, walker.size, at, next)
            .is_some_and(|passage| passage.jumped.is_some());
        if jumps {
            moves.extend(travel.stride.finish());
            moves.push((at, next));
            travel.stride = Stride::at(next);
            travel.landing = Some(next);
            travel.spots.pop_front();
            return travel.spots.is_empty() && travel.route_arrives;
        }
        let way = to_vector(at, next) / unit_distance(at, next);
        let slow = slowness(at.cell(), ahead_of(at, way, CELL_SPAN as f64).cell()).max(0.1);
        let walked = travel
            .stride
            .walk(walker, way, distance / slow, Some(next), moves);
        distance -= walked.distance * slow;
        if walked.distance >= 1.0 {
            travel.rerouted = false;
        }
        if walked.stuck {
            if let Some(door) = door_ahead(walker.level, travel.stride.at, way) {
                wait_at_door(travel, door, asks);
            } else if travel.rerouted {
                info!("The way is blocked: the journey ends here");
                travel.spots.clear();
            } else {
                // Memory out of date, or someone in the way: a new way.
                travel.asked = true;
                travel.rerouted = true;
            }
            break;
        }
    }
    false
}
