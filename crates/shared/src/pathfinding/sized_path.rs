//! Routes for creatures of a size between any two spots: A* over cells'
//! middles with room for them (`clearance::RoomMap`), each move checked by the
//! one rule (`clearance::check_segment_by`: a good swimmer swims water) and
//! priced as moves are (`movement::LevelWay`: slopes, wading, jumps) with its
//! squeeze, then straightened to any angle (`straighten_spots`).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{DIRECTIONS, straighten_spots};
use crate::clearance::{RoomMap, check_segment_by, has_footing_by};
use crate::dungeon::{Cell, Level};
use crate::movement::{LevelWay, PIECE};
use crate::spot::Spot;

/// A route of straight moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SizedPath {
    /// The spots it goes through, starting where it starts.
    pub spots: Vec<Spot>,
    /// What it takes, in units of walking on flat open ground
    /// (`LevelWay::cost`), longer by squeezes.
    pub cost: u64,
    /// Whether it ends at the goal; if not, it ends at the middle of the cell
    /// nearest the goal that could be reached.
    pub reached: bool,
}

/// What a straight move from `a` to `b` takes along `way` for `rooms`'s size,
/// squeezes and all, if it can be taken (a jump only if `jumps`).
pub(super) fn move_cost(
    way: &LevelWay,
    rooms: &RoomMap,
    (a, b): (Spot, Spot),
    jumps: bool,
) -> Option<u64> {
    let passage = check_segment_by(way.level, rooms.size(), rooms.swims(), a, b)?;
    let jumped = passage.jumped.is_some();
    if jumped && !jumps {
        return None;
    }
    Some(way.cost(a, b, jumped) * u64::from(passage.squeeze_per_mille) / 1000)
}

/// The quickest route along `way` (its level, ground and gait) for creatures
/// of `rooms`'s size (a swimmer's, if it was made for one) from `from` to
/// `to`, through cells `allowed` accepts, no dearer than `max_cost` (which
/// keeps a search cheap), straightened to any angle. Slopes and wading count,
/// terrain doesn't (`way.terrain` is left out of the search). As `find_path`,
/// if the goal can't be reached it goes to the reachable cell nearest it:
/// check `reached`.
pub fn find_sized_path(
    way: &LevelWay,
    rooms: &RoomMap,
    from: Spot,
    to: Spot,
    max_cost: Option<u64>,
    allowed: impl Fn(i64, i64) -> bool,
) -> SizedPath {
    let way = LevelWay {
        terrain: None,
        ..*way
    };
    let level = way.level;
    let (start, goal) = (from.cell(), to.cell());
    if !rooms.has_room(start.0, start.1) {
        return SizedPath::default();
    }
    let (min_x, min_y, width, height) = level.bounds();
    let index = |(x, y): (i64, i64)| ((y - min_y) * width + (x - min_x)) as usize;
    let distance = |at: (i64, i64)| Spot::middle_of(at).distance(Spot::middle_of(goal));
    // Never more than the real cost: a good swimmer or jumper goes quicker
    // than walking, slopes only add, and each piece may round down once.
    let cheapest = u64::from(way.gait.cheapest_per_mille());
    let least = |at| {
        let d = distance(at);
        (d * cheapest / 1000).saturating_sub(d / PIECE + 1)
    };

    let mut best = vec![u64::MAX; (width * height) as usize];
    let mut came_from = vec![start; best.len()];
    best[index(start)] = 0;
    let mut open = BinaryHeap::new();
    open.push(Reverse((least(start), distance(start), 0, start)));
    let mut nearest = (distance(start), start);

    while let Some(Reverse((_, left, cost, at))) = open.pop() {
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((left, at));
        if at == goal {
            break;
        }
        for (dx, dy) in DIRECTIONS {
            for reach in [1, 2] {
                let next = (at.0 + dx * reach, at.1 + dy * reach);
                let jumps = reach == 2;
                if jumps && !jumpable(level, rooms.swims(), (at.0 + dx, at.1 + dy)) {
                    continue;
                }
                if !rooms.has_room(next.0, next.1) || !allowed(next.0, next.1) {
                    continue;
                }
                let between = (Spot::middle_of(at), Spot::middle_of(next));
                let Some(step) = move_cost(&way, rooms, between, jumps) else {
                    continue;
                };
                let next_cost = cost + step;
                if next_cost >= best[index(next)] || max_cost.is_some_and(|max| next_cost > max) {
                    continue;
                }
                best[index(next)] = next_cost;
                came_from[index(next)] = at;
                open.push(Reverse((
                    next_cost + least(next),
                    distance(next),
                    next_cost,
                    next,
                )));
            }
        }
    }

    let end = nearest.1;
    let mut cells = Vec::new();
    let mut at = end;
    while at != start {
        cells.push(at);
        at = came_from[index(at)];
    }
    cells.push(start);
    cells.reverse();
    let mut spots = vec![from];
    spots.extend(cells.into_iter().map(Spot::middle_of));
    let last = *spots.last().expect("there is a start");
    let reached = end == goal && move_cost(&way, rooms, (last, to), true).is_some();
    if reached {
        spots.push(to);
    }
    spots.dedup();
    let spots = straighten_spots(&way, rooms, &spots);
    let cost = spots
        .windows(2)
        .map(|pair| move_cost(&way, rooms, (pair[0], pair[1]), true).unwrap_or(0))
        .sum();
    SizedPath {
        spots,
        cost,
        reached,
    }
}

/// Whether a cell is one a jump can go over (water isn't, for a swimmer).
fn jumpable(level: &Level, swims: bool, (x, y): (i64, i64)) -> bool {
    let cell = level.cell(x, y);
    cell == Cell::Fence || (cell.is_jumpable() && !has_footing_by(level, swims, x, y))
}
