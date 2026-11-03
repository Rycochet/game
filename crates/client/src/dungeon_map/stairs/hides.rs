//! Whether stairs stand between the camera and a viewer.

use bevy::prelude::*;
use shared::protocol::SeenStairs;
use shared::site::{StairsDirection, StairsStyle};

use super::super::CELL_SIZE;
use super::super::mesh::View;
use super::{LADDER_ABOVE_FLOOR, RAIL_HEIGHT, STAIRS_UP_TOP, VIEWER_HEIGHT};

/// Whether stairs stand between the camera and any viewer's token (or on the
/// same cell), so they are drawn see-through, like a wall in the way.
pub fn hides_viewer(stairs: &SeenStairs, views: &[View], camera: Vec3) -> bool {
    let top = match (stairs.direction, stairs.style) {
        (StairsDirection::Up, _) => STAIRS_UP_TOP,
        (_, StairsStyle::Spiral | StairsStyle::Turning) => RAIL_HEIGHT,
        _ => LADDER_ABOVE_FLOOR,
    };
    let half = CELL_SIZE / 2.0;
    let centre = Vec3::new(stairs.x as f32, 0.0, stairs.y as f32) * CELL_SIZE;
    let low = centre - Vec3::new(half, 0.0, half);
    let high = centre + Vec3::new(half, top, half);
    views.iter().any(|view| {
        let foot = Vec3::new(view.cell.0 as f32, 0.0, view.cell.1 as f32) * CELL_SIZE;
        let head = foot + Vec3::Y * VIEWER_HEIGHT;
        segment_hits_box(camera, foot, low, high) || segment_hits_box(camera, head, low, high)
    })
}

/// Whether the straight line from `from` to `to` passes through a box.
pub(super) fn segment_hits_box(from: Vec3, to: Vec3, low: Vec3, high: Vec3) -> bool {
    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
    let way = to - from;
    for axis in 0..3 {
        if way[axis].abs() < 1e-6 {
            if from[axis] < low[axis] || from[axis] > high[axis] {
                return false;
            }
            continue;
        }
        let (a, b) = (
            (low[axis] - from[axis]) / way[axis],
            (high[axis] - from[axis]) / way[axis],
        );
        enter = enter.max(a.min(b));
        leave = leave.min(a.max(b));
    }
    enter <= leave
}
