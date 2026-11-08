//! How high the wilderness's cells are, for walking and drawing, for the level
//! the player is on (`shared::movement::WildHeights`).

use bevy::prelude::*;
use shared::movement::{FlatGround, WildHeights};

/// How high a 16th of an elevation is drawn, in cells: one step of height
/// (owner's choice, 2026-11-08: 1/16 from screenshots, then 1/8 after playing).
pub const DRAWN_PER_16TH: f32 = 0.125;

/// The heights of the wilderness level's cells, worked out by `player` from the
/// world map's ground and the flats the server told of, whenever the level is
/// the wilderness and its window moves; `None` anywhere else. Step costs and
/// routes there go by them, as the server's do, and the views draw by them.
#[derive(Resource, Default)]
pub struct WildCellHeights {
    /// The level's bounds they were worked out for.
    pub bounds: (i64, i64, i64, i64),
    /// The flats they were worked out with (`CurrentDungeon::flats`).
    pub flats: Vec<FlatGround>,
    pub heights: Option<WildHeights>,
    /// The height drawn at 0, in 16ths: the window's middle cell's, so what is
    /// drawn stays near the origin's height (far up, the depth buffer and
    /// floats lose precision and the ground shows through tokens).
    pub reference: i32,
}

impl WildCellHeights {
    /// The heights at each end of a step, if both are known.
    pub fn ends(&self, from: (i64, i64), to: (i64, i64)) -> Option<(i32, i32)> {
        let heights = self.heights.as_ref()?;
        (heights.covers(from.0, from.1) && heights.covers(to.0, to.1))
            .then(|| (heights.at(from.0, from.1), heights.at(to.0, to.1)))
    }

    /// How high a cell is drawn, in cells, from `reference`: 0 off the
    /// wilderness, or where its height isn't known.
    pub fn drawn(&self, x: i64, y: i64) -> f32 {
        match &self.heights {
            Some(heights) if heights.covers(x, y) => {
                (heights.at(x, y) - self.reference) as f32 * DRAWN_PER_16TH
            }
            _ => 0.0,
        }
    }

    /// How high a place between cells' middles is drawn, in cells: its own
    /// cell's height (cells are flat), blended with the next cell's only in the
    /// last `EDGE_BLEND` of a cell towards it, so a token or the camera stands
    /// on its cell and rises or drops smoothly as it steps over the edge.
    pub fn drawn_between(&self, x: f32, y: f32) -> f32 {
        if self.heights.is_none() {
            return 0.0;
        }
        let (cx, cy) = (x.round(), y.round());
        let (ox, oy) = (x - cx, y - cy);
        let (cx, cy) = (cx as i64, cy as i64);
        let (sx, sy) = (ox.signum() as i64, oy.signum() as i64);
        let (wx, wy) = (edge_weight(ox), edge_weight(oy));
        let row = |y: i64| self.drawn(cx, y) * (1.0 - wx) + self.drawn(cx + sx, y) * wx;
        row(cy) * (1.0 - wy) + row(cy + sy) * wy
    }
}

/// How much of a cell, at its edge, a place blends towards the next cell's
/// height.
const EDGE_BLEND: f32 = 0.25;

/// How far towards the next cell's height a place `offset` from its cell's
/// middle (-0.5 to 0.5) is: none until the edge's last `EDGE_BLEND`, half at
/// the edge itself.
fn edge_weight(offset: f32) -> f32 {
    let into = ((offset.abs() - (0.5 - EDGE_BLEND)) / EDGE_BLEND).clamp(0.0, 1.0);
    into * into * (3.0 - 2.0 * into) * 0.5
}
