//! Wall faces below a liquid's surface.

use bevy::prelude::*;

use super::super::CELL_SIZE;
use super::super::liquid::{self, FULL_DEPTH, outline_at};
use super::super::mesh::Surface;
use super::{Layer, Layers, SIDEWAYS_SHADE, Tile, half_thickness, reach_at};

impl<'a> Tile<'a> {
    /// Where a face stands in liquid, carries it on down to the bed: straight
    /// down from its foot, whatever curve it has above. The picture starts
    /// again from its top at the floor, so it repeats and isn't stretched.
    pub(super) fn face_under_liquid(
        &self,
        layers: &mut Layers,
        layer: Layer,
        quarter: (f32, f32),
        (facing, running): (usize, usize),
        reach: [f32; 4],
    ) {
        let Some(outline) = outline_at(self.scene.level, (self.x, self.y), quarter) else {
            return;
        };
        let (plane, from, to) = self.face_at(facing, running, reach, 0.0);
        let along_east = facing.is_multiple_of(2);
        let Some((from, to)) = outline.span_along(along_east, plane, (from, to)) else {
            return;
        };
        let (sx, sz) = quarter;
        let (normal, turned) = if along_east {
            (Vec3::Z * sz, 1.0)
        } else {
            (Vec3::X * sx, SIDEWAYS_SHADE)
        };
        let corner = |along: f32, depth: f32| {
            let (east, south) = if along_east {
                (sx * along, sz * plane)
            } else {
                (sx * plane, sz * along)
            };
            let across = if along_east { east } else { south };
            (
                self.centre + Vec3::new(east, -depth, south),
                [across / CELL_SIZE + 0.5, depth / CELL_SIZE],
                turned * liquid::depth_shade(depth),
            )
        };
        let corners = [
            corner(from, 0.0),
            corner(to, 0.0),
            corner(to, FULL_DEPTH),
            corner(from, FULL_DEPTH),
        ];
        self.add(layers, layer, Surface::Wall, normal, corners);
    }

    /// Where a wall's face stands from the cell's centre, and between which
    /// distances along it, at a height. It faces towards side `facing` and
    /// runs out towards side `running`.
    pub(super) fn face_at(
        &self,
        facing: usize,
        running: usize,
        reach: [f32; 4],
        height: f32,
    ) -> (f32, f32, f32) {
        let shape = self.scene.shape;
        let half = half_thickness(shape, height);
        let to = reach_at(shape, reach[running], height);
        if self.joins[facing] {
            (half, half, to)
        } else {
            (reach_at(shape, reach[facing], height), 0.0, to)
        }
    }
}
