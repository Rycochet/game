//! Rectangles on the floor, and the floor the camera shows.

use bevy::prelude::*;

use super::super::CELL_SIZE;

/// A rectangle on the floor, in world units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Area {
    pub(super) min: Vec2,
    pub(super) max: Vec2,
}

impl Area {
    pub(super) fn size(self) -> Vec2 {
        self.max - self.min
    }

    pub(super) fn contains(self, other: Area) -> bool {
        self.min.cmple(other.min).all() && self.max.cmpge(other.max).all()
    }
}

/// The floor the camera shows, as far as the level goes.
pub(super) fn view_area(
    camera: &Camera,
    transform: &Transform,
    bounds: (i64, i64, i64, i64),
) -> Option<Area> {
    let size = camera.logical_viewport_size()?;
    let global = GlobalTransform::from(*transform);
    let far = transform.translation.y.abs() * 8.0 + CELL_SIZE * 8.0;
    let mut low = Vec2::splat(f32::MAX);
    let mut high = Vec2::splat(f32::MIN);
    for corner in [
        Vec2::ZERO,
        Vec2::new(size.x, 0.0),
        Vec2::new(0.0, size.y),
        size,
    ] {
        let ray = camera.viewport_to_world(&global, corner).ok()?;
        // A corner above the horizon has no floor: as far as anything shows.
        let along = if ray.direction.y < -1.0e-3 {
            (-ray.origin.y / ray.direction.y).min(far)
        } else {
            far
        };
        let point = ray.origin + *ray.direction * along;
        let point = Vec2::new(point.x, point.z);
        low = low.min(point);
        high = high.max(point);
    }
    let (min_x, min_y, width, height) = bounds;
    let level_low = Vec2::new(min_x as f32 - 1.0, min_y as f32 - 1.0) * CELL_SIZE;
    let level_high = Vec2::new((min_x + width) as f32, (min_y + height) as f32) * CELL_SIZE;
    let min = low.clamp(level_low, level_high);
    let max = high.clamp(level_low, level_high).max(min + CELL_SIZE);
    Some(Area { min, max })
}
