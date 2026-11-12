//! Where the camera's ray reaches the ground, for clicks: the floor at height
//! 0, or on sloping ground (`common::DrawnGround`) the first place it comes
//! down onto the surface, found in short steps along it and then narrowed.

use bevy::prelude::*;

use super::CELL_SIZE;
use crate::common::DrawnGround;

/// How far apart, across the ground, the ray is tried, in cells: under a
/// quarter of a cell, no slope the ground has can hide between two tries.
const TRY_EVERY: f32 = 0.2;

/// How many times the place between the last try above the ground and the
/// first below it is halved.
const NARROWING: usize = 14;

/// The most tries before giving up.
const MOST_TRIES: usize = 40_000;

/// The ground clicks land on: flat, or the level's sloping ground.
#[derive(Clone, Copy)]
pub struct Ground<'a>(pub Option<&'a DrawnGround>);

impl Ground<'_> {
    /// Flat ground at height 0 (tests).
    #[cfg(test)]
    pub const FLAT: Ground<'static> = Ground(None);

    /// How high a cell's middle is drawn, in world units.
    pub fn at(&self, x: i64, y: i64) -> f32 {
        self.0.map_or(0.0, |ground| ground.drawn(x, y) * CELL_SIZE)
    }

    /// How high the ground is at a point, in world units, or `None` over a
    /// cell whose height isn't known (beyond the window), which the ray
    /// passes over.
    fn under(&self, point: Vec3) -> Option<f32> {
        let Some(ground) = self.0 else {
            return Some(0.0);
        };
        let (x, y) = (point.x / CELL_SIZE, point.z / CELL_SIZE);
        let cell = (x.round() as i64, y.round() as i64);
        ground
            .covers(cell.0, cell.1)
            .then(|| ground.drawn_between(x, y) * CELL_SIZE)
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
        if self.0.is_none_or(|ground| !ground.slopes()) {
            let distance = -origin.y / direction.y;
            let point = origin + direction * distance;
            return Some((distance, point, cell_of(point)));
        }
        let point_at = |distance: f32| origin + direction * distance;
        let below = |distance: f32| {
            let point = point_at(distance);
            self.under(point).is_some_and(|ground| point.y <= ground)
        };
        // Steps short across the ground, and at most a cell downwards when
        // looking almost straight down.
        let across = Vec2::new(direction.x, direction.z).length();
        let step = (TRY_EVERY * CELL_SIZE / across).min(CELL_SIZE / -direction.y);
        let mut above = 0.0;
        for _ in 0..MOST_TRIES {
            let next = above + step;
            if below(next) {
                let mut low = next;
                for _ in 0..NARROWING {
                    let middle = (above + low) / 2.0;
                    if below(middle) {
                        low = middle;
                    } else {
                        above = middle;
                    }
                }
                let point = point_at(low);
                return Some((low, point, cell_of(point)));
            }
            above = next;
        }
        None
    }
}

#[cfg(test)]
mod tests;
