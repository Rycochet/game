//! Where the camera's ray reaches the ground, for clicks: the floor at height
//! 0, or in the wilderness the first raised cell it comes down on, or the face
//! down from one it runs into (which counts as that cell).

use bevy::prelude::*;

use super::CELL_SIZE;
use crate::common::WildCellHeights;

/// The most cells a ray is followed across before giving up.
const MOST_CELLS: usize = 4000;

/// The ground clicks land on: flat, or the wilderness's raised cells.
#[derive(Clone, Copy)]
pub struct Ground<'a>(pub Option<&'a WildCellHeights>);

impl Ground<'_> {
    /// Flat ground at height 0: every level but the wilderness (tests).
    #[cfg(test)]
    pub const FLAT: Ground<'static> = Ground(None);

    /// How high a cell's ground is drawn, in world units.
    pub fn at(&self, x: i64, y: i64) -> f32 {
        self.0
            .map_or(0.0, |heights| heights.drawn(x, y) * CELL_SIZE)
    }

    /// How high a known cell's ground is drawn, or `None` where its height
    /// isn't known (beyond the window), which the ray passes over.
    fn known(&self, x: i64, y: i64) -> Option<f32> {
        match self.0 {
            None => Some(0.0),
            Some(heights) => {
                let known = heights.heights.as_ref()?.covers(x, y);
                known.then(|| self.at(x, y))
            }
        }
    }

    /// How far along the ray it reaches the ground, where, and in which cell.
    pub fn hit(&self, ray: Ray3d) -> Option<(f32, Vec3, (i64, i64))> {
        let (origin, direction) = (ray.origin, *ray.direction);
        if direction.y >= 0.0 {
            return None;
        }
        let cell_of = |point: Vec3| {
            (
                (point.x / CELL_SIZE).round() as i64,
                (point.z / CELL_SIZE).round() as i64,
            )
        };
        if self.0.is_none() {
            let distance = -origin.y / direction.y;
            let point = origin + direction * distance;
            return Some((distance, point, cell_of(point)));
        }
        // Cell by cell along the ray's way across the ground.
        let mut cell = cell_of(origin);
        let mut entered = 0.0;
        let step = (direction.x.signum() as i64, direction.z.signum() as i64);
        let to_edge = |at: f32, cell: i64, sign: i64, along: f32| {
            if sign == 0 {
                return f32::INFINITY;
            }
            let edge = (cell as f32 + 0.5 * sign as f32) * CELL_SIZE;
            (edge - at) / along
        };
        for _ in 0..MOST_CELLS {
            let leaves_x = to_edge(origin.x, cell.0, step.0, direction.x);
            let leaves_z = to_edge(origin.z, cell.1, step.1, direction.z);
            let left = leaves_x.min(leaves_z);
            if let Some(ground) = self.known(cell.0, cell.1) {
                let height_at = |distance: f32| origin.y + direction.y * distance;
                if height_at(entered) <= ground {
                    // Into the face down from this cell.
                    return Some((entered, origin + direction * entered, cell));
                }
                if height_at(left) <= ground {
                    let distance = (ground - origin.y) / direction.y;
                    return Some((distance, origin + direction * distance, cell));
                }
            }
            entered = left;
            if leaves_x < leaves_z {
                cell.0 += step.0;
            } else {
                cell.1 += step.1;
            }
        }
        None
    }
}
