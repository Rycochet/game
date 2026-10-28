//! Turns a mouse click on the 3D map into a grid position, by following the
//! ray from the camera through the cursor until it meets the terrain.

use bevy::prelude::*;
use shared::surface::Surface;

use super::terrain::HeightCache;
use super::{HALF_SIZE, HEIGHT_SCALE};
use crate::surface_style::SurfaceStyle;

/// Distance between samples along the ray, in grid units. Under one unit, so
/// no grid point is skipped.
const STEP: f32 = 0.5;

/// Highest the terrain can be drawn (surface heights stay under 128).
const MAX_HEIGHT: f32 = 128.0 * HEIGHT_SCALE;

/// The grid point the ray first hits on the terrain window centred on
/// `centre`, or `None` if it misses (or only hits an edge face). Works from
/// any direction, so it follows the camera when the view turns.
pub fn pick(
    ray: Ray3d,
    centre: (i64, i64),
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
) -> Option<(i64, i64)> {
    let direction = *ray.direction;
    if direction.y >= 0.0 {
        return None;
    }
    // Skip the empty air above the highest possible terrain.
    let start = ((ray.origin.y - MAX_HEIGHT) / -direction.y).max(0.0);
    let half = HALF_SIZE as f32;
    let mut entered = false;

    let mut t = start;
    while ray.origin.y + direction.y * t >= 0.0 {
        let point = ray.origin + direction * t;
        t += STEP;
        let (x, z) = (point.x.round() as i64, point.z.round() as i64);
        if (x - centre.0).abs() as f32 > half || (z - centre.1).abs() as f32 > half {
            if entered {
                return None; // Passed over the far edge.
            }
            continue;
        }
        let ground =
            style.shown_height(cache.get(surface, x, z), water_level) as f32 * HEIGHT_SCALE;
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
mod tests {
    use super::*;
    use shared::rng::Seed;
    use shared::world::WorldParams;

    fn setup() -> (Surface, SurfaceStyle, HeightCache, f64) {
        let world = WorldParams::from_seed(Seed::new(12345));
        let water = world.sea_level;
        (
            Surface::new(&world),
            SurfaceStyle::default(),
            HeightCache::default(),
            water,
        )
    }

    fn ray(origin: Vec3, towards: Vec3) -> Ray3d {
        Ray3d::new(origin, Dir3::new(towards - origin).unwrap())
    }

    #[test]
    fn straight_down_hits_the_point_below() {
        let (surface, style, mut cache, water) = setup();
        let hit = pick(
            ray(Vec3::new(10.0, 500.0, -20.0), Vec3::new(10.0, 0.0, -20.0)),
            (0, 0),
            &mut cache,
            &surface,
            &style,
            water,
        );
        assert_eq!(hit, Some((10, -20)));
    }

    #[test]
    fn through_an_edge_face_misses() {
        let (surface, style, mut cache, water) = setup();
        // Low and level from the south: enters the window below the surface.
        let hit = pick(
            ray(Vec3::new(0.0, 5.0, 600.0), Vec3::new(0.0, 4.0, 0.0)),
            (0, 0),
            &mut cache,
            &surface,
            &style,
            water,
        );
        assert_eq!(hit, None);
    }

    #[test]
    fn through_the_north_face_misses() {
        let (surface, style, mut cache, water) = setup();
        // As above, from the north (the view turned round).
        let hit = pick(
            ray(Vec3::new(0.0, 5.0, -600.0), Vec3::new(0.0, 4.0, 0.0)),
            (0, 0),
            &mut cache,
            &surface,
            &style,
            water,
        );
        assert_eq!(hit, None);
    }

    #[test]
    fn from_the_east_hits_the_top() {
        let (surface, style, mut cache, water) = setup();
        // Looking west from high up: lands somewhere on the window.
        let hit = pick(
            ray(Vec3::new(900.0, 600.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
            (0, 0),
            &mut cache,
            &surface,
            &style,
            water,
        );
        let (x, y) = hit.expect("should hit the terrain");
        assert!(x.abs() <= HALF_SIZE && y.abs() <= 1);
    }

    #[test]
    fn over_the_map_misses() {
        let (surface, style, mut cache, water) = setup();
        let hit = pick(
            ray(Vec3::new(0.0, 500.0, 600.0), Vec3::new(0.0, 300.0, -600.0)),
            (0, 0),
            &mut cache,
            &surface,
            &style,
            water,
        );
        assert_eq!(hit, None);
    }
}
