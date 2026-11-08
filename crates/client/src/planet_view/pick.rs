//! Which World Gate's ring is under a point of the picture: each gate's spot
//! on the globe, turned as the globe is now, seen through the camera, and the
//! nearest within reach of the point. Plain maths, tested alone.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::globe::FIELD_OF_VIEW;

/// Half the picture's square field of view, as `globe` has it, in radians.
const HALF_VIEW: f32 = FIELD_OF_VIEW / 2.0 * PI / 180.0;
/// The least reach round a gate, in the interface's pixels, so a small ring
/// in a small window can still be hit.
const LEAST_REACH: f32 = 12.0;

/// Where a gate is on the globe's picture: `u` round from `x = 0`, `v` down
/// from the north pole, both 0 to 1, as `gates::stamp_gates` draws it.
pub(super) fn gate_spot((x, y): (i64, i64), width: i64) -> Vec2 {
    let u = x.rem_euclid(width) as f32 / width as f32;
    let v = y as f32 / (width as f32 / 2.0) + 0.5;
    Vec2::new(u, v)
}

/// The point of the unit globe, before it turns, that shows a point of its
/// picture. Bevy's sphere puts `u` round its Z axis from +X and `v` down from
/// +Z; `globe` tips it so its +Z is north (+Y) and its +Y is -Z.
pub(super) fn on_globe(spot: Vec2) -> Vec3 {
    let (round, down) = (spot.x * TAU, spot.y * PI);
    let across = down.sin();
    Vec3::new(across * round.cos(), down.cos(), -across * round.sin())
}

/// Where a point in the scene shows on the picture, `0..1` across and down,
/// if it is in front of the camera.
pub(super) fn on_picture(point: Vec3, camera: &Transform) -> Option<Vec2> {
    let seen = camera.rotation.inverse() * (point - camera.translation);
    if seen.z >= 0.0 {
        return None;
    }
    let scale = -seen.z * HALF_VIEW.tan();
    let flat = Vec2::new(seen.x, seen.y) / scale;
    Some(Vec2::new(flat.x + 1.0, 1.0 - flat.y) / 2.0)
}

/// What a pick looks at.
pub(super) struct Sight<'a> {
    /// The camera, and how the globe is turned.
    pub(super) camera: &'a Transform,
    pub(super) globe: Quat,
    /// The picture's width, in the interface's pixels.
    pub(super) across: f32,
    /// A ring's radius, as a share of the picture of the planet's width.
    pub(super) ring: f32,
}

/// The number of the gate (in `spots`' order) whose ring is nearest `at`
/// (`0..1` across and down the picture), if any is within reach: its ring
/// as seen, or `LEAST_REACH`, whichever is more. Gates round the back of the
/// globe can't be seen, so can't be picked.
pub(super) fn gate_under(at: Vec2, spots: &[Vec2], sight: &Sight) -> Option<usize> {
    let least = LEAST_REACH / sight.across.max(1.0);
    let mut nearest: Option<(usize, f32)> = None;
    for (number, &spot) in spots.iter().enumerate() {
        let point = sight.globe * on_globe(spot);
        // Facing the camera: on the near side of the globe.
        if point.dot(sight.camera.translation - point) <= 0.0 {
            continue;
        }
        let Some(middle) = on_picture(point, sight.camera) else {
            continue;
        };
        let edge = sight.globe * on_globe(spot + Vec2::new(sight.ring, 0.0));
        let ring = on_picture(edge, sight.camera).map_or(0.0, |edge| edge.distance(middle));
        let distance = at.distance(middle);
        if distance <= ring.max(least) && nearest.is_none_or(|(_, best)| distance < best) {
            nearest = Some((number, distance));
        }
    }
    nearest.map(|(number, _)| number)
}

#[cfg(test)]
mod tests;
