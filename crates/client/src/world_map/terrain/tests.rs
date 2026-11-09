use bevy::prelude::*;

use super::super::{HALF_SIZE, MAX_HALF, window_half};
use super::edges::crossings;
use super::rivers::clip;
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
    // The largest window turned 45°, where its bounding box is widest: its
    // chunks are still within what `TerrainGrid` keeps.
    let diagonal = Vec2::new(1.0, 1.0).normalize();
    let square = ViewSquare {
        right: diagonal,
        towards: Vec2::new(-diagonal.y, diagonal.x),
        half: window_half(10.0),
    };
    let centre = (1000, -37);
    let ((from_x, from_z), (to_x, to_z)) = chunks_area(&chunks_needed(square, centre));
    for reach in [
        centre.0 - from_x,
        to_x - centre.0,
        centre.1 - from_z,
        to_z - centre.1,
    ] {
        assert!(reach <= REACH + CHUNK, "{reach}");
    }
}

#[test]
fn the_chunks_needed_cover_the_window_and_little_else() {
    let turned = Vec2::new(0.6, 0.8);
    let square = ViewSquare {
        right: turned,
        towards: Vec2::new(-turned.y, turned.x),
        half: 70.0,
    };
    let centre = (-500, 333);
    let needed = chunks_needed(square, centre);
    let chunk_of = |x: f32, z: f32| {
        (
            (centre.0 + x.floor() as i64).div_euclid(CHUNK),
            (centre.1 + z.floor() as i64).div_euclid(CHUNK),
        )
    };
    for u in -10..=10 {
        for v in -10..=10 {
            let at = (square.right * u as f32 + square.towards * v as f32) * square.half / 10.0;
            assert!(needed.contains(&chunk_of(at.x, at.y)), "{u},{v}");
        }
    }
    // Fewer than the box round it, as its corners are cut off.
    let ((from_x, from_z), (to_x, to_z)) = chunks_area(&needed);
    let boxed = ((to_x - from_x) / CHUNK) * ((to_z - from_z) / CHUNK);
    assert!((needed.len() as i64) < boxed, "{} of {boxed}", needed.len());
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
    let (water, floor) = (params.sea_level, surface.floor());
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
        floor,
        (1.0, 0.0),
        centre,
        &[(
            (
                centre.0 - square.half as i64 - 3,
                centre.1 - square.half as i64 - 3,
            ),
            (
                centre.0 + square.half as i64 + 3,
                centre.1 + square.half as i64 + 3,
            ),
        )],
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
