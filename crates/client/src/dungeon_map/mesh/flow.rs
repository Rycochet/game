//! Water down raised wilderness ground: a liquid cell's surface (with its bed
//! and the foot of its sides) slopes from corner to corner, each corner as low
//! as the lowest liquid cell round it, so a river runs smoothly down where its
//! cells step and neighbouring cells meet. Its picture flows downhill at a
//! speed by how steep it is (`FLOW_BANDS`), drawn by `sight_mask.wesl`.

use bevy::prelude::*;
use std::f32::consts::TAU;

use super::super::CELL_SIZE;
use super::super::liquid::LooksLiquid;
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

/// Slopes the liquid cell at `(x, y)` (built since `marks` and already lifted
/// to its own ground) to its corners, and gives it its flow.
pub(super) fn slope_liquid(layers: &mut Layers, marks: &Marks, scene: &Scene, (x, y): (i64, i64)) {
    if scene.heights.is_none() {
        return;
    }
    let own = scene.ground(x, y);
    // North-west, north-east, south-west, south-east: the lowest liquid round.
    let corner = |dx: i64, dy: i64| {
        [(x, y), (x + dx, y), (x, y + dy), (x + dx, y + dy)]
            .into_iter()
            .filter(|&(cx, cy)| scene.level.cell(cx, cy).looks_liquid())
            .map(|(cx, cy)| scene.ground(cx, cy))
            .fold(own, f32::min)
    };
    let [nw, ne, sw, se] = [corner(-1, -1), corner(1, -1), corner(-1, 1), corner(1, 1)];
    let centre = Vec2::new(x as f32, y as f32) * CELL_SIZE;
    let by = |position: Vec3| {
        let east = ((position.x - centre.x) / CELL_SIZE + 0.5).clamp(0.0, 1.0);
        let south = ((position.z - centre.y) / CELL_SIZE + 0.5).clamp(0.0, 1.0);
        let north_row = nw + (ne - nw) * east;
        let south_row = sw + (se - sw) * east;
        north_row + (south_row - north_row) * south - own
    };
    let step = CELL_SIZE * DRAWN_PER_16TH;
    let drop = ((nw.max(ne).max(sw).max(se) - nw.min(ne).min(sw).min(se)) / step).round() as i32;
    // Downhill: towards the lower side, each way.
    let way = Vec2::new((nw + sw) - (ne + se), (nw + ne) - (sw + se));
    let flow = match band_of(drop) {
        band if band > 0 && way.length() > 0.0 => packed(band, way.normalize()),
        _ => 0.0,
    };
    marks.each(layers, |builder, from| builder.slope(from, by, flow));
}

#[cfg(test)]
mod tests;
