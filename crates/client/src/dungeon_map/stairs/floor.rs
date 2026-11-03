//! The floor of a stairs cell, with its hole, and the light under a ladder.

use bevy::prelude::*;
use std::f32::consts::TAU;

use super::super::CELL_SIZE;
use super::super::mesh::{Builder, Surface};
use super::{Hole, LADDER_LIGHT_TINT, ROUND_RADIUS, ROUND_SIDES, SQUARE_HALF};

/// A point on the cell's floor to picture coordinates.
pub(super) fn floor_uv(point: Vec3, centre: Vec3) -> [f32; 2] {
    let at = (point - centre) / CELL_SIZE + Vec3::splat(0.5);
    [at.x.clamp(0.0, 1.0), at.z.clamp(0.0, 1.0)]
}

/// The floor of a whole cell.
pub(super) fn floor_whole(
    meshes: &mut [Builder],
    floor: Surface,
    centre: Vec3,
    colour: &impl Fn(Vec3) -> Vec3,
) {
    let half = CELL_SIZE / 2.0;
    let corners = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
        .map(|(x, z)| centre + Vec3::new(x * half, 0.0, z * half));
    meshes[floor as usize].free(
        corners,
        Vec3::Y,
        corners.map(|p| floor_uv(p, centre)),
        corners.map(colour),
    );
}

/// The corners of a hole, going round, as offsets from its centre.
pub(super) fn hole_outline(hole: Hole) -> Vec<Vec3> {
    match hole {
        Hole::Round => (0..ROUND_SIDES)
            .map(|i| {
                let angle = i as f32 / ROUND_SIDES as f32 * TAU;
                Vec3::new(angle.cos(), 0.0, angle.sin()) * ROUND_RADIUS
            })
            .collect(),
        Hole::Square => [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
            .map(|(x, z)| Vec3::new(x, 0.0, z) * SQUARE_HALF)
            .to_vec(),
    }
}

/// The floor of a cell with a hole in it: pieces from each side of the hole
/// out to the edge of the cell.
pub(super) fn floor_with_hole(
    meshes: &mut [Builder],
    floor: Surface,
    centre: Vec3,
    hole: Hole,
    colour: &impl Fn(Vec3) -> Vec3,
) {
    let half = CELL_SIZE / 2.0;
    // Straight out from the hole's corner to the edge of the cell.
    let to_edge = |p: Vec3| p / p.x.abs().max(p.z.abs()) * half;
    let outline = hole_outline(hole);
    for (i, &a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        let corners = [a, b, to_edge(b), to_edge(a)].map(|p| centre + p);
        meshes[floor as usize].free(
            corners,
            Vec3::Y,
            corners.map(|p| floor_uv(p, centre)),
            corners.map(colour),
        );
    }
}

/// Fills the round hole `floor_with_hole` leaves with brighter floor.
pub(super) fn light_under_ladder(
    meshes: &mut [Builder],
    floor: Surface,
    centre: Vec3,
    colour: &impl Fn(Vec3) -> Vec3,
) {
    let outline = hole_outline(Hole::Round);
    for (i, &a) in outline.iter().enumerate() {
        let b = outline[(i + 1) % outline.len()];
        // A triangle, as a quad with two corners the same.
        let corners = [centre, centre + a, centre + b, centre + b];
        meshes[floor as usize].free(
            corners,
            Vec3::Y,
            corners.map(|p| floor_uv(p, centre)),
            corners.map(|p| colour(p) * LADDER_LIGHT_TINT),
        );
    }
}
