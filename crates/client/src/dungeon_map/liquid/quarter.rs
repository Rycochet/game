//! Drawing one quarter of liquid, also in a wall's quarter.

use bevy::prelude::*;
use shared::dungeon::Level;

use super::super::CELL_SIZE;
use super::super::mesh::{MEMORY_SHADE, Surface, with_ambient};
use super::super::walls::{Layer, Layers, Scene, flat_uv};
use super::{
    Bed, CHANNEL_HALF, FULL_DEPTH, HALF_CELL, Outline, Point, Rings, bed_at, cell_light,
    outline_at, surfaces_near,
};

/// One quarter of liquid being drawn.
pub struct Quarter<'a> {
    pub layer: Layer,
    pub centre: Vec3,
    pub quarter: (f32, f32),
    pub outline: Outline,
    pub liquid: Surface,
    pub floor: Surface,
    /// Whether to draw the floor beside the liquid.
    pub with_floor: bool,
    /// The vertex colour at a point, before shading.
    pub colour: &'a dyn Fn(Vec3) -> Vec3,
}

/// Draws the liquid cell at `(x, y)`, whose cell is remembered: lit if in
/// sight, otherwise as memory.
pub fn build_cell(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    const QUARTERS: [(f32, f32); 4] = [(1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0)];
    let level = scene.level;
    let in_sight = scene.sight.is_seen(x, y);
    let colour = |point: Vec3| {
        if in_sight {
            with_ambient(cell_light(level, scene.light, (x, y), point))
        } else {
            Vec3::splat(MEMORY_SHADE)
        }
    };
    let (liquid, floor) = surfaces_near(level, (x, y));
    for quarter in QUARTERS {
        let Some(outline) = outline_at(level, (x, y), quarter) else {
            continue;
        };
        let bed = bed_at(level, (x, y), quarter, outline);
        let drawn = Quarter {
            layer: if in_sight {
                Layer::Solid
            } else {
                Layer::Memory
            },
            centre: Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE),
            quarter,
            outline,
            liquid,
            floor,
            with_floor: true,
            colour: &colour,
        };
        drawn.build(layers, bed);
    }
}

impl Quarter<'_> {
    /// Draws liquid in a wall's quarter, which is a full cell deep throughout.
    pub fn build_in_wall(&self, layers: &mut Layers) {
        self.build(layers, Bed::Flat(FULL_DEPTH));
    }

    /// A channel running into a wall's cell stops at the centre line if the
    /// far side of the wall is dry. Where the wall is narrower than the channel
    /// (at its end) that shows from outside, so it is closed there, in the
    /// wall's picture. `(x, y)` is the wall's cell.
    pub fn close_behind_wall(&self, layers: &mut Layers, level: &Level, cell: (i64, i64)) {
        let Some(behind) = self.outline.behind(self.quarter) else {
            return;
        };
        if self.layer == Layer::Memory || outline_at(level, cell, behind).is_some() {
            return;
        }
        let (a, b, towards) = if self.outline == Outline::AlongEast {
            ((0.0, 0.0), (0.0, CHANNEL_HALF), (HALF_CELL, 0.0))
        } else {
            ((0.0, 0.0), (CHANNEL_HALF, 0.0), (0.0, HALF_CELL))
        };
        let bed = Bed::Flat(FULL_DEPTH);
        self.side(layers, Surface::Wall, (a, b), towards, bed);
    }

    /// The surface, then for what is in sight the bed and sides under it (what
    /// is remembered stays quiet: only the outline, flat), then the floor.
    fn build(&self, layers: &mut Layers, bed: Bed) {
        let in_sight = self.layer != Layer::Memory;
        if self.outline == Outline::Full {
            let corners = [
                (0.0, 0.0),
                (HALF_CELL, 0.0),
                (HALF_CELL, HALF_CELL),
                (0.0, HALF_CELL),
            ];
            self.surface(layers, corners);
            if in_sight {
                self.bed(layers, corners, bed);
            }
            return;
        }
        let Rings {
            shore,
            liquid,
            floor,
        } = self.outline.rings();
        for i in 0..shore.len() - 1 {
            let water = [shore[i], shore[i + 1], liquid[i + 1], liquid[i]];
            self.surface(layers, water);
            if in_sight {
                self.bed(layers, water, bed);
                let piece = (shore[i], shore[i + 1]);
                self.side(layers, self.floor, piece, liquid[i], bed);
            }
            if self.with_floor {
                let beside = [shore[i], shore[i + 1], floor[i + 1], floor[i]];
                self.flat(layers, self.floor, beside, 0.0, &|_| 1.0);
            }
        }
    }

    pub(super) fn place(&self, (east, south): Point, height: f32) -> Vec3 {
        let (sx, sz) = self.quarter;
        self.centre + Vec3::new(sx * east, height, sz * south)
    }

    pub(super) fn uv(&self, (east, south): Point) -> [f32; 2] {
        let (sx, sz) = self.quarter;
        flat_uv(sx * east, sz * south)
    }

    pub(super) fn colour_at(&self, position: Vec3, shade: f32) -> Vec3 {
        (self.colour)(position) * shade
    }
}
