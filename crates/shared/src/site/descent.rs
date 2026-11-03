//! Where stairs down lead on the level below.

use super::{Exit, Site, Stairs, StairsDirection};
use crate::dungeon::Level;

impl Site {
    /// The way `stairs` on level `index` (`level`) go down, as a grid step
    /// from their top end to their bottom end, agreeing with the matching
    /// stairs on the level they lead to (generated again to look). Only for
    /// looks: both ends of a staircase are drawn running the same way.
    pub fn stairs_descend(&self, index: u32, level: &Level, stairs: Stairs) -> (i64, i64) {
        let other = match stairs.to {
            Exit::Level(to) => {
                let other = self.level(to);
                let back = other
                    .stairs
                    .iter()
                    .find(|s| s.to == Exit::Level(index))
                    .map(|s| (s.x, s.y));
                back.map(|cell| (other.level, cell))
            }
            Exit::WorldMap => None,
        };
        let here = Some((level, (stairs.x, stairs.y)));
        let other = other.as_ref().map(|(level, cell)| (level, *cell));
        match stairs.direction {
            StairsDirection::Down => descent_way(here, other),
            StairsDirection::Up => descent_way(other, here),
        }
    }
}

/// The four sides of a cell, as grid steps (`y` south), in the order ties
/// are settled.
pub(super) const SIDES: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// The way a staircase goes down between the cell `upper` (its top, on the
/// level above) and `lower` (its bottom, on the level below), either unknown
/// (the world map). Best is walkable floor to step on at the top end of the
/// upper level and off at the bottom end of the lower one; then a wall at
/// the far end of each (stairs run up against a wall). Ties go to the first
/// of `SIDES`.
pub(super) fn descent_way(
    upper: Option<(&Level, (i64, i64))>,
    lower: Option<(&Level, (i64, i64))>,
) -> (i64, i64) {
    let score = |(dx, dy): (i64, i64)| {
        let mut score = 0;
        if let Some((level, (x, y))) = upper {
            score += 2 * level.is_walkable(x - dx, y - dy) as u32;
            score += !level.is_walkable(x + dx, y + dy) as u32;
        }
        if let Some((level, (x, y))) = lower {
            score += 2 * level.is_walkable(x + dx, y + dy) as u32;
            score += !level.is_walkable(x - dx, y - dy) as u32;
        }
        score
    };
    let mut best = SIDES[0];
    for side in SIDES {
        if score(side) > score(best) {
            best = side;
        }
    }
    best
}
