//! A door or window's frame.

use bevy::prelude::*;

use super::super::WALL_HEIGHT;
use super::super::mesh::{Surface, faces_camera};
use super::super::theme::Shape;
use super::{
    FRAME_SHADE, HALF_CELL, Layer, Layers, OPENING_HALF, Opening, RISING_SHADE, TOP_SHADE, Tile,
    face_uv, flat_uv, half_thickness, height_shade, heights,
};

impl<'a> Tile<'a> {
    /// The insides of the hole (the sides, its top and the window's sill) and
    /// the top of the wall.
    pub(super) fn opening_frame(&self, layers: &mut Layers, layer: Layer, opening: &Opening) {
        let shape = opening.shape;
        let surface = match shape {
            Shape::Built => Surface::Door,
            Shape::Natural => Surface::Wall,
        };
        let centre = self.centre;
        let across = |a: f32, height: f32, out: f32| {
            centre + opening.along * a + Vec3::Y * height + opening.out * out
        };
        // Skipped when turned away from the camera, except in the see-through
        // layer, where the back shows too.
        let wanted = |middle: Vec3, normal: Vec3| {
            layer == Layer::SeeThrough || faces_camera(middle, normal, self.scene.camera)
        };
        // The sides of the hole.
        for sign in [1.0f32, -1.0] {
            let a = sign * OPENING_HALF;
            let normal = opening.along * -sign;
            let bottom = opening.bottom();
            let cuts = heights(shape, bottom, opening.top(a));
            for band in cuts.windows(2) {
                let corner = |height: f32, out: f32| {
                    let half = half_thickness(shape, height) * out;
                    (
                        across(a, height, half),
                        face_uv(half, height),
                        FRAME_SHADE * height_shade(shape, height),
                    )
                };
                let corners = [
                    corner(band[0], -1.0),
                    corner(band[0], 1.0),
                    corner(band[1], 1.0),
                    corner(band[1], -1.0),
                ];
                if wanted(corners[0].0, normal) {
                    self.add(layers, layer, surface, normal, corners);
                }
            }
        }
        // Over the hole: flat, or curved for an arch.
        let columns: Vec<f32> = opening
            .columns()
            .into_iter()
            .filter(|a| a.abs() <= OPENING_HALF)
            .collect();
        for pair in columns.windows(2) {
            let corner = |a: f32, out: f32| {
                let height = opening.top(a);
                let half = half_thickness(shape, height) * out;
                (
                    across(a, height, half),
                    flat_uv(a, half),
                    FRAME_SHADE * RISING_SHADE,
                )
            };
            let corners = [
                corner(pair[0], -1.0),
                corner(pair[1], -1.0),
                corner(pair[1], 1.0),
                corner(pair[0], 1.0),
            ];
            if wanted(corners[0].0, Vec3::NEG_Y) {
                self.add(layers, layer, surface, Vec3::NEG_Y, corners);
            }
        }
        if opening.window {
            let height = opening.bottom();
            let half = half_thickness(shape, height);
            let corner = |a: f32, out: f32| (across(a, height, out), flat_uv(a, out), FRAME_SHADE);
            let corners = [
                corner(-OPENING_HALF, -half),
                corner(OPENING_HALF, -half),
                corner(OPENING_HALF, half),
                corner(-OPENING_HALF, half),
            ];
            self.add(layers, layer, surface, Vec3::Y, corners);
        }
        let half = half_thickness(shape, WALL_HEIGHT);
        let corner = |a: f32, out: f32| (across(a, WALL_HEIGHT, out), flat_uv(a, out), TOP_SHADE);
        let corners = [
            corner(-HALF_CELL, -half),
            corner(HALF_CELL, -half),
            corner(HALF_CELL, half),
            corner(-HALF_CELL, half),
        ];
        self.add(layers, layer, Surface::Wall, Vec3::Y, corners);
    }
}
