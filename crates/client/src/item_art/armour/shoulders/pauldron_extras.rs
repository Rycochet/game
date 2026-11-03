//! What pauldrons may have besides, hanging pauldrons and pteruges.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::shoulders::{Cut, HautePiece, ShouldersForm, Wings};
use shared::body::{Frame, Side};
use std::f32::consts::PI;

use super::super::surface::{lines, patch, scales};
use super::super::{Model, Stuff};
use super::{Arm, Look, RIGID_CLEAR, arm_surface, cop_surface, sheet, shoulder_top, wing_surface};

/// The wings over breast and back, and the upper lames, haute-piece and gardbrace.
#[allow(clippy::too_many_arguments)]
pub(super) fn add_pauldron_extras(
    model: &mut Model,
    frame: &Frame,
    arm: &Arm,
    look: Look,
    wings: Wings,
    cut: Cut,
    upper_lames: u8,
    haute_piece: HautePiece,
    gardbrace: bool,
) {
    let joint = frame.shoulder_half as f32;
    let great = cut == Cut::GreatLeft && arm.side == Side::Left;
    let lance = cut == Cut::LanceCut && arm.side == Side::Right;
    let small_cut = cut == Cut::GreatLeft && arm.side == Side::Right;
    let inner = match (wings, great, small_cut) {
        (_, true, _) => joint * 0.15,
        (_, _, true) => joint * 0.6,
        (Wings::Large, ..) => joint * 0.32,
        (Wings::Small, ..) => joint * 0.58,
    };
    let deeper = if great { 40.0 } else { 0.0 };
    let top = frame.shoulder as f32 + 12.0;
    let outer = joint - 15.0;
    let front_inner = if lance { joint * 0.8 } else { inner };
    for (front, drop, from) in [
        (true, 170.0 + deeper, front_inner),
        (false, 200.0 + deeper, inner),
    ] {
        let wing = wing_surface(frame, arm.sign, front, (outer, from), top, drop);
        sheet(model, Stuff::Main, &wing, look.thick);
        if look.rim > 0.0 {
            patch(
                model,
                Stuff::Main,
                &wing,
                (0.0, 1.0),
                (0.9, 1.0),
                look.rim,
                look.thick,
            );
        }
        if look.etched {
            patch(
                model,
                Stuff::Etch,
                &wing,
                (0.0, 1.0),
                (0.7, 0.82),
                look.thick / 2.0 + 0.3,
                0.6,
            );
        }
    }
    // Upper lames, over the trunk towards the neck.
    let neck_side = frame.neck as f32 * 2.2;
    for lame in 0..upper_lames {
        let from = joint - 25.0 - lame as f32 * 22.0;
        let to = (from - 28.0).max(neck_side);
        let surface = |u: f32, v: f32| {
            let x = from + (to - from) * v;
            let z = (u * 2.0 - 1.0) * 75.0;
            let y = shoulder_top(frame, x) + RIGID_CLEAR
                - 4.0 * lame as f32
                - (z / 75.0).powi(2) * 22.0;
            (
                Vec3::new(arm.sign * x, y, z),
                Vec3::new(0.0, 1.0, z * 0.006).normalize(),
            )
        };
        sheet(model, Stuff::Main, &surface, look.thick);
    }
    if haute_piece != HautePiece::None {
        let high = haute_piece.height() as f32 * frame.elbow_length as f32 / 300.0;
        let surface = |u: f32, v: f32| {
            let angle = (u - 0.5) * 2.4;
            // Rising from inside the cop's inner edge.
            let x = joint - 52.0 * angle.cos();
            let z = 80.0 * angle.sin();
            let base = shoulder_top(frame, x) + RIGID_CLEAR - 22.0;
            let point = Vec3::new(arm.sign * (x - v * 6.0), base + v * (high + 22.0), z);
            (
                point,
                Vec3::new(-arm.sign * angle.cos(), 0.0, -angle.sin()).normalize(),
            )
        };
        sheet(model, Stuff::Main, &surface, look.thick);
    }
    if gardbrace {
        let across = arm.radius + RIGID_CLEAR + 7.0;
        let surface = cop_surface(arm, look, (0.0, 1.9), (across, across - 12.0), PI / 2.0);
        let part = |u: f32, v: f32| surface(u, 0.3 + 0.7 * v);
        sheet(model, Stuff::Main, &part, look.thick);
    }
}

/// Pauldrons of scales or plated mail hang in one piece from a small cop.
pub(super) fn add_hanging_pauldron(model: &mut Model, form: &ShouldersForm, arm: &Arm, look: Look) {
    let across = arm.radius + RIGID_CLEAR - 8.0;
    let cop = cop_surface(arm, look, (-2.2, 2.2), (across, across - 10.0), PI / 2.0);
    let band = arm_surface(arm, look, (-2.2, 2.2), (-6.0, arm.reach * 1.1), across);
    match form.construction {
        Construction::Scale => {
            sheet(model, Stuff::Backing, &cop, 1.5);
            sheet(model, Stuff::Backing, &band, 1.5);
            scales(model, Stuff::Main, &cop, 4, 14);
            scales(model, Stuff::Main, &band, 6, 16);
        }
        _ => {
            let small = cop_surface(arm, look, (-1.8, 1.8), (across + 4.0, across - 6.0), 1.1);
            sheet(model, Stuff::Main, &cop, 1.5);
            lines(model, &cop, 8, 0.8);
            sheet(model, Stuff::Main, &small, 2.0);
            sheet(model, Stuff::Main, &band, 1.5);
            lines(model, &band, 10, 0.8);
            // Small plates set in the mail, in rows.
            for row in 0..3 {
                let v = 0.12 + row as f32 * 0.3;
                for column in 0..6 {
                    let u = 0.08 + column as f32 * 0.17;
                    patch(
                        model,
                        Stuff::Main,
                        &band,
                        (u, u + 0.12),
                        (v, v + 0.2),
                        1.6,
                        1.6,
                    );
                }
            }
        }
    }
}

/// Spaulders' pteruges: strips hanging below the last lame, in one or two rows.
pub(super) fn add_pteruges(model: &mut Model, arm: &Arm, rows: u8, radius: f32) {
    for row in 0..rows {
        let long = 85.0 + row as f32 * 30.0;
        let radius = radius - 4.0 - row as f32 * 4.0;
        let count = 9;
        for strip in 0..count {
            let middle = -1.7 + 3.4 * (strip as f32 + 0.5 * row as f32) / count as f32;
            let half = 13.0 / radius;
            let surface = |u: f32, v: f32| {
                arm.round(
                    arm.reach - 10.0 + v * long,
                    middle + (u * 2.0 - 1.0) * half,
                    radius,
                )
            };
            sheet(model, Stuff::Backing, &surface, 2.0);
        }
    }
}
