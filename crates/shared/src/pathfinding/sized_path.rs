//! Routes for creatures of a size between any two spots: A* over cells'
//! middles with room for them (`clearance::RoomMap`), each move checked by the
//! one rule (`clearance::check_segment`) and priced with its squeeze, then
//! straightened to any angle (`straighten_spots`).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{DIRECTIONS, straighten_spots};
use crate::clearance::{RoomMap, has_footing, segment_cost};
use crate::dungeon::{Cell, Level};
use crate::spot::Spot;

/// A route of straight moves.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SizedPath {
    /// The spots it goes through, starting where it starts.
    pub spots: Vec<Spot>,
    /// What it takes, in units of distance walked freely (longer by
    /// squeezes).
    pub cost: u64,
    /// Whether it ends at the goal; if not, it ends at the middle of the cell
    /// nearest the goal that could be reached.
    pub reached: bool,
}

/// The quickest route for creatures of `rooms`'s size from `from` to `to`,
/// through cells `allowed` accepts, no dearer than `max_cost` (which keeps a
/// search cheap), straightened to any angle. As `find_path`, if the goal can't
/// be reached it goes to the reachable cell nearest it: check `reached`.
pub fn find_sized_path(
    level: &Level,
    rooms: &RoomMap,
    from: Spot,
    to: Spot,
    max_cost: Option<u64>,
    allowed: impl Fn(i64, i64) -> bool,
) -> SizedPath {
    let size = rooms.size();
    let (start, goal) = (from.cell(), to.cell());
    if !rooms.has_room(start.0, start.1) {
        return SizedPath::default();
    }
    let (min_x, min_y, width, height) = level.bounds();
    let index = |(x, y): (i64, i64)| ((y - min_y) * width + (x - min_x)) as usize;
    let least = |at: (i64, i64)| Spot::middle_of(at).distance(Spot::middle_of(goal));

    let mut best = vec![u64::MAX; (width * height) as usize];
    let mut came_from = vec![start; best.len()];
    best[index(start)] = 0;
    let mut open = BinaryHeap::new();
    open.push(Reverse((least(start), least(start), start)));
    let mut nearest = (least(start), start);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
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
                if reach == 2 && !jumpable(level, (at.0 + dx, at.1 + dy)) {
                    continue;
                }
                if !rooms.has_room(next.0, next.1) || !allowed(next.0, next.1) {
                    continue;
                }
                let Some(step) =
                    segment_cost(level, size, Spot::middle_of(at), Spot::middle_of(next))
                else {
                    continue;
                };
                let next_cost = cost + step;
                if next_cost >= best[index(next)] || max_cost.is_some_and(|max| next_cost > max) {
                    continue;
                }
                best[index(next)] = next_cost;
                came_from[index(next)] = at;
                let left = least(next);
                open.push(Reverse((next_cost + left, left, next)));
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
    let reached = end == goal && segment_cost(level, size, last, to).is_some();
    if reached {
        spots.push(to);
    }
    spots.dedup();
    let spots = straighten_spots(level, size, &spots);
    let cost = spots
        .windows(2)
        .map(|pair| segment_cost(level, size, pair[0], pair[1]).unwrap_or(0))
        .sum();
    SizedPath {
        spots,
        cost,
        reached,
    }
}

/// Whether a cell is one a jump can go over.
fn jumpable(level: &Level, (x, y): (i64, i64)) -> bool {
    let cell = level.cell(x, y);
    cell == Cell::Fence || (cell.is_jumpable() && !has_footing(level, x, y))
}
