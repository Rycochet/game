//! Where the plates are: one per cell of a coarse grid round the planet, its
//! centre somewhere in the cell from the seed, and its kind and drift from the
//! seed too. A point belongs to the plate whose centre is nearest.

use core::f64::consts::TAU;

use crate::rng::Seed;

/// Plates across the planet's width; as many rows of them as fit.
pub(super) const PLATES_AROUND: i64 = 10;
/// How much of its cell a plate's centre may wander over (from the middle).
const WANDER: f64 = 0.4;
/// The share of plates carrying a continent.
const CONTINENTAL: f64 = 0.4;

/// One plate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Plate {
    /// Its cell, `x` wrapped to `0..PLATES_AROUND`: also which plate it is.
    pub(super) cell: (i64, i64),
    /// Its centre, in grid units (`x` may be past the width, for wrapping).
    pub(super) centre: (f64, f64),
    pub(super) continental: bool,
    /// Which way and how fast it moves, at most 1.
    pub(super) drift: (f64, f64),
}

/// The plates of one world, on a grid of cells `size` grid units across.
pub(super) struct Cells {
    seed: Seed,
    pub(super) size: f64,
}

impl Cells {
    pub(super) fn new(seed: Seed, width: i64) -> Self {
        Self {
            seed: seed.derive("plates"),
            size: width as f64 / PLATES_AROUND as f64,
        }
    }

    /// The plate of cell (`column`, `row`), `column` not yet wrapped.
    fn plate(&self, column: i64, row: i64) -> Plate {
        let wrapped = column.rem_euclid(PLATES_AROUND);
        let mut rng = Seed::new(self.seed.at(&[wrapped, row])).rng();
        let mut wander = || 0.5 + rng.range_f64(-WANDER, WANDER);
        let centre = (
            (column as f64 + wander()) * self.size,
            (row as f64 + wander()) * self.size,
        );
        let continental = rng.next_f64() < CONTINENTAL;
        let angle = rng.range_f64(0.0, TAU);
        let speed = rng.range_f64(0.3, 1.0);
        Plate {
            cell: (wrapped, row),
            centre,
            continental,
            drift: (libm::cos(angle) * speed, libm::sin(angle) * speed),
        }
    }

    /// The nearest plate to a point and the next nearest, with their
    /// distances squared.
    pub(super) fn nearest_two(&self, (x, y): (f64, f64)) -> ((Plate, f64), (Plate, f64)) {
        let column = libm::floor(x / self.size) as i64;
        let row = libm::floor(y / self.size) as i64;
        let mut best: [(Option<Plate>, f64); 2] = [(None, f64::MAX), (None, f64::MAX)];
        for down in -1..=1 {
            for across in -1..=1 {
                let plate = self.plate(column + across, row + down);
                let (dx, dy) = (x - plate.centre.0, y - plate.centre.1);
                let distance = dx * dx + dy * dy;
                if distance < best[0].1 {
                    best[1] = best[0];
                    best[0] = (Some(plate), distance);
                } else if distance < best[1].1 {
                    best[1] = (Some(plate), distance);
                }
            }
        }
        let [(Some(first), near), (Some(second), next)] = best else {
            unreachable!("nine cells always give two plates");
        };
        ((first, near), (second, next))
    }
}
