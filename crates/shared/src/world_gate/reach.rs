//! How much land a cell is joined to: a flood fill over coarse land, stopping
//! as soon as it has found enough, so it stays cheap on big continents.

use std::collections::{HashSet, VecDeque};

use super::land::CoarseLand;

/// Whether `start` is land joined to at least `needed` cells of land (itself
/// included), going no further from the equator than `furthest_y` either way.
pub(super) fn has_room(
    land: &mut CoarseLand,
    start: (i64, i64),
    needed: usize,
    furthest_y: i64,
) -> bool {
    if !land.is_land(start) {
        return false;
    }
    let mut seen = HashSet::from([land.wrapped(start)]);
    let mut next = VecDeque::from([start]);
    while let Some((x, y)) = next.pop_front() {
        if seen.len() >= needed {
            return true;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let cell = (x + dx, y + dy);
            let (_, middle_y) = land.middle(cell);
            if middle_y.abs() > furthest_y || seen.contains(&land.wrapped(cell)) {
                continue;
            }
            if land.is_land(cell) {
                seen.insert(land.wrapped(cell));
                next.push_back(cell);
            }
        }
    }
    seen.len() >= needed
}
