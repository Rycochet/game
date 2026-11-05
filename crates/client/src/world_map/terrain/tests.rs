use bevy::prelude::*;

use super::super::{HALF_SIZE, MAX_HALF, window_half};
use super::edges::crossings;
use super::*;

#[test]
fn grid_reaches_the_window_corners() {
    let corner = MAX_HALF * std::f32::consts::SQRT_2;
    // The corner plus one square for the triangles around it.
    assert!(REACH as f32 >= corner + 1.0);
}

#[test]
fn window_never_outgrows_the_stores() {
    // Zoomed in, the window shrinks; zoomed out past `MAX_HALF` (debug
    // tools only) it stays that size.
    assert_eq!(window_half(1.0), HALF_SIZE as f32);
    assert_eq!(window_half(0.6), HALF_SIZE as f32 * 0.6);
    assert_eq!(window_half(1.0 / 0.1), MAX_HALF);
    // The largest window turned 45°, where its bounding box is widest, is
    // still within what `TerrainGrid` keeps (`reach` asserts it).
    let diagonal = Vec2::new(1.0, 1.0).normalize();
    let square = ViewSquare {
        right: diagonal,
        towards: Vec2::new(-diagonal.y, diagonal.x),
        half: window_half(10.0),
    };
    let (x, z) = square.reach();
    assert!(x <= REACH && z <= REACH);
}

#[test]
fn crossings_include_ends_and_grid_lines() {
    let stops = crossings(Vec2::new(0.5, 0.5), Vec2::new(2.5, 0.5));
    // x = 1, x = 2 and the diagonals x + z = 1, 2, 3 (z is 0.5): the
    // diagonals fall at x = 0.5 (the start), 1.5 and 2.5 (the end).
    assert_eq!(stops, [0.0, 0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn clipping_keeps_the_inside_part() {
    let half = HALF_SIZE as f32;
    let square = ViewSquare {
        right: Vec2::X,
        towards: Vec2::Y,
        half,
    };
    let at = |x: f32, z: f32| Corner {
        position: Vec3::new(x, 0.0, z),
        normal: Vec3::Y,
        colour: Vec4::ONE,
        cover: Vec4::ZERO,
        inside: square.inside(Vec2::new(x, z)),
    };
    // A triangle sticking out of the right side by half its width.
    let clipped = clip(&[
        at(half - 1.0, 0.0),
        at(half - 1.0, 1.0),
        at(half + 1.0, 0.0),
    ]);
    assert!(clipped.len() >= 3);
    assert!(clipped.iter().all(|c| c.position.x <= half + 1e-4));
}

#[test]
fn tokens_rest_on_the_drawn_triangles() {
    use super::super::rivers::KnownRivers;
    use crate::surface_style::SurfaceStyle;
    use shared::climate::Climate;
    use shared::rng::Seed;
    use shared::surface::Surface;
    use shared::vegetation::Vegetation;
    use shared::world::WorldParams;

    let params = WorldParams::from_seed(Seed::new(12345));
    let (surface, climate, vegetation) = (
        Surface::new(&params),
        Climate::new(&params),
        Vegetation::new(&params),
    );
    let (rivers, style) = (KnownRivers::default(), SurfaceStyle::default());
    let (water, snow_line) = (params.sea_level, 1.0);
    let mut cache = HeightCache::default();
    let mut grid = TerrainGrid::default();
    let square = ViewSquare {
        right: Vec2::X,
        towards: Vec2::Y,
        half: 8.0,
    };
    let centre = (40, -25);
    grid.fill(
        &mut cache,
        &surface,
        &climate,
        &vegetation,
        &rivers,
        &style,
        water,
        snow_line,
        (1.0, 0.0),
        centre,
        square,
        false,
    );
    // Both halves of a grid square, and a grid point itself.
    for (dx, dz) in [(0.25, 0.25), (0.8, 0.7), (-2.5, 3.1), (1.0, -1.0)] {
        let expected = grid.height_at(&grid.shown, Vec2::new(dx, dz));
        let at = shown_height_at(
            &mut cache,
            &surface,
            &climate,
            &vegetation,
            &rivers,
            &style,
            water,
            snow_line,
            (
                centre.0 as f64 + f64::from(dx),
                centre.1 as f64 + f64::from(dz),
            ),
        );
        assert!(
            (at - expected).abs() < 1e-3,
            "{at} vs {expected} at {dx},{dz}"
        );
    }
}
