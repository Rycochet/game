//! A wall's faces, cut into pieces, and its top.

use bevy::prelude::*;

use super::super::mesh::{Surface, WallFace, faces_camera};
use super::super::theme::Shape;
use super::super::{CELL_SIZE, WALL_HEIGHT};
use super::{
    Layer, Layers, NATURAL_SEGMENTS, ROUGH_SHADE, ROUGHNESS, SIDEWAYS_SHADE, TOP_SHADE, Tile,
    face_uv, flat_uv, height_shade, heights, noise,
};

impl<'a> Tile<'a> {
    /// Draws a face of a wall, if it should be seen, and says on which layer.
    /// `see_through` says its stretch of wall is see-through.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn wall_face(
        &self,
        layers: &mut Layers,
        quarter: (f32, f32),
        in_sight: bool,
        facing: usize,
        running: usize,
        reach: [f32; 4],
        see_through: bool,
    ) -> Option<Layer> {
        let (sx, sz) = quarter;
        let scene = self.scene;
        let faces_south_or_north = facing.is_multiple_of(2);
        // From local (along, out from the centre, height) to the world.
        let place = |along: f32, out: f32, height: f32| {
            let (east, south) = if faces_south_or_north {
                (sx * along, sz * out)
            } else {
                (sx * out, sz * along)
            };
            self.centre + Vec3::new(east, height, south)
        };
        let normal = if faces_south_or_north {
            Vec3::Z * sz
        } else {
            Vec3::X * sx
        };
        let (plane, from, to) = self.face_at(facing, running, reach, WALL_HEIGHT / 2.0);
        if to <= from {
            return None;
        }
        let middle = place((from + to) / 2.0, plane, WALL_HEIGHT / 2.0);
        let faces = faces_camera(middle, normal, scene.camera);
        let layer = if in_sight {
            if !self.face_seen(quarter, facing) {
                // The quarter shows from round the wall's end, but this face
                // is out of sight: as any other memory of it.
                if !faces {
                    return None;
                }
                Layer::Memory
            } else if see_through {
                Layer::SeeThrough
            } else {
                Layer::Solid
            }
        } else if faces {
            Layer::Memory
        } else {
            return None;
        };
        if layer == Layer::Solid {
            layers.solid_faces.push(WallFace {
                centre: middle,
                normal,
                half_across: (to - from) / 2.0,
                half_up: WALL_HEIGHT / 2.0,
            });
        }

        for corners in self.face_pieces(quarter, facing, running, reach) {
            self.add(layers, layer, Surface::Wall, normal, corners);
        }
        if layer != Layer::Memory {
            self.face_under_liquid(layers, layer, quarter, (facing, running), reach);
        }
        Some(layer)
    }

    /// The pieces a face towards side `facing` is cut into, each four corners
    /// of `(position, picture coordinates, shade)`.
    fn face_pieces(
        &self,
        (sx, sz): (f32, f32),
        facing: usize,
        running: usize,
        reach: [f32; 4],
    ) -> Vec<[(Vec3, [f32; 2], f32); 4]> {
        let shape = self.scene.shape;
        let faces_south_or_north = facing.is_multiple_of(2);
        let place = |along: f32, out: f32, height: f32| {
            let (east, south) = if faces_south_or_north {
                (sx * along, sz * out)
            } else {
                (sx * out, sz * along)
            };
            self.centre + Vec3::new(east, height, south)
        };
        let (_, from, to) = self.face_at(facing, running, reach, WALL_HEIGHT / 2.0);
        let turned = if faces_south_or_north {
            1.0
        } else {
            SIDEWAYS_SHADE
        };
        let segments = match shape {
            Shape::Natural if to - from > CELL_SIZE * 0.2 => NATURAL_SEGMENTS,
            _ => 1,
        };
        let cuts = heights(shape, 0.0, WALL_HEIGHT);
        let mut pieces = Vec::new();
        // Only points inside the face wander, so faces still meet each other.
        let corner = |band: usize, segment: usize| {
            let height = cuts[band];
            let (plane, from, to) = self.face_at(facing, running, reach, height);
            let along = from + (to - from) * segment as f32 / segments as f32;
            let inside = shape == Shape::Natural
                && band > 0
                && band + 1 < cuts.len()
                && segment > 0
                && segment < segments;
            let bump = if inside {
                let seed = 4 * self.x + 7 * self.y + facing as i64 * 11 + (sx + 2.0 * sz) as i64;
                noise(seed, band as i64, segment as i64)
            } else {
                0.0
            };
            let point = place(along, plane + bump * ROUGHNESS, height);
            let across = if faces_south_or_north {
                point.x - self.centre.x
            } else {
                point.z - self.centre.z
            };
            let shade = height_shade(shape, height) * turned * (1.0 + bump * ROUGH_SHADE);
            (point, face_uv(across, height), shade)
        };
        for band in 0..cuts.len() - 1 {
            for segment in 0..segments {
                pieces.push([
                    corner(band, segment),
                    corner(band, segment + 1),
                    corner(band + 1, segment + 1),
                    corner(band + 1, segment),
                ]);
            }
        }
        pieces
    }

    /// The top of a wall in a quarter: the part running one way, then what
    /// the other way adds to it.
    pub(super) fn wall_top(
        &self,
        layers: &mut Layers,
        layer: Layer,
        (sx, sz): (f32, f32),
        (east_west, north_south): (usize, usize),
        reach: [f32; 4],
    ) {
        let (z_plane, _, x_end) = self.face_at(north_south, east_west, reach, WALL_HEIGHT);
        let (x_plane, _, z_end) = self.face_at(east_west, north_south, reach, WALL_HEIGHT);
        let point = |east: f32, south: f32| {
            let (e, s) = (sx * east, sz * south);
            let position = self.centre + Vec3::new(e, WALL_HEIGHT, s);
            (position, flat_uv(e, s), TOP_SHADE)
        };
        let rect = |x0: f32, x1: f32, z0: f32, z1: f32| {
            [point(x0, z0), point(x1, z0), point(x1, z1), point(x0, z1)]
        };
        self.add(
            layers,
            layer,
            Surface::Wall,
            Vec3::Y,
            rect(0.0, x_end, 0.0, z_plane),
        );
        if z_end > z_plane {
            self.add(
                layers,
                layer,
                Surface::Wall,
                Vec3::Y,
                rect(0.0, x_plane, z_plane, z_end),
            );
        }
    }
}
