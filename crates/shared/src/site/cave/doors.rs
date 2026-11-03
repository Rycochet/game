//! Doors in a cave's narrow places.

use super::super::grid::{self, chance};
use super::DOOR_PERCENT;
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// Doors where a passage enters a cave through a gap one cell wide.
pub(super) fn add_doors(level: &mut Level, rng: &mut Rng, keep: &[(i64, i64)]) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) != Cell::Floor
                || level.room(x, y).is_some()
                || keep.contains(&(x, y))
            {
                continue;
            }
            // A doorway: walls on two opposite sides, a cave cell on one of
            // the others and a passage on the last.
            let doorway = [(1, 0), (0, 1)].into_iter().any(|(dx, dy)| {
                let walls = level.cell(x + dy, y + dx) == Cell::Wall
                    && level.cell(x - dy, y - dx) == Cell::Wall;
                let a = (x + dx, y + dy);
                let b = (x - dx, y - dy);
                let ends = |p: (i64, i64), q: (i64, i64)| {
                    level.room(p.0, p.1).is_some()
                        && level.cell(q.0, q.1) == Cell::Floor
                        && level.room(q.0, q.1).is_none()
                };
                walls && (ends(a, b) || ends(b, a))
            });
            if doorway && !grid::door_nearby(level, x, y) && chance(rng, DOOR_PERCENT) {
                level.set(x, y, Cell::Door);
            }
        }
    }
}
