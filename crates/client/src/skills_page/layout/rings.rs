//! How far out each level's ring goes: at least its usual place (`RINGS`), and far
//! enough that no two circles touch, inside out.

use super::{RINGS, SIZES};

/// The room kept between two circles, at zoom 1.
pub(super) const GAP: f32 = 12.0;

/// Each level's radius, from every placed skill's level and angle (radians).
pub(super) fn radii(placed: impl Iterator<Item = (u8, f32)>) -> [f32; 5] {
    let mut angles: [Vec<f32>; 5] = Default::default();
    for (depth, angle) in placed {
        angles[usize::from(depth).min(4)].push(angle);
    }
    let mut radii = RINGS;
    for depth in 1..radii.len() {
        let inner = radii[depth - 1];
        // Clear of the level inside it, and in the same proportion to it as the
        // usual rings, which keeps every branch pointing outwards (`MAX_SLICE`).
        let mut radius = RINGS[depth].max(inner + (SIZES[depth - 1] + SIZES[depth]) / 2.0 + GAP);
        if depth > 1 {
            radius = radius.max(inner * RINGS[depth] / RINGS[depth - 1]);
        }
        // Wide enough round that neighbours on it don't touch.
        radius = radius.max(room_round(&mut angles[depth], SIZES[depth] + GAP));
        radii[depth] = radius;
    }
    radii
}

/// The radius at which circles at these angles are at least `apart` from their
/// neighbours, centre to centre (by the chord between them).
fn room_round(angles: &mut [f32], apart: f32) -> f32 {
    if angles.len() < 2 {
        return 0.0;
    }
    angles.sort_by(f32::total_cmp);
    let wrap = std::f32::consts::TAU - (angles[angles.len() - 1] - angles[0]);
    let closest = angles
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .chain([wrap])
        .fold(f32::INFINITY, f32::min)
        .max(1e-4);
    apart / (2.0 * (closest.min(std::f32::consts::PI) / 2.0).sin())
}
