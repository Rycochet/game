//! Digging a pond in a plot: farms' and forts'.

use super::farm::{Plot, fence_line};
use super::grid::{STEPS, roll};
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// How a pond is made: how many blobs (an inclusive range), and how far each
/// reaches from its middle, both ways (an inclusive range). Blobs reaching
/// past the plot are cut to it, so a big reach fills a small plot.
#[derive(Clone, Copy, Debug)]
pub(in crate::site) struct PondShape {
    pub blobs: (i64, i64),
    pub reach: (i64, i64),
}

/// Fills part of a plot with water, in one piece of no particular shape.
pub(in crate::site) fn dig_pond(level: &mut Level, rng: &mut Rng, plot: Plot, shape: PondShape) {
    let PondShape { blobs, reach } = shape;
    let Plot { x0, y0, x1, y1 } = plot;
    let (width, height) = (x1 - x0, y1 - y0);
    let at = |x: i64, y: i64| ((y - y0) * width + (x - x0)) as usize;
    let mut water = vec![false; (width * height) as usize];
    // Each blob starts inside what is there, so they are all one piece.
    let mut middle = ((x0 + x1) / 2, (y0 + y1) / 2);
    for _ in 0..roll(rng, blobs.0, blobs.1) {
        let (reach_x, reach_y) = (roll(rng, reach.0, reach.1), roll(rng, reach.0, reach.1));
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = ((x - middle.0) * reach_y, (y - middle.1) * reach_x);
                if dx * dx + dy * dy <= (reach_x * reach_y).pow(2) {
                    water[at(x, y)] = true;
                }
            }
        }
        let (x, y) = (
            roll(rng, middle.0 - reach_x, middle.0 + reach_x),
            roll(rng, middle.1 - reach_y, middle.1 + reach_y),
        );
        middle = (x.clamp(x0, x1 - 1), y.clamp(y0, y1 - 1));
        if !water[at(middle.0, middle.1)] {
            middle = ((x0 + x1) / 2, (y0 + y1) / 2);
        }
    }
    // Land inside it, with no way off, would be out of reach: flood from the
    // plot's edge, and what isn't reached is water.
    let mut dry = vec![false; water.len()];
    let mut stack: Vec<(i64, i64)> = fence_line(plot)
        .into_iter()
        .filter(|&(x, y)| !water[at(x, y)])
        .collect();
    while let Some((x, y)) = stack.pop() {
        if !dry[at(x, y)] {
            dry[at(x, y)] = true;
            for (dx, dy) in STEPS {
                let (nx, ny) = (x + dx, y + dy);
                let inside = (x0..x1).contains(&nx) && (y0..y1).contains(&ny);
                if inside && !water[at(nx, ny)] {
                    stack.push((nx, ny));
                }
            }
        }
    }
    for y in y0..y1 {
        for x in x0..x1 {
            if !dry[at(x, y)] {
                level.set(x, y, Cell::Water);
            }
        }
    }
}
