//! Digging out one cave from smoothed noise.

use super::super::grid::{STEPS, chance};
use super::{Cave, FILL_PERCENT, SMOOTHING_PASSES};
use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::Rng;

/// Digs one cave: its ellipse filled at random, plus a random walk from the
/// centre (so the middle joins up), smoothed by the cellular automata rule,
/// keeping only what joins up with the centre. Never a plain rectangle.
pub(super) fn dig_cave(level: &mut Level, rng: &mut Rng, cave: &Cave, room: RoomId) {
    let (x0, y0) = (cave.x - cave.rx - 1, cave.y - cave.ry - 1);
    let (w, h) = (2 * cave.rx + 3, 2 * cave.ry + 3);
    let index = |x: i64, y: i64| ((y - y0) * w + (x - x0)) as usize;
    let inside = |x: i64, y: i64| x >= x0 && y >= y0 && x < x0 + w && y < y0 + h;
    let mut dug = vec![false; (w * h) as usize];

    for cy in y0..y0 + h {
        for cx in x0..x0 + w {
            dug[index(cx, cy)] = cave.contains(cx, cy) && chance(rng, FILL_PERCENT);
        }
    }
    let (mut x, mut y) = (cave.x, cave.y);
    dug[index(x, y)] = true;
    for _ in 0..cave.rx * cave.ry * 2 {
        let (dx, dy) = STEPS[rng.range_u64(0, 4) as usize];
        if cave.contains(x + dx, y + dy) {
            (x, y) = (x + dx, y + dy);
            dug[index(x, y)] = true;
        }
    }

    // Cellular automata: dug where most of the eight neighbours are.
    let count = |dug: &[bool], x: i64, y: i64| {
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) && inside(x + dx, y + dy) && dug[index(x + dx, y + dy)] {
                    n += 1;
                }
            }
        }
        n
    };
    for _ in 0..SMOOTHING_PASSES {
        let before = dug.clone();
        for cy in y0..y0 + h {
            for cx in x0..x0 + w {
                let n = count(&before, cx, cy);
                let i = index(cx, cy);
                dug[i] = cave.contains(cx, cy) && (n >= 5 || (before[i] && n >= 4));
            }
        }
    }
    dug[index(cave.x, cave.y)] = true;

    // Keep only what joins up with the centre.
    let mut keep = vec![false; dug.len()];
    let mut stack = vec![(cave.x, cave.y)];
    keep[index(cave.x, cave.y)] = true;
    while let Some((cx, cy)) = stack.pop() {
        for (dx, dy) in STEPS {
            let (nx, ny) = (cx + dx, cy + dy);
            if inside(nx, ny) && dug[index(nx, ny)] && !keep[index(nx, ny)] {
                keep[index(nx, ny)] = true;
                stack.push((nx, ny));
            }
        }
    }

    // Never a rectangle: if it fills its bounding box, take a corner off.
    let cells: Vec<(i64, i64)> = (y0..y0 + h)
        .flat_map(|cy| (x0..x0 + w).map(move |cx| (cx, cy)))
        .filter(|&(cx, cy)| keep[index(cx, cy)])
        .collect();
    let (min_x, max_x) = (
        cells.iter().map(|c| c.0).min(),
        cells.iter().map(|c| c.0).max(),
    );
    let (min_y, max_y) = (
        cells.iter().map(|c| c.1).min(),
        cells.iter().map(|c| c.1).max(),
    );
    if let (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) = (min_x, max_x, min_y, max_y)
        && (max_x - min_x + 1) * (max_y - min_y + 1) == cells.len() as i64
    {
        if max_x > min_x && max_y > min_y {
            // A corner away from the centre (a full rectangle stays joined up
            // without one).
            let corner = [
                (min_x, min_y),
                (max_x, min_y),
                (min_x, max_y),
                (max_x, max_y),
            ]
            .into_iter()
            .find(|&c| c != (cave.x, cave.y))
            .expect("four corners");
            keep[index(corner.0, corner.1)] = false;
        } else {
            // A line or a single cell: add cells to make an L.
            keep[index(min_x + 1, min_y)] = true;
            keep[index(min_x, max_y + 1)] = true;
        }
    }

    for cy in y0..y0 + h {
        for cx in x0..x0 + w {
            if keep[index(cx, cy)] && level.cell(cx, cy) == Cell::Wall {
                level.set(cx, cy, Cell::Floor);
                level.set_room(cx, cy, Some(room));
            }
        }
    }
}
