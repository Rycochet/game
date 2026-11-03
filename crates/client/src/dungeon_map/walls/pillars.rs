//! Natural windows: lines of columns, or stalagmites below stalactites.

use bevy::prelude::*;

use super::super::WALL_HEIGHT;
use super::super::mesh::{Surface, faces_camera};
use super::super::theme::Shape;
use super::{
    Layer, Layers, Opening, PILLAR_FOOT_HALF, PILLAR_HALF, PILLAR_SPACING, PILLARS, SIDEWAYS_SHADE,
    Tile, face_uv, height_shade, heights, noise,
};

impl<'a> Tile<'a> {
    /// A natural window: a line of columns, or of stalagmites below
    /// stalactites, as chosen by the cell's position.
    pub(super) fn build_pillars(&self, layers: &mut Layers, layer: Layer, opening: &Opening) {
        for i in 0..PILLARS {
            let a = (i - PILLARS / 2) as f32 * PILLAR_SPACING;
            let foot = self.centre + opening.along * a;
            let kind = noise(self.x, self.y, i);
            if kind < 0.2 {
                // A whole column, narrowest half way up.
                let width = |height: f32| {
                    let from_middle = 2.0 * height / WALL_HEIGHT - 1.0;
                    PILLAR_HALF + (PILLAR_FOOT_HALF - PILLAR_HALF) * from_middle * from_middle
                };
                self.pillar(layers, layer, opening, foot, (0.0, WALL_HEIGHT), &width);
            } else {
                // Each comes to a point, with a gap between them.
                let gap = noise(self.y, self.x, i) * 0.12 + 0.5;
                let low = WALL_HEIGHT * (gap - 0.1);
                let high = WALL_HEIGHT * (gap + 0.1);
                let rising = |height: f32| PILLAR_FOOT_HALF * (1.0 - height / low).max(0.0);
                let hanging = |height: f32| {
                    PILLAR_FOOT_HALF * ((height - high) / (WALL_HEIGHT - high)).max(0.0)
                };
                self.pillar(layers, layer, opening, foot, (0.0, low), &rising);
                self.pillar(layers, layer, opening, foot, (high, WALL_HEIGHT), &hanging);
            }
        }
    }

    /// One four-sided column between two heights, `width` being half its
    /// width at a height.
    fn pillar(
        &self,
        layers: &mut Layers,
        layer: Layer,
        opening: &Opening,
        foot: Vec3,
        (low, high): (f32, f32),
        width: &dyn Fn(f32) -> f32,
    ) {
        let cuts = heights(Shape::Natural, low, high);
        let (along, out) = (opening.along, opening.out);
        for (normal, right, shade) in [
            (out, along, 1.0),
            (-out, along, 1.0),
            (along, out, SIDEWAYS_SHADE),
            (-along, out, SIDEWAYS_SHADE),
        ] {
            let middle = foot + normal * PILLAR_HALF + Vec3::Y * ((low + high) / 2.0);
            if layer != Layer::SeeThrough && !faces_camera(middle, normal, self.scene.camera) {
                continue;
            }
            for band in cuts.windows(2) {
                let corner = |height: f32, sign: f32| {
                    let w = width(height);
                    let across = (foot - self.centre).dot(right) + sign * w;
                    (
                        foot + normal * w + right * (sign * w) + Vec3::Y * height,
                        face_uv(across, height),
                        shade * height_shade(Shape::Natural, height),
                    )
                };
                let corners = [
                    corner(band[0], -1.0),
                    corner(band[0], 1.0),
                    corner(band[1], 1.0),
                    corner(band[1], -1.0),
                ];
                self.add(layers, layer, Surface::Wall, normal, corners);
            }
        }
    }
}
