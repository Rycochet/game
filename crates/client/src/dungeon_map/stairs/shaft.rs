//! The side of a hole, darker the deeper, and shading by the way faces face.

use bevy::prelude::*;

use super::super::CELL_SIZE;
use super::super::mesh::{Builder, Surface};
use super::{
    DEEPEST_SHADE, FRONT_SHADE, Hole, RIM_SHADE, STAIRS_DOWN_DEPTH, TREAD_TINT, floor_uv,
    hole_outline,
};

/// The side of a hole, in the wall's picture and darker the deeper (only its
/// inside: the near part faces away from the camera, so doesn't hide the
/// model), and a solid black base, so nothing behind can show through.
pub(super) fn shaft(meshes: &mut [Builder], centre: Vec3, hole: Hole, lit: Vec3) {
    let depth = Vec3::Y * STAIRS_DOWN_DEPTH;
    let outline = hole_outline(hole);
    let mut along = 0.0;
    for (i, &a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        let width = a.distance(b) / CELL_SIZE;
        let (a, b) = (centre + a, centre + b);
        let middle = (a + b) / 2.0;
        let normal = (centre - middle).normalize();
        let (u0, u1) = (along, (along + width).min(1.0));
        let v = STAIRS_DOWN_DEPTH / CELL_SIZE;
        let top = lit * RIM_SHADE;
        let bottom = lit * DEEPEST_SHADE;
        meshes[Surface::Wall as usize].free(
            [a, b, b - depth, a - depth],
            normal,
            [[u0, 0.0], [u1, 0.0], [u1, v], [u0, v]],
            [top, top, bottom, bottom],
        );
        along = (along + width).fract();
        // A slice of the base, from the middle.
        let base = centre - depth;
        let corners = [base, a - depth, b - depth, base];
        meshes[Surface::Floor as usize].free(
            corners,
            Vec3::Y,
            corners.map(|p| floor_uv(p, centre)),
            [Vec3::ZERO; 4],
        );
    }
}

/// How bright a face is, from the way it faces: tops brightest, then the
/// south and east, undersides and the north and west darker.
pub(super) fn facing_shade(normal: Vec3) -> f32 {
    (0.62 + 0.38 * normal.y + 0.14 * normal.z + 0.07 * normal.x).clamp(0.3, 1.0)
}

/// The tint that makes the front of a step, facing `normal`, a fixed amount
/// darker than its tread, whichever way it faces.
pub(super) fn front_tint(normal: Vec3) -> Vec3 {
    Vec3::splat(TREAD_TINT * FRONT_SHADE / facing_shade(normal))
}
