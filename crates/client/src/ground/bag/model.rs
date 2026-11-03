//! What a shape is made of, with smooth normals, grown and greyed copies.

use super::MEMORY_SHADE;

/// What a shape is made of, ready for a Bevy mesh.
#[derive(Default)]
pub(in crate::ground) struct Model {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colours: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl Model {
    pub(super) fn add_triangle(&mut self, a: usize, b: usize, c: usize) {
        self.indices.extend([a as u32, b as u32, c as u32]);
    }

    /// Normals from the triangles touching each point, added to what any have.
    pub(super) fn smooth_normals(&mut self, first_point: usize, first_index: usize) {
        for point in first_point..self.positions.len() {
            self.normals[point] = [0.0; 3];
        }
        let triangles: Vec<[usize; 3]> = self.indices[first_index..]
            .chunks(3)
            .map(|triangle| {
                [
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                ]
            })
            .collect();
        for [a, b, c] in triangles {
            let [pa, pb, pc] = [a, b, c].map(|i| self.positions[i]);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            for i in [a, b, c] {
                let normal = &mut self.normals[i];
                *normal = [
                    normal[0] + cross[0],
                    normal[1] + cross[1],
                    normal[2] + cross[2],
                ];
            }
        }
        for normal in &mut self.normals[first_point..] {
            let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                .sqrt()
                .max(1e-6);
            *normal = normal.map(|axis| axis / length);
        }
    }

    /// The same shape, grown along its normals: seen from inside, what stands out past
    /// the bag is its outline.
    pub fn grown(&self, by: f32) -> Self {
        let mut grown = Self {
            positions: Vec::with_capacity(self.positions.len()),
            normals: self.normals.clone(),
            uvs: self.uvs.clone(),
            colours: self.colours.clone(),
            indices: self.indices.clone(),
        };
        for (position, normal) in self.positions.iter().zip(&self.normals) {
            grown.positions.push([
                position[0] + normal[0] * by,
                position[1] + normal[1] * by,
                position[2] + normal[2] * by,
            ]);
        }
        grown
    }

    /// The same shape, grey, for memory.
    pub fn greyed(&self) -> Self {
        let colours = self.colours.iter().map(|colour| {
            let grey = (0.3 * colour[0] + 0.59 * colour[1] + 0.11 * colour[2]) * MEMORY_SHADE;
            [grey, grey, grey, 1.0]
        });
        Self {
            positions: self.positions.clone(),
            normals: self.normals.clone(),
            uvs: self.uvs.clone(),
            colours: colours.collect(),
            indices: self.indices.clone(),
        }
    }
}
