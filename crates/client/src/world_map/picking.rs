//! Turns a mouse click on the 3D map into a grid position, by following the
//! ray from the camera through the cursor until it meets the terrain.

use bevy::prelude::*;
use shared::surface::Surface;

use super::HEIGHT_SCALE;
use super::terrain::{HeightCache, ViewSquare};
use crate::surface_style::SurfaceStyle;

/// Distance between samples along the ray, in grid units. Under one unit, so
/// no grid point is skipped.
const STEP: f32 = 0.5;

/// Highest the terrain can be drawn: surfaces in metres reach about 90 above
/// a sea up to 60, plus woods.
const MAX_HEIGHT: f32 = 256.0 * HEIGHT_SCALE;

/// The grid point the ray first hits on the terrain window `square` centred
/// on `centre`, or `None` if it misses (or only hits an edge face). Works from
/// any direction, so it follows the camera when the view turns. `lift` is how
/// far the woods raise the drawn surface at a point.
#[allow(clippy::too_many_arguments)]
pub fn pick(
    ray: Ray3d,
    centre: (i64, i64),
    square: ViewSquare,
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
    mut lift: impl FnMut(&mut HeightCache, i64, i64) -> f32,
) -> Option<(i64, i64)> {
    let direction = *ray.direction;
    if direction.y >= 0.0 {
        return None;
    }
    // Skip the empty air above the highest possible terrain.
    let start = ((ray.origin.y - MAX_HEIGHT) / -direction.y).max(0.0);
    let mut entered = false;

    let mut t = start;
    while ray.origin.y + direction.y * t >= 0.0 {
        let point = ray.origin + direction * t;
        t += STEP;
        let (x, z) = (point.x.round() as i64, point.z.round() as i64);
        // Position across and along the window.
        let offset = Vec2::new(point.x - centre.0 as f32, point.z - centre.1 as f32);
        if !square.contains(offset) {
            if entered {
                return None; // Passed over the far edge.
            }
            continue;
        }
        let ground = style.shown_height(cache.get(surface, x, z), water_level) as f32
            * HEIGHT_SCALE
            + lift(cache, x, z);
        if point.y <= ground {
            // Below the surface on the very first sample inside the window
            // means the ray came through an edge face, not the top.
            return entered.then_some((x, z));
        }
        entered = true;
    }
    None
}

#[cfg(test)]
mod tests;
