//! A click on a World Gate's model means the gate's own cell. The frame stands
//! up to about 1.6 cells tall, so the floor a click on it reaches is usually
//! the cell behind, and walking there would only lead up beside the gate.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};

use super::CELL_SIZE;
use super::ground_hit::Ground;
use super::mesh::WallFace;
use super::picking::hits_face;

/// How tall a gate is taken to be for clicks, in cells: a little over the
/// tallest design (`hall_view::GateModel::size`).
const GATE_TALL: f32 = 1.7;

/// How wide a gate is taken to be either side of its cell's middle, in cells:
/// the frame is 1.14 across, along `x`.
const GATE_HALF_WIDE: f32 = 0.57;

/// How deep a gate is taken to be either side of its cell's middle, in cells:
/// the frame is thin, but a click on its water or edge should count.
const GATE_HALF_DEEP: f32 = 0.25;

/// How far apart the ray is tried, in cells: small enough to land in any
/// gate's box it crosses (half a cell deep).
const STRIDE: f32 = 0.2;

/// The seen World Gate whose model the ray reaches first, if no solid wall
/// piece is in the way.
pub fn gate_clicked(
    ray: Ray3d,
    level: &Level,
    sight: &Sight,
    solid_faces: &[WallFace],
    ground: Ground,
) -> Option<(i64, i64)> {
    let direction = *ray.direction;
    let top = GATE_TALL * CELL_SIZE;
    // From where the ray comes down past the top of the tallest gate to where
    // it reaches the ground.
    let (end, _, _) = ground.hit(ray)?;
    let start = (end - top / -direction.y).max(0.0);
    let mut distance = start;
    while distance <= end {
        let point = ray.origin + direction * distance;
        let cell = (
            (point.x / CELL_SIZE).round() as i64,
            (point.z / CELL_SIZE).round() as i64,
        );
        let across = (point.x - cell.0 as f32 * CELL_SIZE).abs();
        let deep = (point.z - cell.1 as f32 * CELL_SIZE).abs();
        if across <= GATE_HALF_WIDE * CELL_SIZE
            && deep <= GATE_HALF_DEEP * CELL_SIZE
            && point.y - ground.at(cell.0, cell.1) <= top
            && level.cell(cell.0, cell.1) == Cell::WorldGate
            && sight.is_seen(cell.0, cell.1)
        {
            let blocked = solid_faces
                .iter()
                .any(|face| hits_face(ray.origin, direction, face).is_some_and(|d| d < distance));
            return (!blocked).then_some(cell);
        }
        distance += STRIDE * CELL_SIZE;
    }
    None
}

#[cfg(test)]
mod tests;
