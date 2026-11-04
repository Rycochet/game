//! Plates, for the `Plates` generator: the planet's crust in pieces, some
//! carrying continents, drifting; their edges make the ranges, arcs, trenches,
//! ridges and rifts. The continents' noise adds the detail on top.

mod boundary;
mod cells;

use fastnoise_lite::FastNoiseLite;

use super::layers::{Point, layered, scaled};
use super::shape::smoothstep;
use crate::surface::PlateSample;
use crate::surface::new_noise;
use crate::world::WorldParams;
pub(super) use boundary::Boundary;
use cells::{Cells, Plate};

/// How far the boundaries wander, in plate cells, and how fine that is.
const WANDER: f64 = 0.35;
const WANDER_SCALE: f64 = 2.0;
/// How far from a boundary between a continental and an oceanic plate the
/// land and sea blend, in plate cells.
const BLEND: f64 = 0.25;

/// The plates of one world.
pub(super) struct Plates {
    cells: Cells,
    wander: FastNoiseLite,
    width: i64,
}

/// What the plates make at a point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Tectonics {
    /// `1` well inside a continental plate, `-1` an oceanic one, between at
    /// their boundaries.
    pub(super) base: f64,
    pub(super) boundary: Boundary,
}

impl Plates {
    pub(super) fn new(world: &WorldParams, width: i64) -> Self {
        Self {
            cells: Cells::new(world.seed, width),
            wander: new_noise(world, "plates wander"),
            width,
        }
    }

    /// What the plates make at grid position (`x`, `y`), with `point` its
    /// place in the continents' noise space, for the boundaries' wandering.
    pub(super) fn at(&self, at: (i64, i64), point: Point) -> Tectonics {
        let (here, there, edge) = self.find(at, point);
        let kind = |continental: bool| if continental { 1.0 } else { -1.0 };
        let mix = 0.5 + 0.5 * smoothstep(0.0, BLEND, edge);
        Tectonics {
            base: kind(here.continental) * mix + kind(there.continental) * (1.0 - mix),
            boundary: boundary::boundary(&here, &there, edge),
        }
    }

    /// Which plate a point is on, for looking at.
    pub(super) fn sample(&self, at: (i64, i64), point: Point) -> PlateSample {
        let (here, there, edge) = self.find(at, point);
        PlateSample {
            plate: here.cell,
            continental: here.continental,
            edge,
            closing: boundary::closing(&here, &there),
        }
    }

    /// The plate a point is on, the next nearest, and how far it is from the
    /// line between them, in plate cells.
    fn find(&self, (x, y): (i64, i64), point: Point) -> (Plate, Plate, f64) {
        let size = self.cells.size;
        let push = |offset: Point| {
            let moved = (point.0 + offset.0, point.1 + offset.1, point.2 + offset.2);
            layered(&self.wander, scaled(moved, WANDER_SCALE), 3, u32::MAX).0 * WANDER * size
        };
        let at = (
            x.rem_euclid(self.width) as f64 + push((0.0, 0.0, 0.0)),
            y as f64 + push((7.1, 3.3, 9.7)),
        );
        let ((here, near), (there, next)) = self.cells.nearest_two(at);
        // How far from the line halfway between the two centres, in cells.
        let (dx, dy) = (
            there.centre.0 - here.centre.0,
            there.centre.1 - here.centre.1,
        );
        let apart = libm::sqrt(dx * dx + dy * dy).max(1e-9);
        let edge = (next - near) / (2.0 * apart) / size;
        (here, there, edge)
    }
}
