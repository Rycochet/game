//! Collecting quads into one mesh.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::super::CELL_SIZE;
use super::{MEMORY_SHADE, RAISED, RAISED_NORMAL};

/// Collects quads into one mesh.
#[derive(Default)]
pub struct Builder {
    pub positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    pub colours: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    /// The height of the ground each vertex stands on (raised wilderness
    /// cells), which tops are measured from (`RAISED`).
    bases: Vec<f32>,
}

impl Builder {
    /// A rectangle `2 × half_across` by `2 × half_up` (along `up`), with each
    /// corner's vertex colour from `colour_at` its position. The picture keeps
    /// its scale (one per cell), so small parts show part of it.
    pub(super) fn quad(
        &mut self,
        centre: Vec3,
        normal: Vec3,
        up: Vec3,
        (half_across, half_up): (f32, f32),
        colour_at: impl Fn(Vec3) -> Vec3,
    ) {
        // "Right" makes the corners run anticlockwise seen from the front.
        let right = up.cross(normal);
        let (r, u) = (right * half_across, up * half_up);
        let (us, vs) = (2.0 * half_across / CELL_SIZE, 2.0 * half_up / CELL_SIZE);
        let first = self.positions.len() as u32;
        let corners = [
            (centre - r - u, [0.0, vs]),
            (centre + r - u, [us, vs]),
            (centre + r + u, [us, 0.0]),
            (centre - r + u, [0.0, 0.0]),
        ];
        for (position, uv) in corners {
            let lit = colour_at(position);
            self.positions.push(position.to_array());
            self.normals.push(normal.to_array());
            self.uvs.push(uv);
            self.colours.push([lit.x, lit.y, lit.z, 1.0]);
            self.bases.push(0.0);
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
    }

    /// Any four-cornered surface, with its picture coordinates and vertex
    /// colours. The corners go round it either way: `normal` says which side
    /// is the front.
    pub fn free(
        &mut self,
        positions: [Vec3; 4],
        normal: Vec3,
        uvs: [[f32; 2]; 4],
        colours: [Vec3; 4],
    ) {
        self.faded(positions, normal, uvs, colours, [1.0; 4]);
    }

    /// A surface as `free`, with an opacity at each corner (which only shows
    /// on a blended material).
    pub fn faded(
        &mut self,
        mut positions: [Vec3; 4],
        normal: Vec3,
        mut uvs: [[f32; 2]; 4],
        mut colours: [Vec3; 4],
        mut opacity: [f32; 4],
    ) {
        if (positions[1] - positions[0])
            .cross(positions[2] - positions[0])
            .dot(normal)
            < 0.0
        {
            positions.reverse();
            uvs.reverse();
            colours.reverse();
            opacity.reverse();
        }
        let first = self.positions.len() as u32;
        for i in 0..4 {
            let lit = colours[i];
            self.positions.push(positions[i].to_array());
            self.normals.push(normal.to_array());
            self.uvs.push(uvs[i]);
            self.colours.push([lit.x, lit.y, lit.z, opacity[i]]);
            self.bases.push(0.0);
        }
        self.indices
            .extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
    }

    /// Copies what was added since `(vertices, indices)` into `into` at the
    /// even shade of memory, keeping its opacity, without any raised top: the
    /// shader never cuts those (`RAISED` in `sight_mask.wesl`), so they would
    /// show twice.
    pub(super) fn copy_grey(&self, (vertices, indices): (usize, usize), into: &mut Builder) {
        let first = into.positions.len() as u32;
        for i in vertices..self.positions.len() {
            let [_, _, _, alpha] = self.colours[i];
            into.positions.push(self.positions[i]);
            into.normals.push(self.normals[i]);
            into.uvs.push(self.uvs[i]);
            into.bases.push(self.bases[i]);
            into.colours
                .push([MEMORY_SHADE, MEMORY_SHADE, MEMORY_SHADE, alpha]);
        }
        for triangle in self.indices[indices..].as_chunks::<3>().0 {
            let raised = triangle.iter().all(|&i| {
                let i = i as usize;
                self.positions[i][1] - self.bases[i] > RAISED && self.normals[i][1] > RAISED_NORMAL
            });
            if !raised {
                into.indices
                    .extend(triangle.iter().map(|&i| i - vertices as u32 + first));
            }
        }
    }

    /// Raises everything added since vertex `from` by `by`, onto ground that
    /// high.
    pub(in crate::dungeon_map) fn lift(&mut self, from: usize, by: f32) {
        for (position, base) in self.positions[from..]
            .iter_mut()
            .zip(&mut self.bases[from..])
        {
            position[1] += by;
            *base += by;
        }
    }

    /// The ground's height under each vertex reaches the shader as the second
    /// picture coordinates' first part (`sight_mask.wesl`).
    pub(in crate::dungeon_map) fn into_mesh(self) -> Mesh {
        let bases: Vec<[f32; 2]> = self.bases.iter().map(|&base| [base, 0.0]).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, bases)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}
