//! Walls between the camera and what is seen (owner, 2026-11-09): solid, in
//! place of half see-through, and cut down to a stub near the player, so the
//! player is never hidden and what is further away is seen through them only
//! as the x-ray shows it (`view3d`).

use bevy::prelude::*;

use super::super::WALL_HEIGHT;
use super::super::walls::Layers;
use super::{Marks, Surface};

/// How far from the player (in cells, measured between middles) walls in the
/// way are cut down to `STUB_HEIGHT`, and how far they then rise to their
/// full height, so walls don't jump from one to the other as the player walks.
const CUT_NEAR: f32 = 4.0;
const CUT_FAR: f32 = 6.0;

/// How high a wall cut down stands.
const STUB_HEIGHT: f32 = WALL_HEIGHT * 0.25;

/// How high a wall in the way stands at `cell`, or `None` at its full height.
pub(super) fn cut_height(player: (i64, i64), cell: (i64, i64)) -> Option<f32> {
    let distance = Vec2::new((cell.0 - player.0) as f32, (cell.1 - player.1) as f32).length();
    if distance >= CUT_FAR {
        return None;
    }
    let rise = ((distance - CUT_NEAR) / (CUT_FAR - CUT_NEAR)).clamp(0.0, 1.0);
    // Eased, so the slope is gentle at both ends.
    let eased = rise * rise * (3.0 - 2.0 * rise);
    Some(STUB_HEIGHT + (WALL_HEIGHT - STUB_HEIGHT) * eased)
}

/// After a wall, door or window's cell is built: whatever of it went into the
/// see-through layer since `see_through` (vertices, indices) is a side seen
/// from behind, so the wall stands in the way. That goes into the solid walls,
/// facing the camera, and the whole cell (since `marks`) is cut down if it is
/// near the player.
pub(super) fn settle_in_the_way(
    layers: &mut Layers,
    marks: &Marks,
    see_through: (usize, usize),
    camera: Vec3,
    cut: Option<f32>,
) {
    let Layers {
        solid,
        see_through: through,
        ..
    } = layers;
    let in_the_way = through.move_facing(see_through, &mut solid[Surface::Wall as usize], camera);
    if let (true, Some(height)) = (in_the_way, cut) {
        marks.each(layers, |builder, from| builder.cut_down(from, height));
    }
}

#[cfg(test)]
mod tests;
