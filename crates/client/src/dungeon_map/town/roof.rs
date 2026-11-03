//! Roofs: each outermost cell's slope and the flat middle.

use bevy::prelude::*;

use super::super::walls::{Layer, Layers, Scene};
use super::super::{CELL_SIZE, WALL_HEIGHT};
use super::{
    FIRST_ROOF_TILE, HALF_CELL, ROOF_RAMP, ROOF_RISE, WALL_OUT, atlas_uv, emit, flat_uv,
    layer_colour,
};

/// The flat black middle of a roof.
pub(super) fn roof_centre(layers: &mut Layers, scene: &Scene, layer: Layer, centre: Vec3) {
    let height = Vec3::Y * (WALL_HEIGHT + ROOF_RISE);
    let corners = [
        Vec3::new(-HALF_CELL, 0.0, -HALF_CELL),
        Vec3::new(HALF_CELL, 0.0, -HALF_CELL),
        Vec3::new(HALF_CELL, 0.0, HALF_CELL),
        Vec3::new(-HALF_CELL, 0.0, HALF_CELL),
    ];
    emit(
        layers,
        scene,
        layer,
        corners.map(|corner| centre + corner + height),
        Vec3::Y,
        // Any part of any tile: it is black.
        [atlas_uv(FIRST_ROOF_TILE, 0.5, 0.5); 4],
        [Vec3::ZERO; 4],
    );
}

/// One outermost cell's roof.
pub(super) struct Roof {
    pub(super) centre: Vec3,
    /// Which cells round it are the same building, by row and column.
    pub(super) neighbours: [[bool; 3]; 3],
    pub(super) layer: Layer,
    pub(super) tile: usize,
}

impl Roof {
    /// How far in from the walls a point is (from the cell's centre), up to
    /// `ROOF_RAMP`, measured square: it makes corners like a hipped roof.
    fn distance_in(&self, east: f32, south: f32) -> f32 {
        let mut nearest = ROOF_RAMP;
        // The space outside a wall reaches this far from its own cell's
        // centre, towards this cell.
        let reach = CELL_SIZE - WALL_OUT;
        for (row, cells) in self.neighbours.iter().enumerate() {
            for (column, &inside) in cells.iter().enumerate() {
                if inside {
                    continue;
                }
                let dx = (column as f32 - 1.0) * CELL_SIZE - east;
                let dz = (row as f32 - 1.0) * CELL_SIZE - south;
                nearest = nearest.min(dx.abs().max(dz.abs()) - reach);
            }
        }
        nearest.max(0.0)
    }

    pub(super) fn patch(&self, layers: &mut Layers, scene: &Scene, patch: Rect) {
        let corners = [
            (patch.min.x, patch.min.y),
            (patch.max.x, patch.min.y),
            (patch.max.x, patch.max.y),
            (patch.min.x, patch.max.y),
        ];
        let along = corners.map(|(east, south)| self.distance_in(east, south) / ROOF_RAMP);
        let shade = layer_colour(self.layer);
        let positions = std::array::from_fn(|i| {
            let (east, south) = corners[i];
            self.centre + Vec3::new(east, WALL_HEIGHT + ROOF_RISE * along[i], south)
        });
        emit(
            layers,
            scene,
            self.layer,
            positions,
            Vec3::Y,
            corners.map(|(east, south)| flat_uv(self.tile, east, south)),
            along.map(|up| shade * (1.0 - up)),
        );
    }
}
