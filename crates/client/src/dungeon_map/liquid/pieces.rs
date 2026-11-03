//! The flat pieces, surface, bed and sides a quarter of liquid is drawn with.

use bevy::prelude::*;

use super::super::CELL_SIZE;
use super::super::mesh::Surface;
use super::super::walls::Layers;
use super::{Bed, Point, Quarter, SURFACE_DROP, bed_shade, depth_shade};

impl Quarter<'_> {
    /// A flat piece at a height, shaded per corner.
    pub(super) fn flat(
        &self,
        layers: &mut Layers,
        surface: Surface,
        corners: [Point; 4],
        height: f32,
        shade: &dyn Fn(Point) -> f32,
    ) {
        let positions = corners.map(|point| self.place(point, height));
        let colours = corners.map(|point| self.colour_at(self.place(point, height), shade(point)));
        layers.builder(self.layer, surface).free(
            positions,
            Vec3::Y,
            corners.map(|point| self.uv(point)),
            colours,
        );
    }

    pub(super) fn surface(&self, layers: &mut Layers, corners: [Point; 4]) {
        self.flat(layers, self.liquid, corners, -SURFACE_DROP, &|_| 1.0);
    }

    pub(super) fn bed(&self, layers: &mut Layers, corners: [Point; 4], bed: Bed) {
        let positions = corners.map(|point| self.place(point, -bed.depth(point)));
        let colours = corners.map(|point| {
            let depth = bed.depth(point);
            self.colour_at(self.place(point, -depth), bed_shade(depth))
        });
        layers.builder(self.layer, self.floor).free(
            positions,
            Vec3::Y,
            corners.map(|point| self.uv(point)),
            colours,
        );
    }

    /// The side of the channel along a piece of shore, facing the liquid at
    /// `towards`. It keeps the picture's scale, starting again from the top at
    /// the floor, so it is never stretched.
    pub(super) fn side(
        &self,
        layers: &mut Layers,
        surface: Surface,
        (a, b): (Point, Point),
        towards: Point,
        bed: Bed,
    ) {
        let (sx, sz) = self.quarter;
        let length = Vec2::new(b.0 - a.0, b.1 - a.1).length();
        let inwards = Vec3::new(sx * (towards.0 - a.0), 0.0, sz * (towards.1 - a.1)).normalize();
        let start = (a.0 + a.1) / CELL_SIZE;
        let u1 = start + length / CELL_SIZE;
        let (depth_a, depth_b) = (bed.depth(a), bed.depth(b));
        let positions = [
            self.place(a, 0.0),
            self.place(b, 0.0),
            self.place(b, -depth_b),
            self.place(a, -depth_a),
        ];
        let uvs = [
            [start, 0.0],
            [u1, 0.0],
            [u1, depth_b / CELL_SIZE],
            [start, depth_a / CELL_SIZE],
        ];
        let shades = [
            depth_shade(0.0),
            depth_shade(0.0),
            depth_shade(depth_b),
            depth_shade(depth_a),
        ];
        let colours = std::array::from_fn(|i| self.colour_at(positions[i], shades[i]));
        layers
            .builder(self.layer, surface)
            .free(positions, inwards, uvs, colours);
    }
}
