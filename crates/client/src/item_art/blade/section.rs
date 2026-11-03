//! How wide and thick a blade is along its length, and the shape of its cross-section.

use bevy::prelude::*;
use shared::blade::{BladeForm, BladeShape, BladeWidth, Curve, Edges, Point, Ricasso, Section};

pub(super) fn half_width(form: &BladeForm) -> f32 {
    match form.width {
        BladeWidth::Narrow => 12.0,
        BladeWidth::Normal => 22.0,
        BladeWidth::Broad => 32.0,
    }
}

/// How thick a blade is at its middle, against its full width.
pub(super) fn thickness(half: f32) -> f32 {
    (half * 0.22).max(2.5)
}

/// Round a blade cut across: from its back at -1 to its edge at 1, thickness from -1 to 1.
pub(super) fn section_points(form: &BladeForm) -> Vec<Vec2> {
    let points: &[(f32, f32)] = if form.edges == Edges::Single {
        &[
            (1.0, 0.0),
            (0.0, 0.9),
            (-1.0, 0.7),
            (-1.0, -0.7),
            (0.0, -0.9),
        ]
    } else {
        match form.section {
            Section::Flat => &[
                (1.0, 0.0),
                (0.6, 0.6),
                (-0.6, 0.6),
                (-1.0, 0.0),
                (-0.6, -0.6),
                (0.6, -0.6),
            ],
            Section::Lenticular => &[
                (1.0, 0.0),
                (0.5, 0.75),
                (0.0, 1.0),
                (-0.5, 0.75),
                (-1.0, 0.0),
                (-0.5, -0.75),
                (0.0, -1.0),
                (0.5, -0.75),
            ],
            Section::Diamond => &[(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)],
            Section::Midrib => &[
                (1.0, 0.0),
                (0.2, 0.4),
                (0.0, 1.1),
                (-0.2, 0.4),
                (-1.0, 0.0),
                (-0.2, -0.4),
                (0.0, -1.1),
                (0.2, -0.4),
            ],
            Section::Triangular => &[(1.0, -0.7), (0.0, 1.2), (-1.0, -0.7)],
        }
    };
    points.iter().map(|&(x, z)| Vec2::new(x, z)).collect()
}

/// A blade at one place along it: its back and edge from its middle, and that middle.
pub(super) struct Across {
    pub(super) back: f32,
    pub(super) edge: f32,
    pub(super) middle: f32,
}

/// How far along a blade its plain part below the sharpened part reaches.
pub(super) fn ricasso_length(form: &BladeForm) -> f32 {
    match form.ricasso {
        Ricasso::None => 0.0,
        Ricasso::Plain | Ricasso::WithHooks => (form.blade as f32 * 0.14).clamp(25.0, 180.0),
    }
}

pub(super) fn across_at(form: &BladeForm, t: f32) -> Across {
    let length = form.blade as f32;
    let half = half_width(form);
    let bend = match form.curve {
        Curve::Straight => 0.0,
        Curve::Slight => -(length * 0.07).min(60.0) * t * t,
        Curve::Strong => -(length * 0.15).min(110.0) * t * t,
        Curve::Inward => (length * 0.18).min(90.0) * t * t,
        Curve::Wavy => half * 0.4 * (t * (length / 45.0).max(4.0)).sin(),
    };
    let body = match form.shape {
        BladeShape::Parallel => 1.0,
        BladeShape::Tapered => 1.0 - 0.5 * t,
        BladeShape::Leaf => 0.7 + 0.5 * (t * std::f32::consts::PI * 0.85).sin(),
        BladeShape::Widening => 0.7 + 0.55 * t,
    };
    let plain = t * length < ricasso_length(form);
    let wide = half * body * if plain { 0.55 } else { 1.0 };
    let tip = (half * 3.0).min(length * 0.3);
    let near_end = ((t * length - (length - tip)) / tip).clamp(0.0, 1.0);
    let single = form.edges == Edges::Single;
    let (back, edge, hook) = match form.point {
        Point::Sharp if single => (wide, wide * (1.0 - 2.0 * near_end), 0.0),
        Point::Sharp => (wide * (1.0 - near_end), wide * (1.0 - near_end), 0.0),
        Point::Rounded => {
            let left = (1.0 - near_end * near_end).max(0.0).sqrt();
            (wide * left, wide * left, 0.0)
        }
        Point::None | Point::Notched => (wide, wide, 0.0),
        Point::Hooked => {
            let left = 1.0 - near_end;
            (wide * left, wide * left, near_end * near_end * half * 1.4)
        }
        Point::Clip => (
            wide * (1.0 - near_end),
            wide * (1.0 - near_end * near_end),
            0.0,
        ),
        Point::Flared => (
            wide * (1.0 + 0.5 * near_end),
            wide * (1.0 + 0.5 * near_end),
            0.0,
        ),
    };
    Across {
        back,
        edge,
        middle: bend + hook,
    }
}
