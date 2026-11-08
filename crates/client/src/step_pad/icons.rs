//! The pad's pictures: a numeric keypad for the button that opens it, arrows,
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

/// A numeric keypad: a rounded frame round three columns of keys in four
/// rows, the bottom row a wide 0 and a dot.
pub(super) fn keypad() -> Image {
    const KEY: Vec2 = Vec2::new(10.0, 8.0);
    const GAP: Vec2 = Vec2::new(3.0, 4.0);
    const FIRST: Vec2 = Vec2::new(14.0, 10.0);
    picture(|at| {
        let frame = outside_box(at, Vec2::new(32.0, 32.0), Vec2::new(24.0, 28.0), 5.0);
        if frame.abs() <= 1.5 {
            return true;
        }
        (0..4).any(|row| {
            // The bottom row's first key is as wide as two.
            let keys: &[(f32, f32)] = if row == 3 {
                &[(0.0, 2.0), (2.0, 1.0)]
            } else {
                &[(0.0, 1.0), (1.0, 1.0), (2.0, 1.0)]
            };
            keys.iter().any(|&(column, wide)| {
                let corner = FIRST + Vec2::new(column, row as f32) * (KEY + GAP);
                let size = Vec2::new(KEY.x * wide + GAP.x * (wide - 1.0), KEY.y);
                outside_box(at, corner + size / 2.0, size / 2.0, 1.5) <= 0.0
            })
        })
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
