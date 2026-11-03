//! The outline of each cut, seen from above.

use bevy::prelude::*;
use shared::item_form::Cut;
use std::f32::consts::TAU;

use super::{OUTLINE_POINTS, TRILLION_POINTS};

/// Where a cut's outline goes, clockwise from the top of the picture, in the flat
/// plane (`x`, `z`), with `-z` being away from the viewer.
pub(super) fn outline(cut: Cut) -> Vec<Vec2> {
    let round = |n: usize, shape: &dyn Fn(f32) -> f32, stretch: Vec2| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let angle = i as f32 / n as f32 * TAU;
                let along = Vec2::new(angle.sin(), -angle.cos());
                along * shape(angle) * stretch
            })
            .collect()
    };
    let circle = |_: f32| 1.0;
    match cut {
        Cut::Round => round(OUTLINE_POINTS, &circle, Vec2::ONE),
        Cut::Oval => round(OUTLINE_POINTS, &circle, Vec2::new(0.72, 1.0)),
        Cut::Cushion => {
            // A circle pushed out towards a square.
            let squarish = |angle: f32| {
                let (x, y) = (angle.sin().abs(), angle.cos().abs());
                0.92 * (x.powi(4) + y.powi(4)).powf(-0.25)
            };
            round(OUTLINE_POINTS, &squarish, Vec2::ONE)
        }
        Cut::Princess => {
            let square = |angle: f32| 0.8 / angle.sin().abs().max(angle.cos().abs());
            round(OUTLINE_POINTS, &square, Vec2::ONE)
        }
        Cut::Trillion => {
            // Between a triangle and a circle, corners first, mixed so that it can't
            // bend inwards.
            let triangle = |angle: f32| {
                let third = TAU / 3.0;
                let into_side = angle.rem_euclid(third) - third / 2.0;
                let side = (third / 2.0).cos() / into_side.cos();
                1.02 / (0.8 / side + 0.2)
            };
            round(TRILLION_POINTS, &triangle, Vec2::ONE)
        }
        Cut::Pear => pear(),
        Cut::Emerald => emerald_outline(),
        // Has no outline: see `uncut_mesh`.
        Cut::Uncut => Vec::new(),
    }
}

/// A point away from the viewer, its sides straight down to a round end.
pub(super) fn pear() -> Vec<Vec2> {
    const RADIUS: f32 = 0.68;
    let centre = Vec2::new(0.0, 0.32);
    let tip = Vec2::new(0.0, -1.0);
    // How far round the circle from straight at the tip the straight sides touch it.
    let touches = (RADIUS / (centre - tip).length()).acos();
    let mut points = vec![tip];
    let arc = OUTLINE_POINTS - 1;
    for i in 0..arc {
        let along = i as f32 / (arc - 1) as f32;
        let angle = touches + along * (TAU - 2.0 * touches);
        points.push(centre + RADIUS * Vec2::new(angle.sin(), -angle.cos()));
    }
    points
}

/// A long rectangle with its corners cut off.
pub(super) fn emerald_outline() -> Vec<Vec2> {
    const WIDE: f32 = 0.68;
    const LONG: f32 = 1.0;
    const CORNER: f32 = 0.2;
    vec![
        Vec2::new(-WIDE + CORNER, -LONG),
        Vec2::new(WIDE - CORNER, -LONG),
        Vec2::new(WIDE, -LONG + CORNER),
        Vec2::new(WIDE, LONG - CORNER),
        Vec2::new(WIDE - CORNER, LONG),
        Vec2::new(-WIDE + CORNER, LONG),
        Vec2::new(-WIDE, LONG - CORNER),
        Vec2::new(-WIDE, -LONG + CORNER),
    ]
}
