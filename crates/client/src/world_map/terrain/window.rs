//! The terrain mesh for the window on show.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::{TerrainGrid, ViewSquare, clip};

/// The terrain mesh: the window `square`, cut out of the grid.
pub fn build_window(grid: &TerrainGrid, square: ViewSquare) -> Mesh {
    let (reach_x, reach_z) = square.reach();
    let width = (2 * reach_x + 1) as usize;
    let local = |dx: i64, dz: i64| (dz + reach_z) as usize * width + (dx + reach_x) as usize;

    // Which points in the box are inside the window.
    let mut inside = Vec::with_capacity(width * (2 * reach_z + 1) as usize);
    for dz in -reach_z..=reach_z {
        for dx in -reach_x..=reach_x {
            let point = Vec2::new(dx as f32, dz as f32);
            inside.push(square.contains(point));
        }
    }
    // Squares near the window's corners need clipping even when none of
    // their points are inside (a corner can poke into a triangle).
    let corners =
        [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)].map(|(u, v)| square.at(u, v));
    let near_corner = |x: i64, z: i64| {
        corners
            .iter()
            .any(|c| (c.x - x as f32).abs() <= 2.0 && (c.y - z as f32).abs() <= 2.0)
    };

    let expected = (4.0 * square.half * square.half) as usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(expected);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(expected);
    let mut colours: Vec<[f32; 4]> = Vec::with_capacity(expected);
    // Two pairs of numbers, in the mesh's texture coordinate slots, as those
    // reach the shader without a vertex shader of our own.
    let mut growing: Vec<[f32; 2]> = Vec::with_capacity(expected);
    let mut kinds: Vec<[f32; 2]> = Vec::with_capacity(expected);
    let mut indices: Vec<u32> = Vec::with_capacity(expected * 6);
    // Points already added to the mesh (by box position), so whole
    // triangles share them.
    let mut added = vec![u32::MAX; inside.len()];

    for z in -reach_z..reach_z {
        for x in -reach_x..reach_x {
            // Two triangles per square, wound anticlockwise seen from above.
            for triangle in [
                [(x, z), (x, z + 1), (x + 1, z)],
                [(x + 1, z), (x, z + 1), (x + 1, z + 1)],
            ] {
                let count = triangle
                    .iter()
                    .filter(|&&(dx, dz)| inside[local(dx, dz)])
                    .count();
                if count == 3 {
                    for (dx, dz) in triangle {
                        let j = local(dx, dz);
                        if added[j] == u32::MAX {
                            let i = grid.slot(dx, dz);
                            positions.push([dx as f32, grid.shown[i], dz as f32]);
                            normals.push(grid.normals[i]);
                            colours.push(grid.colours[i]);
                            growing.push(grid.covers[i].xy().to_array());
                            kinds.push(grid.covers[i].zw().to_array());
                            added[j] = positions.len() as u32 - 1;
                        }
                        indices.push(added[j]);
                    }
                    continue;
                }
                if count == 0 && !near_corner(x, z) {
                    continue;
                }
                // Crosses the border: cut off the parts outside each side in
                // turn, then fan the remaining polygon into triangles.
                let clipped = clip(&triangle.map(|(dx, dz)| grid.corner(dx, dz, square)));
                let first = positions.len() as u32;
                for c in &clipped {
                    positions.push(c.position.to_array());
                    normals.push(c.normal.normalize().to_array());
                    colours.push(c.colour.to_array());
                    growing.push(c.cover.xy().to_array());
                    kinds.push(c.cover.zw().to_array());
                }
                for k in 1..clipped.len().saturating_sub(1) as u32 {
                    indices.extend_from_slice(&[first, first + k, first + k + 1]);
                }
            }
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
