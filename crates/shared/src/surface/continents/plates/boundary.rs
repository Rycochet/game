//! What happens where two plates meet, by how they move and what they carry:
//! - pushing together, continent against continent: a great range on both
//!   sides (as the Himalaya);
//! - continent against ocean: a range along the coast (as the Andes), and a
//!   trench off it where the ocean plate dives under;
//! - ocean against ocean: one dives, leaving a trench, and the other rises
//!   into an arc of islands;
//! - pulling apart: a ridge along the ocean floor, or a rift valley on land.
//!
//! Sliding past each other leaves nothing.

use super::super::shape::smoothstep;
use super::cells::Plate;

/// How fast plates must close or part (of 2 at most) to make anything, and
/// how fast for the most.
const SLOW: f64 = 0.2;
const FAST: f64 = 1.0;
/// How far from the boundary each thing reaches, in plate cells.
const RANGE_REACH: f64 = 0.2;
const COASTAL_REACH: f64 = 0.14;
const TRENCH_REACH: f64 = 0.025;
const ARC_REACH: f64 = 0.03;
const RIDGE_REACH: f64 = 0.04;
const RIFT_REACH: f64 = 0.02;
/// How deep, or high, each is at its strongest, in metres.
const TRENCH: f64 = 4000.0;
const ARC: f64 = 4500.0;
const OCEAN_RIDGE: f64 = 2500.0;
const RIFT: f64 = 800.0;

/// What a boundary adds at a point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::surface::continents) struct Boundary {
    /// How strong a mountain range is here, `0..=1`.
    pub(in crate::surface::continents) range: f64,
    /// Metres added (or taken, if below 0) to the ground.
    pub(in crate::surface::continents) metres: f64,
}

/// For a point on plate `here`, `edge` cells from its boundary with `there`.
pub(super) fn boundary(here: &Plate, there: &Plate, edge: f64) -> Boundary {
    let closing = closing(here, there);
    let near = |reach: f64| 1.0 - smoothstep(0.0, reach, edge);
    let strength = smoothstep(SLOW, FAST, closing.abs());
    if strength <= 0.0 {
        return Boundary::default();
    }
    let mut found = Boundary::default();
    if closing > 0.0 {
        match (here.continental, there.continental) {
            (true, true) => found.range = strength * near(RANGE_REACH),
            (true, false) => found.range = strength * near(COASTAL_REACH),
            (false, true) => found.metres = -TRENCH * strength * near(TRENCH_REACH),
            (false, false) => {
                // The plate with the lower cell dives; the other rises.
                if here.cell < there.cell {
                    found.metres = -TRENCH * strength * near(TRENCH_REACH);
                } else {
                    found.metres = ARC * strength * near(ARC_REACH);
                    found.range = 0.5 * strength * near(ARC_REACH);
                }
            }
        }
    } else if here.continental || there.continental {
        found.metres = -RIFT * strength * near(RIFT_REACH);
    } else {
        found.metres = OCEAN_RIDGE * strength * near(RIDGE_REACH);
    }
    found
}

/// How fast two plates close on each other, `-2..2`: below 0 they part.
pub(super) fn closing(here: &Plate, there: &Plate) -> f64 {
    let (nx, ny) = towards(here, there);
    (here.drift.0 - there.drift.0) * nx + (here.drift.1 - there.drift.1) * ny
}

/// The way from one plate's centre to the other's, of length 1.
fn towards(here: &Plate, there: &Plate) -> (f64, f64) {
    let (dx, dy) = (
        there.centre.0 - here.centre.0,
        there.centre.1 - here.centre.1,
    );
    let length = libm::sqrt(dx * dx + dy * dy).max(1e-9);
    (dx / length, dy / length)
}
