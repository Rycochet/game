//! Rings of points, sizes and hashed numbers models are built from.

use bevy::prelude::*;
use shared::site::Outline;
use std::f32::consts::TAU;

use super::CELLS_AT_REACH;

/// Twice the signed area of a triangle.
pub(super) fn edge(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).perp_dot(c - a)
}

pub(super) fn rectangle(half: Vec2) -> Vec<Vec2> {
    vec![
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        half,
        Vec2::new(-half.x, half.y),
    ]
}

/// A ring of points round an ellipse.
pub(super) fn circle(half: Vec2, points: usize) -> Vec<Vec2> {
    (0..points)
        .map(|i| {
            let angle = i as f32 / points as f32 * TAU;
            Vec2::new(angle.cos() * half.x, angle.sin() * half.y)
        })
        .collect()
}

/// The ring of a building's outline.
pub(super) fn ring(outline: Outline, half: Vec2) -> Vec<Vec2> {
    let cut = half.min_element() * 0.35;
    match outline {
        Outline::Rectangle => rectangle(half),
        Outline::CutCorners => vec![
            Vec2::new(-half.x + cut, -half.y),
            Vec2::new(half.x - cut, -half.y),
            Vec2::new(half.x, -half.y + cut),
            Vec2::new(half.x, half.y - cut),
            Vec2::new(half.x - cut, half.y),
            Vec2::new(-half.x + cut, half.y),
            Vec2::new(-half.x, half.y - cut),
            Vec2::new(-half.x, -half.y + cut),
        ],
        Outline::RoundCorners | Outline::Circle => circle(half, 12),
    }
}

/// Half a building's width on the model, from its width in cells.
pub(super) fn half_size(cells: u16) -> f32 {
    let share = (f32::from(cells) / CELLS_AT_REACH).min(1.0);
    2.5 + share * 3.7
}

/// The next of a run of numbers from `0..1`. Only for how a model looks.
pub(super) fn hashed(number: &mut u64) -> f32 {
    *number = number
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (*number >> 40) as f32 / (1u64 << 24) as f32
}
