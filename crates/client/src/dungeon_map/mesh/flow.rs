//! Water on sloping ground: a liquid cell is draped over the ground with the
//! rest, so a river runs down with the land and neighbouring cells meet. Its
//! picture flows downhill at a speed by how steep its cell is (`FLOW_BANDS`),
//! drawn by `sight_mask.wesl`.

use bevy::prelude::*;
use std::f32::consts::TAU;

use super::super::CELL_SIZE;
use super::super::walls::{Layers, Scene};
use super::Marks;
use crate::common::DRAWN_PER_16TH;

/// Drops across a cell (in 16ths of an elevation, top corner to bottom) from
/// which water flows gently, steadily and fast: bands 1, 2 and 3, each twice
/// the speed of the one before (`SPEEDS` in `sight_mask.wesl`). Less is still.
pub const FLOW_BANDS: [i32; 3] = [1, 2, 4];

/// What a vertex's flow is packed into for the shader: `band × 512` plus the
/// way it flows, in degrees from east towards south (0 for still water).
fn packed(band: usize, way: Vec2) -> f32 {
    let degrees = (way.y.atan2(way.x) / TAU * 360.0).rem_euclid(360.0);
    band as f32 * 512.0 + degrees
}

/// The band water dropping `drop` 16ths across its cell flows in (0 still).
pub fn band_of(drop: i32) -> usize {
    FLOW_BANDS.iter().filter(|&&least| drop >= least).count()
}

/// Gives the liquid cell at `(x, y)` (built since `marks`) its flow, from its
/// corners' heights.
pub(super) fn flow_liquid(layers: &mut Layers, marks: &Marks, scene: &Scene, (x, y): (i64, i64)) {
    let [nw, ne, sw, se] = scene.corners(x, y);
    let step = CELL_SIZE * DRAWN_PER_16TH;
    let drop = ((nw.max(ne).max(sw).max(se) - nw.min(ne).min(sw).min(se)) / step).round() as i32;
    // Downhill: towards the lower side, each way.
    let way = Vec2::new((nw + sw) - (ne + se), (nw + ne) - (sw + se));
    let flow = match band_of(drop) {
        band if band > 0 && way.length() > 0.0 => packed(band, way.normalize()),
        _ => 0.0,
    };
    marks.each(layers, |builder, from| builder.set_flow(from, flow));
}

#[cfg(test)]
mod tests;
