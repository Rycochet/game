//! The wall round a door or window, and its sides.

use bevy::prelude::*;

use super::super::mesh::{Surface, View, WallFace, faces_camera};
use super::super::theme::Shape;
use super::super::{CELL_SIZE, WALL_HEIGHT};
use super::{
    Layer, Layers, OPENING_HALF, Opening, SIDEWAYS_SHADE, Tile, WALL_HALF, face_uv, half_thickness,
    height_shade, heights,
};

impl<'a> Tile<'a> {
    /// The wall of a door or window: its two sides, each drawn on the layer
    /// it should be seen on, and the frame round the hole. A natural window
    /// is a line of columns instead.
    pub(super) fn build_opening_wall(
        &self,
        layers: &mut Layers,
        east_west: bool,
        window: bool,
        in_sight: bool,
    ) {
        let scene = self.scene;
        let (along, out) = if east_west {
            (Vec3::X, Vec3::Z)
        } else {
            (Vec3::Z, Vec3::X)
        };
        let side_middle = |side: f32| self.centre + out * side * WALL_HALF;
        let in_front = |view: &View, side: f32| {
            let viewer = Vec3::new(
                view.cell.0 as f32 * CELL_SIZE,
                0.0,
                view.cell.1 as f32 * CELL_SIZE,
            );
            (viewer - side_middle(side)).dot(out * side) > 0.0
        };
        let seeing = |view: &&View| view.sight.is_seen(self.x, self.y);
        // Someone standing in line with the wall sees both sides.
        let in_line = !scene
            .views
            .iter()
            .filter(seeing)
            .any(|view| in_front(view, 1.0) || in_front(view, -1.0));
        let faces = |side: f32| {
            let middle = side_middle(side) + Vec3::Y * (WALL_HEIGHT / 2.0);
            faces_camera(middle, out * side, scene.camera)
        };
        let seen =
            |side: f32| in_line || scene.views.iter().filter(seeing).any(|v| in_front(v, side));
        // As with walls, both sides look the same: see-through if either is
        // seen turned away from the camera.
        let see_through = [1.0, -1.0].iter().any(|&side| seen(side) && !faces(side));
        let side_layers = [1.0, -1.0].map(|side| {
            if in_sight {
                match (seen(side), see_through) {
                    (false, _) => None,
                    (true, false) => Some(Layer::Solid),
                    (true, true) => Some(Layer::SeeThrough),
                }
            } else {
                let faces = faces(side);
                faces.then_some(Layer::Memory)
            }
        });
        let Some(frame_layer) = [Layer::Solid, Layer::SeeThrough, Layer::Memory]
            .into_iter()
            .find(|&layer| side_layers.contains(&Some(layer)))
        else {
            return;
        };
        let opening = Opening {
            along,
            out,
            window,
            shape: scene.shape,
        };
        if scene.shape == Shape::Natural && window {
            self.build_pillars(layers, frame_layer, &opening);
            return;
        }
        for (side, layer) in [1.0, -1.0].into_iter().zip(side_layers) {
            if let Some(layer) = layer {
                self.opening_side(layers, layer, &opening, side);
            }
        }
        self.opening_frame(layers, frame_layer, &opening);
    }

    /// One side of a wall with a hole in it, cut into columns from one edge
    /// of the cell to the other.
    fn opening_side(&self, layers: &mut Layers, layer: Layer, opening: &Opening, side: f32) {
        let columns = opening.columns();
        let turned = if opening.along == Vec3::X {
            1.0
        } else {
            SIDEWAYS_SHADE
        };
        let shape = opening.shape;
        let normal = opening.out * side;
        for pair in columns.windows(2) {
            let (a0, a1) = (pair[0], pair[1]);
            // Whether this column is across the hole, not beside it.
            let hole = a0 >= -OPENING_HALF && a1 <= OPENING_HALF;
            let (top0, top1) = if hole {
                (opening.top(a0), opening.top(a1))
            } else {
                (0.0, 0.0)
            };
            let parts: &[(f32, f32, f32, f32)] = &[
                (top0, top1, WALL_HEIGHT, WALL_HEIGHT),
                (0.0, 0.0, opening.bottom(), opening.bottom()),
            ];
            let count = if hole && opening.window { 2 } else { 1 };
            for &(low0, low1, high0, high1) in &parts[..count] {
                let bands = heights(shape, 0.0, 1.0);
                let corner = |a: f32, low: f32, high: f32, t: f32| {
                    let height = low + (high - low) * t;
                    let point = self.centre
                        + opening.along * a
                        + Vec3::Y * height
                        + normal * half_thickness(shape, height);
                    (
                        point,
                        face_uv(a, height),
                        height_shade(shape, height) * turned,
                    )
                };
                for band in bands.windows(2) {
                    let corners = [
                        corner(a0, low0, high0, band[0]),
                        corner(a1, low1, high1, band[0]),
                        corner(a1, low1, high1, band[1]),
                        corner(a0, low0, high0, band[1]),
                    ];
                    self.add(layers, layer, Surface::Wall, normal, corners);
                }
                if layer == Layer::Solid {
                    let (low, high) = ((low0 + low1) / 2.0, (high0 + high1) / 2.0);
                    layers.solid_faces.push(WallFace {
                        centre: self.centre
                            + opening.along * ((a0 + a1) / 2.0)
                            + Vec3::Y * ((low + high) / 2.0)
                            + normal * WALL_HALF,
                        normal,
                        half_across: (a1 - a0) / 2.0,
                        half_up: (high - low) / 2.0,
                    });
                }
            }
        }
    }
}
