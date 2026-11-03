//! One wall cell being built: its quarters, light and colours.

use bevy::prelude::*;
use shared::dungeon::Sight;

use super::super::CELL_SIZE;
use super::super::mesh::{MEMORY_SHADE, Surface, with_ambient};
use super::{HALF_CELL, Layer, Layers, Scene, Tile};

impl<'a> Tile<'a> {
    pub(super) fn new(scene: &'a Scene<'a>, (x, y): (i64, i64), joins: [bool; 4]) -> Self {
        Self {
            scene,
            x,
            y,
            centre: Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE),
            joins,
        }
    }

    /// The other cells around the corner of this cell in a quarter.
    pub(super) fn around_corner(&self, (sx, sz): (f32, f32)) -> [(i64, i64); 3] {
        let (dx, dy) = (sx as i64, sz as i64);
        [
            (self.x + dx, self.y),
            (self.x, self.y + dy),
            (self.x + dx, self.y + dy),
        ]
    }

    /// Whether `sight` includes an open cell around the corner in a quarter.
    fn quarter_in(&self, sight: &Sight, quarter: (f32, f32)) -> bool {
        let level = self.scene.level;
        self.around_corner(quarter)
            .iter()
            .any(|&(x, y)| level.is_open(x, y) && sight.is_seen(x, y))
    }

    /// Whether a quarter counts as in `sight`: an open cell around its corner
    /// is, or the wall is and such a cell is known. The edge of sight can run
    /// along the floor of the wall's cell without reaching that open cell, and
    /// the mask would hide memory there.
    pub(super) fn quarter_shows(&self, sight: &Sight, quarter: (f32, f32)) -> bool {
        self.quarter_in(sight, quarter)
            || (sight.is_seen(self.x, self.y) && self.quarter_open(quarter))
    }

    /// Whether an open cell the client knows of is around the corner in a
    /// quarter, seen or not (cells it doesn't know of are wall here).
    pub(super) fn quarter_open(&self, quarter: (f32, f32)) -> bool {
        let level = self.scene.level;
        self.around_corner(quarter)
            .iter()
            .any(|&(x, y)| level.is_open(x, y))
    }

    /// The light at a point on this cell, blending between the cell's corners
    /// except across the wall itself.
    fn light_at(&self, point: Vec3) -> Vec3 {
        let local = point - self.centre;
        let (sx, sz) = (
            if local.x < 0.0 { -1.0f32 } else { 1.0 },
            if local.z < 0.0 { -1.0f32 } else { 1.0 },
        );
        let Scene { level, light, .. } = self.scene;
        let corner = |sx: f32, sz: f32| {
            let cx = self.x + i64::from(sx > 0.0);
            let cy = self.y + i64::from(sz > 0.0);
            light.corner(level, cx, cy)
        };
        let here = corner(sx, sz).unwrap_or(light.at(self.x, self.y));
        let wall_north_south = self.joins[if sz < 0.0 { 0 } else { 2 }];
        let wall_east_west = self.joins[if sx < 0.0 { 3 } else { 1 }];
        let beyond_x = if wall_north_south {
            here
        } else {
            corner(-sx, sz).unwrap_or(here)
        };
        let beyond_z = if wall_east_west {
            here
        } else {
            corner(sx, -sz).unwrap_or(here)
        };
        let fx = (local.x.abs() / HALF_CELL).min(1.0);
        let fz = (local.z.abs() / HALF_CELL).min(1.0);
        here + (beyond_x - here) * (1.0 - fx) / 2.0 + (beyond_z - here) * (1.0 - fz) / 2.0
    }

    /// A vertex colour at a point: its light (or memory's grey), shaded.
    pub(super) fn colour(&self, layer: Layer, point: Vec3, shade: f32) -> Vec3 {
        match layer {
            Layer::Memory => Vec3::splat(MEMORY_SHADE * shade),
            _ => with_ambient(self.light_at(point)) * shade,
        }
    }

    /// Adds a four-cornered surface, each corner `(position, picture
    /// coordinates, shade)`.
    pub(super) fn add(
        &self,
        layers: &mut Layers,
        layer: Layer,
        surface: Surface,
        normal: Vec3,
        corners: [(Vec3, [f32; 2], f32); 4],
    ) {
        let positions = corners.map(|(position, _, _)| position);
        let uvs = corners.map(|(_, uv, _)| uv);
        let colours = corners.map(|(position, _, shade)| self.colour(layer, position, shade));
        layers
            .builder(layer, surface)
            .free(positions, normal, uvs, colours);
    }
}
