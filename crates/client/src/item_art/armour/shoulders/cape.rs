//! Capes and yokes.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::shoulders::{Finish, Length, ShouldersForm};
use shared::body::{Frame, Side};
use std::f32::consts::{PI, TAU};

use super::super::surface::{grid, lines, scales, studs};
use super::super::{Model, Stuff, plate};
use super::{DRAPE_CLEAR, OPENING, shoulder_top};
use crate::item_art::body::trunk_depth;

/// A cape over both shoulders: rings from the neck out over the shoulders and down.
pub(super) fn add_cape(model: &mut Model, form: &ShouldersForm, frame: &Frame, length: Length) {
    let f = |value: i32| value as f32;
    let spread = f(frame.arm_spread) / 1000.0;
    let down = match length {
        Length::Short => 60.0,
        Length::Long => 150.0,
    } * f(frame.elbow_length)
        / 300.0;
    let bottom = f(frame.shoulder) - down;
    // Over the shoulders' tops, rising a little round the neck as a collar.
    let over = f(frame.shoulder) + f(frame.upper_arm) * 1.15 + DRAPE_CLEAR;
    let rings = [
        (over + 22.0, (f(frame.neck) * 1.45, f(frame.neck) * 1.3)),
        (
            over,
            (
                f(frame.shoulder_half) + 15.0,
                f(frame.chest_half.1) * 0.75 + DRAPE_CLEAR,
            ),
        ),
        (
            bottom,
            (
                f(frame.shoulder_half) + down * spread + f(frame.upper_arm) + DRAPE_CLEAR + 10.0,
                f(frame.chest_half.1) + DRAPE_CLEAR + 15.0,
            ),
        ),
    ];
    let surface = |u: f32, v: f32| {
        // Open at the front, where it hooks together.
        let angle = PI / 2.0 + OPENING + u * (TAU - 2.0 * OPENING);
        let (from, to, t) = if v < 0.3 {
            (rings[0], rings[1], v / 0.3)
        } else {
            (rings[1], rings[2], (v - 0.3) / 0.7)
        };
        // Longest over the arms, shorter in front and behind.
        let y = from.0
            + (to.0 - from.0) * t
            + if v > 0.3 {
                35.0 * angle.sin().abs() * t
            } else {
                0.0
            };
        let across = from.1.0 + (to.1.0 - from.1.0) * t;
        let deep = from.1.1 + (to.1.1 - from.1.1) * t;
        let point = Vec3::new(across * angle.cos(), y, deep * angle.sin());
        let normal = Vec3::new(
            angle.cos() / across * 100.0,
            (1.0 - v) * 0.9,
            angle.sin() / deep * 100.0,
        )
        .normalize();
        (point, normal)
    };
    let mesh = grid(&surface, 10, 40);
    match form.construction {
        Construction::Scale => {
            plate(model.of(Stuff::Backing), &mesh, 1.5, false);
            for part in 0..4 {
                let u0 = part as f32 / 4.0;
                let quarter = |u: f32, v: f32| surface(u0 + u / 4.0, 0.12 + v * 0.88);
                scales(model, Stuff::Main, &quarter, 7, 9);
            }
        }
        Construction::Leather => {
            plate(model.of(Stuff::Main), &mesh, 3.0, false);
            for part in 0..4 {
                let u0 = part as f32 / 4.0;
                let quarter = |u: f32, v: f32| surface(u0 + u / 4.0, v);
                lines(model, &quarter, 14, 2.0);
            }
        }
        _ => {
            plate(model.of(Stuff::Main), &mesh, 2.0, false);
            for part in 0..4 {
                let u0 = part as f32 / 4.0;
                let quarter = |u: f32, v: f32| surface(u0 + u / 4.0, v);
                lines(model, &quarter, 16, 1.2);
            }
        }
    }
    if form.trim.is_some() {
        let edge = |u: f32, v: f32| surface(u, 0.94 + v * 0.06);
        plate(model.of(Stuff::Trim), &grid(&edge, 2, 40), 3.0, false);
    }
}

/// The top of a linothorax: a band from the back over each shoulder, tied down in front.
pub(super) fn add_yoke(model: &mut Model, form: &ShouldersForm, frame: &Frame, length: Length) {
    let f = |value: i32| value as f32;
    let joint = f(frame.shoulder_half);
    let low = f(frame.shoulder)
        - match length {
            Length::Short => 80.0,
            Length::Long => 110.0,
        };
    for side in Side::BOTH {
        let sign = side.sign() as f32;
        let surface = |u: f32, v: f32| {
            let x = joint * (0.42 + 0.46 * u);
            let angle = (v - 0.5) * PI;
            let top = shoulder_top(frame, x) + DRAPE_CLEAR;
            let y = low + (top - low) * angle.cos();
            let z = angle.sin() * (trunk_depth(frame, x, y) + DRAPE_CLEAR);
            (
                Vec3::new(sign * x, y, z),
                Vec3::new(0.0, angle.cos(), angle.sin()).normalize(),
            )
        };
        let thick = if form.construction == Construction::Padded {
            5.0
        } else {
            3.5
        };
        plate(model.of(Stuff::Main), &grid(&surface, 16, 6), thick, false);
        if form.finish == Finish::Studded {
            studs(model, Stuff::Rivet, &surface, (8, 2), thick / 2.0, 3.0);
        } else if form.finish == Finish::Tooled || form.construction == Construction::Padded {
            lines(model, &|u, v| surface(v, u), 4, thick / 2.0);
        }
    }
}
