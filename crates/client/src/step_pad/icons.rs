//! The pad's pictures: a d-pad for the button that opens it, arrows,
//! and ladders in perspective for Up (narrow at the top, climbing away) and
//! Down (narrow at the bottom, going down away).

use bevy::prelude::*;

use super::picture::{SIZE, on_line, outside_box, picture};

/// Which way an arrow points on the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pointing {
    Up,
    Down,
    Left,
    Right,
}

/// A d-pad: a cross with rounded ends, a small triangle cut into each arm
/// pointing out.
pub(super) fn dpad() -> Image {
    const MIDDLE: Vec2 = Vec2::new(32.0, 32.0);
    // Each arm, from the middle: half as wide as it is, and how far it reaches.
    const HALF_WIDTH: f32 = 9.0;
    const REACH: f32 = 28.0;
    picture(|at| {
        let long = Vec2::new(HALF_WIDTH, REACH);
        let cross = outside_box(at, MIDDLE, long, 3.0).min(outside_box(at, MIDDLE, long.yx(), 3.0));
        if cross > 0.0 {
            return false;
        }
        // The triangles, in the arm's own terms: `out` from the middle, `side`
        // across, mirrored so one test covers all four.
        let off = at - MIDDLE;
        let (out, side) = if off.x.abs() > off.y.abs() {
            (off.x.abs(), off.y.abs())
        } else {
            (off.y.abs(), off.x.abs())
        };
        let in_triangle = (12.0..=22.0).contains(&out) && side <= (22.0 - out) * 0.55;
        !in_triangle
    })
}

/// An arrow with a wide head, pointing this way.
pub(super) fn arrow(pointing: Pointing) -> Image {
    let far = SIZE as f32;
    picture(|at| {
        // Turned so it points up, the head's tip at the top.
        let up = match pointing {
            Pointing::Up => at,
            Pointing::Down => Vec2::new(at.x, far - at.y),
            Pointing::Left => Vec2::new(at.y, at.x),
            Pointing::Right => Vec2::new(at.y, far - at.x),
        };
        let head = up.y >= 10.0 && up.y <= 32.0 && (up.x - 32.0).abs() <= (up.y - 10.0) * 0.85;
        let shaft = (up.x - 32.0).abs() <= 6.0 && (30.0..=54.0).contains(&up.y);
        head || shaft
    })
}

/// A curved arrow turning over the top: anticlockwise for Turn Left, its head
/// on the left pointing down, and clockwise for Turn Right.
pub(super) fn turn(left: bool) -> Image {
    const MIDDLE: Vec2 = Vec2::new(32.0, 38.0);
    const RADIUS: f32 = 19.0;
    const HALF_THICK: f32 = 3.5;
    let far = SIZE as f32;
    picture(|at| {
        // Drawn as Turn Left, mirrored for Turn Right.
        let at = if left {
            at
        } else {
            Vec2::new(far - at.x, at.y)
        };
        let off = at - MIDDLE;
        // The arc over the top, from the right round to near the left end.
        let angle = off.y.atan2(off.x).to_degrees();
        let arc = (off.length() - RADIUS).abs() <= HALF_THICK && (-160.0..=-10.0).contains(&angle);
        // The head at the left end, pointing down.
        let tip = Vec2::new(MIDDLE.x - RADIUS, MIDDLE.y + 12.0);
        let down = tip.y - at.y;
        let head = (0.0..=14.0).contains(&down) && (at.x - tip.x).abs() <= down * 0.75;
        arc || head
    })
}

/// A ladder in perspective: its rails closing in towards the far end, and its
/// rungs closer together there. `climbing` puts the far end at the top (Up),
/// otherwise at the bottom (Down).
pub(super) fn ladder(climbing: bool) -> Image {
    const NEAR: (f32, f32) = (58.0, 21.0);
    const FAR: (f32, f32) = (6.0, 8.0);
    const RUNGS: usize = 5;
    // Across the rails at the far end is to across at the near end as the near
    // end's distance is to the far end's: rungs evenly spaced in distance.
    let (near_depth, far_depth) = (1.0 / NEAR.1, 1.0 / FAR.1);
    let rungs: Vec<(f32, f32)> = (0..RUNGS)
        .map(|rung| {
            let depth = near_depth + (rung as f32 + 0.5) / RUNGS as f32 * (far_depth - near_depth);
            let half = 1.0 / depth;
            let share = (NEAR.1 - half) / (NEAR.1 - FAR.1);
            (NEAR.0 + (FAR.0 - NEAR.0) * share, half)
        })
        .collect();
    let far = SIZE as f32;
    picture(|at| {
        let at = if climbing {
            at
        } else {
            Vec2::new(at.x, far - at.y)
        };
        let rail = |side: f32| {
            let from = Vec2::new(32.0 + side * NEAR.1, NEAR.0);
            (from, Vec2::new(32.0 + side * FAR.1, FAR.0))
        };
        if on_line(at, rail(-1.0), 4.0) || on_line(at, rail(1.0), 4.0) {
            return true;
        }
        rungs.iter().any(|&(y, half)| {
            // Thinner the further away.
            let thickness = (3.5 * half / NEAR.1).max(1.5);
            on_line(
                at,
                (Vec2::new(32.0 - half, y), Vec2::new(32.0 + half, y)),
                thickness,
            )
        })
    })
}

#[cfg(test)]
mod tests;
