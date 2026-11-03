//! Where the caves of a level go, and the cells inside one.

use super::super::grid::roll;
use super::{CAVE_GAP, CELLS_PER_CAVE, MAX_RADIUS, MIN_RADIUS};
use crate::rng::Rng;

/// One cave as planned: centre and radii.
#[derive(Clone, Copy, Debug)]
pub(super) struct Cave {
    pub(super) x: i64,
    pub(super) y: i64,
    pub(super) rx: i64,
    pub(super) ry: i64,
}

impl Cave {
    pub(super) fn contains(&self, x: i64, y: i64) -> bool {
        let (dx, dy) = (x - self.x, y - self.y);
        dx * dx * self.ry * self.ry + dy * dy * self.rx * self.rx
            <= self.rx * self.rx * self.ry * self.ry
    }
}

/// Picks where the caves go, apart from each other. Always at least one.
pub(super) fn place_caves(rng: &mut Rng, width: i64, height: i64, margin: i64) -> Vec<Cave> {
    let wanted = (width * height / CELLS_PER_CAVE).clamp(3, 24) as usize;
    let mut caves: Vec<Cave> = Vec::new();
    for _ in 0..wanted * 12 {
        if caves.len() >= wanted {
            break;
        }
        let rx = roll(rng, MIN_RADIUS, MAX_RADIUS);
        let ry = roll(rng, MIN_RADIUS, MAX_RADIUS);
        let (x0, x1) = (margin + rx, width - 1 - margin - rx);
        let (y0, y1) = (margin + ry, height - 1 - margin - ry);
        if x1 < x0 || y1 < y0 {
            continue;
        }
        let cave = Cave {
            x: roll(rng, x0, x1),
            y: roll(rng, y0, y1),
            rx,
            ry,
        };
        let clear = caves.iter().all(|other| {
            (cave.x - other.x).abs() > cave.rx + other.rx + CAVE_GAP
                || (cave.y - other.y).abs() > cave.ry + other.ry + CAVE_GAP
        });
        if clear {
            caves.push(cave);
        }
    }
    if caves.is_empty() {
        caves.push(Cave {
            x: width / 2,
            y: height / 2,
            rx: MIN_RADIUS,
            ry: MIN_RADIUS,
        });
    }
    caves
}
