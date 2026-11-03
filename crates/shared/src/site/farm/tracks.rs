//! Tracks from the farmhouse door to the field gates.

use super::super::building::{self};
use super::super::grid::STEPS;
use crate::dungeon::{Cell, Level};

/// Dirt tracks over bare ground: from the front door to the nearest edge of
/// the level, and from each gate to the nearest track.
pub(super) fn lay_tracks(
    level: &mut Level,
    house: &building::Plan,
    door: (i64, i64),
    gates: &[(i64, i64)],
) {
    let shape = house.ground();
    let at_edge = |level: &Level, x: i64, y: i64| {
        STEPS.iter().any(|(dx, dy)| !level.contains(x + dx, y + dy))
    };
    let main = route(level, |x, y| shape.contains(x, y), door, at_edge);
    let is_track = |level: &Level, x: i64, y: i64| level.cell(x, y) == Cell::DirtTrack;
    for (x, y) in main {
        level.set(x, y, Cell::DirtTrack);
    }
    for &outside in gates {
        for (x, y) in route(level, |x, y| shape.contains(x, y), outside, is_track) {
            level.set(x, y, Cell::DirtTrack);
        }
    }
}

/// The shortest way from `from` across bare ground or track to a cell `done`
/// accepts, with every cell on it. Nothing if there is none.
pub(super) fn route(
    level: &Level,
    forbidden: impl Fn(i64, i64) -> bool,
    from: (i64, i64),
    done: impl Fn(&Level, i64, i64) -> bool,
) -> Vec<(i64, i64)> {
    let (min_x, min_y, width, height) = level.bounds();
    let at = |x: i64, y: i64| ((y - min_y) * width + (x - min_x)) as usize;
    let mut came_from: Vec<Option<(i64, i64)>> = vec![None; (width * height) as usize];
    came_from[at(from.0, from.1)] = Some(from);
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some((x, y)) = queue.pop_front() {
        if done(level, x, y) {
            let mut path = vec![(x, y)];
            let mut spot = (x, y);
            while spot != from {
                spot = came_from[at(spot.0, spot.1)].expect("came from somewhere");
                path.push(spot);
            }
            return path;
        }
        for (dx, dy) in STEPS {
            let (nx, ny) = (x + dx, y + dy);
            let cell = level.cell(nx, ny);
            if level.contains(nx, ny)
                && matches!(cell, Cell::Ground | Cell::DirtTrack)
                && !forbidden(nx, ny)
                && came_from[at(nx, ny)].is_none()
            {
                came_from[at(nx, ny)] = Some((x, y));
                queue.push_back((nx, ny));
            }
        }
    }
    Vec::new()
}
