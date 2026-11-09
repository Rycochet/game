use bevy::prelude::*;

use super::super::super::WALL_HEIGHT;
use super::super::{HALF_CELL, WALL_OUT, WINDOW_HALF, WINDOW_SILL, WINDOW_TOP, cell_centre};
use super::{Opening, Piece};

/// Facing south, `(0, 1)`.
fn opening(door: bool) -> Opening {
    let (half, low, high) = if door {
        (HALF_CELL, 0.0, WALL_HEIGHT)
    } else {
        (WINDOW_HALF, WINDOW_SILL, WINDOW_TOP)
    };
    Opening {
        faces: (0, 1),
        door,
        half,
        low,
        high,
    }
}

/// The south face of cell (2, 3) from `a` to `b` east of its middle.
fn south_face(a: f32, b: f32) -> Piece {
    let middle = cell_centre(2, 3) + Vec3::Z * WALL_OUT;
    Piece {
        from: middle + Vec3::X * a,
        to: middle + Vec3::X * b,
        normal: Vec3::Z,
        across: (0.0, 1.0),
    }
}

#[test]
fn a_door_leaves_nothing_of_the_wall_on_its_side() {
    let door = opening(true);
    assert!(door.around(&south_face(-HALF_CELL, 0.0), (2, 3)).is_empty());
    assert!(
        door.around(&south_face(WALL_OUT, HALF_CELL), (2, 3))
            .is_empty()
    );
}

#[test]
fn other_sides_are_left_whole() {
    let door = opening(true);
    let mut east = south_face(0.0, HALF_CELL);
    east.normal = Vec3::X;
    let parts = door.around(&east, (2, 3));
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].1, (0.0, WALL_HEIGHT));
}

/// Nothing drawn covers the window's pane, and the wall is all there round it.
#[test]
fn a_window_is_left_a_hole_and_the_wall_goes_round_it() {
    let window = opening(false);
    for (a, b) in [(-HALF_CELL, 0.0), (0.0, HALF_CELL), (HALF_CELL, -HALF_CELL)] {
        let parts = window.around(&south_face(a, b), (2, 3));
        let middle = cell_centre(2, 3).x;
        let mut area = 0.0;
        for (part, (low, high)) in &parts {
            let (x0, x1) = (part.from.x - middle, part.to.x - middle);
            let (x0, x1) = (x0.min(x1), x0.max(x1));
            let overlaps_across = x1 > -WINDOW_HALF + 1e-3 && x0 < WINDOW_HALF - 1e-3;
            let overlaps_up = *high > WINDOW_SILL + 1e-3 && *low < WINDOW_TOP - 1e-3;
            assert!(
                !(overlaps_across && overlaps_up),
                "{a}..{b}: {part:?} {low}..{high}"
            );
            area += (x1 - x0) * (high - low);
        }
        let (low, high) = (a.min(b), a.max(b));
        let across = high.min(WINDOW_HALF) - low.max(-WINDOW_HALF);
        let hole = across * (WINDOW_TOP - WINDOW_SILL);
        let expected = (b - a).abs() * WALL_HEIGHT - hole;
        assert!(
            (area - expected).abs() < 1e-2,
            "{a}..{b}: {area} not {expected}"
        );
    }
}

/// A part keeps its place in the wall's picture, so the bricks line up.
#[test]
fn parts_keep_the_picture_lined_up() {
    let window = opening(false);
    let parts = window.around(&south_face(-HALF_CELL, 0.0), (2, 3));
    let left = parts
        .iter()
        .find(|(_, heights)| *heights == (WINDOW_SILL, WINDOW_TOP))
        .expect("a part beside the window");
    let share = (HALF_CELL - WINDOW_HALF) / HALF_CELL;
    assert!((left.0.across.0 - 0.0).abs() < 1e-5);
    assert!((left.0.across.1 - share).abs() < 1e-5);
}
