use bevy::prelude::*;

use super::super::HALF_SIZE;
use super::edges::crossings;
use super::*;

#[test]
fn grid_reaches_the_window_corners() {
    let corner = HALF_SIZE as f32 * std::f32::consts::SQRT_2;
    // The corner plus one square for the triangles around it.
    assert!(REACH as f32 >= corner + 1.0);
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
