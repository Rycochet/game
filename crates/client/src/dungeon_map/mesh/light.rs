//! Vertex colours from light, corner light, and faces towards the camera.

use bevy::prelude::*;
use shared::dungeon::Level;

use super::super::{AMBIENT_LIGHT, CELL_SIZE};
use super::LightGrid;

/// A vertex colour: `light` on top of the dull ambient light.
pub fn with_ambient(light: Vec3) -> Vec3 {
    AMBIENT_LIGHT + (Vec3::ONE - AMBIENT_LIGHT) * light
}

/// The light at the grid corner nearest a vertex's (x, z) position.
pub(in crate::dungeon_map) fn corner_light(
    level: &Level,
    light: &LightGrid,
    position: Vec3,
) -> Option<Vec3> {
    let x = (position.x / CELL_SIZE + 0.5).round() as i64;
    let y = (position.z / CELL_SIZE + 0.5).round() as i64;
    light.corner(level, x, y)
}

// Whether a wall side (centred on `centre`, facing `normal`) faces the
/// camera at `camera`: the camera is in front of the side's plane. Measured
/// from the camera's real position (not just the way it looks), so a side
/// square-on to the view turns away only once the camera passes its plane.
pub fn faces_camera(centre: Vec3, normal: Vec3, camera: Vec3) -> bool {
    normal.dot(camera - centre) >= 0.0
}
