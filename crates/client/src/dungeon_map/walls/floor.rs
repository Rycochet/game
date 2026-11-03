//! The floor under a wall cell, fading into the wall.

use bevy::prelude::*;

use super::super::mesh::Surface;
use super::super::theme::Shape;
use super::{
    BLEND_LIFT, BLEND_STEP, BLEND_WIDTH, Layer, Layers, Patch, Tile, cuts, flat_uv, floor_near,
};

impl<'a> Tile<'a> {
    /// The floor in a quarter of the cell, like the open cell nearest it, up
    /// to the wall's `foot`. In natural places the wall's picture fades out
    /// across it, starting from `fade_from`.
    pub(super) fn floor_quarter(
        &self,
        layers: &mut Layers,
        layer: Layer,
        (sx, sz): (f32, f32),
        foot: &[Patch],
        fade_from: &[Patch],
    ) {
        let surface = floor_near(self.scene.level, self.around_corner((sx, sz)));
        let point = |east: f32, south: f32| {
            let (e, s) = (sx * east, sz * south);
            (self.centre + Vec3::new(e, 0.0, s), flat_uv(e, s), 1.0)
        };
        let all = [foot, fade_from].concat();
        let across = cuts(&all, |patch| patch.east);
        let down = cuts(&all, |patch| patch.south);
        for x in across.windows(2) {
            for z in down.windows(2) {
                let middle = ((x[0] + x[1]) / 2.0, (z[0] + z[1]) / 2.0);
                if foot.iter().any(|p| p.contains(middle.0, middle.1)) {
                    continue;
                }
                let corners = [
                    point(x[0], z[0]),
                    point(x[1], z[0]),
                    point(x[1], z[1]),
                    point(x[0], z[1]),
                ];
                self.add(layers, layer, surface, Vec3::Y, corners);
                if self.scene.shape == Shape::Natural {
                    self.fade_wall(layers, layer, (sx, sz), (x, z), fade_from);
                }
            }
        }
    }

    /// The wall's picture over a piece of floor, fading out with the distance
    /// from `from`.
    fn fade_wall(
        &self,
        layers: &mut Layers,
        layer: Layer,
        (sx, sz): (f32, f32),
        (x, z): (&[f32], &[f32]),
        from: &[Patch],
    ) {
        let steps = |low: f32, high: f32| ((high - low) / BLEND_STEP).ceil().max(1.0) as usize;
        let (across, down) = (steps(x[0], x[1]), steps(z[0], z[1]));
        let at = |low: f32, high: f32, i: usize, count: usize| {
            low + (high - low) * i as f32 / count as f32
        };
        let opacity = |east: f32, south: f32| {
            from.iter()
                .map(|patch| 1.0 - patch.distance_to(east, south) / BLEND_WIDTH)
                .fold(0.0, f32::max)
        };
        let vertex = |i: usize, j: usize| {
            let (east, south) = (at(x[0], x[1], i, across), at(z[0], z[1], j, down));
            let (e, s) = (sx * east, sz * south);
            let position = self.centre + Vec3::new(e, BLEND_LIFT, s);
            (position, flat_uv(e, s), opacity(east, south))
        };
        for i in 0..across {
            for j in 0..down {
                let corners = [
                    vertex(i, j),
                    vertex(i + 1, j),
                    vertex(i + 1, j + 1),
                    vertex(i, j + 1),
                ];
                let fades = corners.map(|(_, _, fade)| fade);
                if fades.iter().all(|&fade| fade <= 0.0) {
                    continue;
                }
                let colours = corners.map(|(position, _, _)| self.colour(layer, position, 1.0));
                layers.builder(layer, Surface::WallBlend).faded(
                    corners.map(|(position, _, _)| position),
                    Vec3::Y,
                    corners.map(|(_, uv, _)| uv),
                    colours,
                    fades,
                );
            }
        }
    }
}
