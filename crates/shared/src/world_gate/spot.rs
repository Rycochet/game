//! A gate's exact spot: the coarse cell was land by its broad height, but the
//! full surface may still have water just there, so look outward in square
//! rings until the ground is dry.

use crate::surface::Surface;

/// How far out to look, in grid points: further than a coarse cell.
const FURTHEST: i64 = 48;

/// The nearest dry point to (`x`, `y`) by the full surface, rings outward, or
/// `None` if there is none within `FURTHEST`.
pub(super) fn dry_spot(
    surface: &Surface,
    (x, y): (i64, i64),
    dry_above: f64,
) -> Option<(i64, i64)> {
    let dry = |at: (i64, i64)| surface.sample(at.0, at.1).height > dry_above;
    if dry((x, y)) {
        return Some((x, y));
    }
    for ring in 1..=FURTHEST {
        // Along the ring's top and bottom, then down its sides between them.
        for dx in -ring..=ring {
            for at in [(x + dx, y - ring), (x + dx, y + ring)] {
                if dry(at) {
                    return Some(at);
                }
            }
        }
        for dy in 1 - ring..ring {
            for at in [(x - ring, y + dy), (x + ring, y + dy)] {
                if dry(at) {
                    return Some(at);
                }
            }
        }
    }
    None
}
