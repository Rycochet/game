//! A finished model as a mesh, its outline, and a picture of it.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;

use super::{
    OUTLINE_WIDTH, PICTURE_AMBIENT, PICTURE_LIGHT, PICTURE_MARGIN, PICTURE_SPAN, PoiModel, edge,
};

impl PoiModel {
    /// Makes the `Mesh`.
    pub fn mesh(&self) -> Mesh {
        let mut positions = Vec::with_capacity(self.corners.len() * 3);
        let mut normals = Vec::with_capacity(self.corners.len() * 3);
        let mut colours = Vec::with_capacity(self.corners.len() * 3);
        for (corners, colour) in self.corners.iter().zip(&self.colours) {
            let [a, b, c] = *corners;
            let normal = (b - a).cross(c - a).normalize_or_zero();
            let [red, green, blue] = *colour;
            let linear = Color::srgb_u8(red, green, blue).to_linear();
            for corner in corners {
                positions.push(corner.to_array());
                normals.push(normal.to_array());
                colours.push(linear.to_f32_array());
            }
        }
        let indices = (0..positions.len() as u32).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
    }

    /// Its outline when hovered: the same triangles pushed out along the
    /// normals, for drawing from the inside as a black shell round the model.
    /// Corners in the same place move together, so the shell has no gaps.
    pub fn outline_mesh(&self) -> Mesh {
        let key = |corner: Vec3| (corner * 1000.0).as_ivec3().to_array();
        let mut outwards: HashMap<[i32; 3], Vec3> = HashMap::new();
        for [a, b, c] in &self.corners {
            let normal = (*b - *a).cross(*c - *a).normalize_or_zero();
            for corner in [*a, *b, *c] {
                *outwards.entry(key(corner)).or_default() += normal;
            }
        }
        let positions: Vec<[f32; 3]> = self
            .corners
            .iter()
            .flatten()
            .map(|&corner| {
                let out = outwards[&key(corner)].normalize_or_zero();
                (corner + out * OUTLINE_WIDTH).to_array()
            })
            .collect();
        let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
        let indices = (0..positions.len() as u32).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
    }

    /// The model seen from the south-east from above, `size` pixels square, as
    /// RGBA with a see-through background: an isometric picture for the maps.
    pub fn picture(&self, size: u32) -> Vec<u8> {
        // Screen axes and the direction towards the viewer.
        let right = Vec3::new(1.0, 0.0, -1.0).normalize();
        let down = Vec3::new(1.0, -1.2, 1.0).normalize();
        let towards = right.cross(down).normalize();
        let towards = if towards.y < 0.0 { -towards } else { towards };
        let above_ground = |corner: &Vec3| corner.y >= -0.01;
        let shown = self.corners.iter().flatten().filter(|c| above_ground(c));
        let (mut min, mut max) = (Vec2::MAX, Vec2::MIN);
        for corner in shown {
            let at = Vec2::new(corner.dot(right), corner.dot(down));
            (min, max) = (min.min(at), max.max(at));
        }
        let span = (max - min).max_element().max(PICTURE_SPAN);
        let margin = size as f32 * PICTURE_MARGIN;
        let room = size as f32 - 2.0 * margin;
        let scale = room / span;
        // Centred, with the feet near the bottom.
        let offset = Vec2::new(
            (size as f32 - (max.x - min.x) * scale) / 2.0 - min.x * scale,
            size as f32 - margin - max.y * scale,
        );
        let light = Vec3::from(PICTURE_LIGHT).normalize();

        let side = size as usize;
        let mut pixels = vec![0; side * side * 4];
        let mut nearest = vec![f32::MIN; side * side];
        for (corners, colour) in self.corners.iter().zip(&self.colours) {
            let [a, b, c] = *corners;
            let normal = (b - a).cross(c - a).normalize_or_zero();
            if normal.dot(towards) <= 0.0 {
                continue;
            }
            let shade = (PICTURE_AMBIENT + normal.dot(light).max(0.0)).min(1.2);
            let on_screen = |p: Vec3| Vec2::new(p.dot(right), p.dot(down)) * scale + offset;
            let (pa, pb, pc) = (on_screen(a), on_screen(b), on_screen(c));
            let area = edge(pa, pb, pc);
            if area == 0.0 {
                continue;
            }
            let low = pa.min(pb).min(pc).floor().max(Vec2::ZERO);
            let high = pa
                .max(pb)
                .max(pc)
                .ceil()
                .min(Vec2::splat(size as f32 - 1.0));
            for y in low.y as usize..=high.y as usize {
                for x in low.x as usize..=high.x as usize {
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    let (wa, wb, wc) = (
                        edge(pb, pc, p) / area,
                        edge(pc, pa, p) / area,
                        edge(pa, pb, p) / area,
                    );
                    if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                        continue;
                    }
                    let point = a * wa + b * wb + c * wc;
                    // What is underground never shows.
                    let depth = point.dot(towards);
                    if point.y < 0.0 || depth <= nearest[y * side + x] {
                        continue;
                    }
                    nearest[y * side + x] = depth;
                    let at = (y * side + x) * 4;
                    for channel in 0..3 {
                        let lit = f32::from(colour[channel]) * shade;
                        pixels[at + channel] = lit.clamp(0.0, 255.0) as u8;
                    }
                    pixels[at + 3] = 255;
                }
            }
        }
        pixels
    }
}
