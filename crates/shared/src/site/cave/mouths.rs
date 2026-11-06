//! Digging the mouths a cave is entered by.

use super::super::grid::{STEPS, chance, roll};
use super::{BORDER, Cave, SECOND_MOUTH_PERCENT, wind};
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// A mouth's passage: its two cells just inside the mouth's ground, and the
/// way out from them.
#[derive(Clone, Copy, Debug)]
pub(super) struct Mouth {
    pub(super) passage: [(i64, i64); 2],
    pub(super) out: (i64, i64),
}

/// Digs the cave mouths: outside ground on the south edge (and sometimes the
/// east or west), as wide as the mouth, with a passage in. Returns where you
/// arrive from the world map (the middle of the south ground) and the mouths,
/// south first.
pub(super) fn dig_mouths(
    level: &mut Level,
    rng: &mut Rng,
    caves: &[Cave],
    bounds: (i64, i64, i64, i64),
) -> ((i64, i64), Vec<Mouth>) {
    let (_, _, width, height) = level.bounds();
    let mouth = |level: &mut Level, rng: &mut Rng, south: bool, east: bool| {
        let span = roll(rng, 4, 8);
        // Ground: `x0..x0 + w`, `y0..y0 + h`; the passage starts just inside.
        let (x0, y0, w, h, start) = if south {
            let x0 = roll(rng, 2, width - 2 - span);
            (
                x0,
                height - BORDER,
                span,
                BORDER,
                (x0 + span / 2, height - BORDER - 1),
            )
        } else {
            let y0 = roll(rng, height / 2, height - 2 - span);
            let x0 = if east { width - BORDER } else { 0 };
            let inside_x = if east { x0 - 1 } else { BORDER };
            (x0, y0, BORDER, span, (inside_x, y0 + span / 2))
        };
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                level.set(x, y, Cell::Ground);
                level.set_room(x, y, None);
            }
        }
        // A passage two wide into the nearest cave.
        let nearest = caves
            .iter()
            .min_by_key(|c| (c.x - start.0).pow(2) + (c.y - start.1).pow(2))
            .expect("a cave");
        let side = if south { (1, 0) } else { (0, 1) };
        let beside = (start.0 + side.0, start.1 + side.1);
        wind(level, rng, start, (nearest.x, nearest.y), bounds);
        wind(level, rng, beside, (nearest.x, nearest.y), bounds);
        let out = match (south, east) {
            (true, _) => (0, 1),
            (false, true) => (1, 0),
            (false, false) => (-1, 0),
        };
        let passage = Mouth {
            passage: [start, beside],
            out,
        };
        ((x0 + w / 2, y0 + h / 2), passage)
    };
    let (arrival, south) = mouth(level, rng, true, false);
    let mut mouths = vec![south];
    if chance(rng, SECOND_MOUTH_PERCENT) {
        let east = chance(rng, 50);
        mouths.push(mouth(level, rng, false, east).1);
    }
    (arrival, mouths)
}

/// The centre of the cave furthest (by walking) from `from`.
pub(super) fn furthest_cave(level: &Level, caves: &[Cave], from: (i64, i64)) -> (i64, i64) {
    let (min_x, min_y, width, height) = level.bounds();
    let index = |x: i64, y: i64| ((y - min_y) * width + (x - min_x)) as usize;
    let mut steps: Vec<Option<u32>> = vec![None; (width * height) as usize];
    let mut queue = std::collections::VecDeque::from([from]);
    steps[index(from.0, from.1)] = Some(0);
    while let Some((x, y)) = queue.pop_front() {
        let here = steps[index(x, y)].expect("visited");
        for (dx, dy) in STEPS {
            let (nx, ny) = (x + dx, y + dy);
            if level.is_walkable(nx, ny) && steps[index(nx, ny)].is_none() {
                steps[index(nx, ny)] = Some(here + 1);
                queue.push_back((nx, ny));
            }
        }
    }
    let other_caves = caves
        .iter()
        .filter(|c| (c.x, c.y) != from)
        .max_by_key(|c| steps[index(c.x, c.y)].unwrap_or(0))
        .map(|c| (c.x, c.y));
    // With only one cave, the furthest floor cell (first in row order).
    other_caves.unwrap_or_else(|| {
        let mut best = (0, from);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if let Some(n) = steps[index(x, y)]
                    && n > best.0
                    && level.cell(x, y) == Cell::Floor
                {
                    best = (n, (x, y));
                }
            }
        }
        best.1
    })
}
