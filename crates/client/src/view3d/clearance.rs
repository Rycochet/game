//! Keeping a clear line from the camera to the player over raised wilderness
//! ground (owner): the camera is shown tilted up just enough that nothing of
//! the land lies on the way, or under the camera, so the back of the land's
//! meshes is never seen. The player's own tilt setting stays as it is
//! (`OrbitCamera::pitch`); only what is shown is raised (`raise`), easing in
//! and out.

use bevy::prelude::*;

use super::camera::{MAX_PITCH_DEGREES, OrbitCamera};

/// How far above the ground the line must pass, in cells.
const MARGIN: f32 = 0.25;

/// How high above the floor the line starts: the top of the player's token,
/// in cells, so a step beside them doesn't hide them.
const HEAD: f32 = 0.75;

/// How often the line is tried, in cells along the ground.
const STRIDE: f32 = 0.25;

/// How finely the tilt is raised while looking for a clear line.
const RAISE_STEP_DEGREES: f32 = 1.0;

/// How fast what is shown follows what is wanted: the share of the way left
/// gone in a second is `1 - e^-EASE`.
const EASE: f32 = 8.0;

/// How much higher than its setting the camera must be shown to see
/// `target` (on the floor, in world units) clearly, in radians: 0 if it
/// already does. `ground(x, y)` is a cell's floor in world units, `None`
/// where unknown (passed over); `cell` is a cell's size.
pub(super) fn clearing_raise(
    orbit: &OrbitCamera,
    target: Vec3,
    cell: f32,
    ground: impl Fn(i64, i64) -> Option<f32>,
) -> f32 {
    let most = MAX_PITCH_DEGREES.to_radians();
    let head = target + Vec3::Y * HEAD * cell;
    let mut raise = 0.0;
    loop {
        let pitch = (orbit.pitch + raise).min(most);
        let camera = orbit.transform_at(pitch, target).translation;
        if pitch >= most || is_clear(head, camera, cell, &ground) {
            return pitch - orbit.pitch;
        }
        raise += RAISE_STEP_DEGREES.to_radians();
    }
}

/// Whether the line from `from` to `to` keeps `MARGIN` above every cell's
/// ground it passes over, the cell under `to` included.
fn is_clear(from: Vec3, to: Vec3, cell: f32, ground: &impl Fn(i64, i64) -> Option<f32>) -> bool {
    let across = Vec2::new(to.x - from.x, to.z - from.z).length() / cell;
    let steps = (across / STRIDE).ceil().max(1.0) as usize;
    (1..=steps).all(|step| {
        let at = from.lerp(to, step as f32 / steps as f32);
        let (x, y) = ((at.x / cell).round() as i64, (at.z / cell).round() as i64);
        ground(x, y).is_none_or(|floor| at.y >= floor + MARGIN * cell)
    })
}

/// The raise shown this frame: `wanted`, or eased towards it from `now`
/// over `seconds` (at once with Reduce Motion, `snap`).
pub(super) fn eased(now: f32, wanted: f32, seconds: f32, snap: bool) -> f32 {
    match snap {
        true => wanted,
        false => now + (wanted - now) * (1.0 - (-EASE * seconds).exp()),
    }
}

#[cfg(test)]
mod tests;
