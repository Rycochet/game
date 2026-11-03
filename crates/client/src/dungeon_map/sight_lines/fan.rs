//! The floor triangles from a viewer to its outline.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::super::CELL_SIZE;
use super::Vec2d;

/// Adds the triangles from a viewer to its outline, on the floor.
pub(super) fn add_fan(triangles: &mut Vec<[f32; 3]>, from: Vec2d, outline: &[Vec2d]) {
    let on_floor = |p: Vec2d| [p.x as f32 * CELL_SIZE, 0.0, p.y as f32 * CELL_SIZE];
    for i in 0..outline.len() {
        triangles.push(on_floor(from));
        triangles.push(on_floor(outline[i]));
        triangles.push(on_floor(outline[(i + 1) % outline.len()]));
    }
}

pub(super) fn fan_mesh(positions: Vec<[f32; 3]>) -> Mesh {
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32((0..count as u32).collect()))
}
