//! The trunk's outline: a fixed number of rings along the spine, from the rump to the
//! base of the neck, each with its half width and how far its front (a beast's belly)
//! and its back are from the spine. The same number on every body, so one body blends
//! into another ring by ring. Beasts' rings are measured (`measured.rs`); every other
//! body's are worked out from its three stations, as its trunk was always drawn.

use super::layout::{Layout, Role};

/// How many rings every trunk has.
pub const RINGS: usize = 10;

/// One ring round the trunk, in millimetres.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Ring {
    /// How far along the spine from the middle of the hips (behind them is negative).
    pub along: i32,
    /// Its half width across.
    pub half: i32,
    /// How far its front is in front of the spine: forwards when upright, down when
    /// level (a beast's belly). Negative when the whole ring is behind the spine, as a
    /// rump's tip above a level back.
    pub belly: i32,
    /// How far its back is behind the spine: up when level.
    pub back: i32,
}

impl Ring {
    /// A ring round the spine, as deep in front as behind.
    const fn round(along: i32, (half, deep): (i32, i32)) -> Ring {
        Ring {
            along,
            half,
            belly: deep,
            back: deep,
        }
    }

    /// Halfway between two rings.
    fn between(a: Ring, b: Ring) -> Ring {
        let half = |x: i32, y: i32| (x + y) / 2;
        Ring {
            along: half(a.along, b.along),
            half: half(a.half, b.half),
            belly: half(a.belly, b.belly),
            back: half(a.back, b.back),
        }
    }
}

/// The rings worked out from a body's three stations, its limbs and its neck: the
/// trunk's six levels as it was always drawn (from the crotch, the hips, waist and
/// chest, the shoulders as wide as the arms hang apart, and the neck's base), with a
/// ring halfway along each gap above the crotch, so a body looks as it did.
pub fn from_stations(layout: &Layout) -> [Ring; RINGS] {
    let s = &layout.spine;
    // The crotch is below the hips by the thigh's thickness.
    let thigh = layout
        .limbs()
        .find(|limb| matches!(limb.role, Role::Leg | Role::Hindleg))
        .and_then(|limb| limb.bones[0])
        .map_or(s.hips.half.1 / 2, |bone| bone.radius);
    // The shoulders as wide as the arms (or forelegs) hang apart.
    let shoulders = layout
        .limbs()
        .find(|limb| matches!(limb.role, Role::Arm | Role::WingArm | Role::Foreleg))
        .map_or(s.chest.half.0, |limb| limb.apart);
    let neck = layout.neck.radius;
    let levels = [
        Ring::round(-thigh, s.hips.half),
        Ring::round(0, s.hips.half),
        Ring::round(s.waist.along, s.waist.half),
        Ring::round(s.chest.along, s.chest.half),
        Ring::round(s.shoulders, (shoulders, s.chest.half.1 * 7 / 10)),
        Ring::round(s.neck_base, (neck * 16 / 10, neck * 12 / 10)),
    ];
    let mut rings = [levels[0]; RINGS];
    rings[1] = levels[1];
    for gap in 1..levels.len() - 1 {
        rings[gap * 2] = Ring::between(levels[gap], levels[gap + 1]);
        rings[gap * 2 + 1] = levels[gap + 1];
    }
    rings
}

#[cfg(test)]
mod tests;
