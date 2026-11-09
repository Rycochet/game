//! The terrain drawn in square chunks of the grid, each built once and kept
//! while it is in reach: stepping or turning builds none, as the window is
//! cut out of them by the terrain's shaders (`terrain.wesl`, and
//! `terrain_prepass.wesl` for shadows), not on the CPU.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::{TerrainGrid, ViewSquare};

/// Grid points along a chunk's side. Bigger chunks are fewer to draw (each is
/// a draw call) but cost more to build as the player crosses into new ones.
pub const CHUNK: i64 = 32;

/// A box of grid points by world position: first and last corners, both
/// included.
pub type GridArea = ((i64, i64), (i64, i64));

/// The chunks (by world position: the chunk at `(cx, cz)` starts at grid point
/// `(cx × CHUNK, cz × CHUNK)`) any part of the window `square` round `centre`
/// may show.
pub fn chunks_needed(square: ViewSquare, centre: (i64, i64)) -> Vec<(i64, i64)> {
    // A point to spare on each side, for edge faces and rounding.
    let half = square.half + 1.0;
    let corners = [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)]
        .map(|(u, v)| (square.right * u + square.towards * v) * half);
    let (low, high) = corners
        .iter()
        .fold((Vec2::MAX, Vec2::MIN), |(low, high), c| {
            (low.min(*c), high.max(*c))
        });
    let first = |at: f32, centre: i64| (centre + at.floor() as i64).div_euclid(CHUNK);
    let (from_x, to_x) = (first(low.x, centre.0), first(high.x, centre.0));
    let (from_z, to_z) = (first(low.y, centre.1), first(high.y, centre.1));
    let mut needed = Vec::new();
    for cz in from_z..=to_z {
        for cx in from_x..=to_x {
            let start = Vec2::new(
                (cx * CHUNK - centre.0) as f32,
                (cz * CHUNK - centre.1) as f32,
            );
            if overlaps(start, half, square) {
                needed.push((cx, cz));
            }
        }
    }
    needed
}

/// Whether the chunk starting at `start` (relative to the window's centre)
/// overlaps the turned square of `half`: separated along neither of the
/// square's axes (the chunk's own are ruled out by `chunks_needed`'s box).
fn overlaps(start: Vec2, half: f32, square: ViewSquare) -> bool {
    let side = CHUNK as f32;
    let corners =
        [(0.0, 0.0), (side, 0.0), (0.0, side), (side, side)].map(|(x, z)| start + Vec2::new(x, z));
    [square.right, square.towards].iter().all(|axis| {
        let (low, high) = corners.iter().fold((f32::MAX, f32::MIN), |(low, high), c| {
            let along = c.dot(*axis);
            (low.min(along), high.max(along))
        });
        high >= -half && low <= half
    })
}

/// The grid points `chunks` need, both corners included: what
/// `TerrainGrid::fill` must have worked out before they are built.
pub fn chunks_area(chunks: &[(i64, i64)]) -> GridArea {
    let (mut low, mut high) = ((i64::MAX, i64::MAX), (i64::MIN, i64::MIN));
    for &(cx, cz) in chunks {
        low = (low.0.min(cx * CHUNK), low.1.min(cz * CHUNK));
        high = (high.0.max((cx + 1) * CHUNK), high.1.max((cz + 1) * CHUNK));
    }
    (low, high)
}

/// One chunk's mesh, whole triangles from the grid, relative to its first
/// point (which its entity is placed at). Every point must be worked out.
pub fn build_chunk(grid: &TerrainGrid, (cx, cz): (i64, i64)) -> Mesh {
    let side = (CHUNK + 1) as usize;
    let mut positions = Vec::with_capacity(side * side);
    let mut normals = Vec::with_capacity(side * side);
    let mut colours = Vec::with_capacity(side * side);
    // Two pairs of numbers, in the mesh's texture coordinate slots, as those
    // reach the shader without a vertex shader of our own.
    let mut growing = Vec::with_capacity(side * side);
    let mut kinds = Vec::with_capacity(side * side);
    for dz in 0..=CHUNK {
        for dx in 0..=CHUNK {
            let (x, z) = (cx * CHUNK + dx, cz * CHUNK + dz);
            let i = grid.slot(x - grid.centre.0, z - grid.centre.1);
            positions.push([dx as f32, grid.shown[i], dz as f32]);
            normals.push(grid.normals[i]);
            colours.push(grid.colours[i]);
            growing.push(grid.covers[i].xy().to_array());
            kinds.push(grid.covers[i].zw().to_array());
        }
    }
    let at = |dx: usize, dz: usize| (dz * side + dx) as u32;
    let mut indices = Vec::with_capacity(CHUNK as usize * CHUNK as usize * 6);
    for dz in 0..CHUNK as usize {
        for dx in 0..CHUNK as usize {
            // Two triangles per square, wound anticlockwise seen from above,
            // split along the north-east to south-west diagonal as
            // `TerrainGrid::height_at` and the edge faces expect.
            indices.extend_from_slice(&[
                at(dx, dz),
                at(dx, dz + 1),
                at(dx + 1, dz),
                at(dx + 1, dz),
                at(dx, dz + 1),
                at(dx + 1, dz + 1),
            ]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, growing)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, kinds)
    .with_inserted_indices(Indices::U32(indices))
}
