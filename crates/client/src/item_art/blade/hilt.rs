//! The hilt: handle, guard and pommel.

use bevy::prelude::*;
use shared::blade::{BladeForm, Guard, HandleShape, Pommel};

use super::{HANDLE_RADIUS, HANDLE_STEPS, Model, Stuff, half_width};

/// Where a handle's line is, from 0 where it meets the blade to 1 at its end.
pub(super) fn handle_at(form: &BladeForm, along: f32) -> Vec2 {
    let length = form.handle as f32;
    let reach = length.min(500.0);
    let x = match form.handle_shape {
        HandleShape::Straight => 0.0,
        HandleShape::Curved => -reach * 0.14 * along * along,
        HandleShape::SCurved => reach * 0.07 * (along * std::f32::consts::TAU).sin(),
        HandleShape::Angled => -reach * 0.35 * (along - 0.3).max(0.0),
    };
    Vec2::new(x, -along * length)
}

/// The handle, with its fittings at each end.
pub(super) fn add_handle(model: &mut Model, form: &BladeForm) {
    let path: Vec<(Vec2, f32)> = (0..=HANDLE_STEPS)
        .map(|step| {
            let along = step as f32 / HANDLE_STEPS as f32;
            (handle_at(form, along), HANDLE_RADIUS)
        })
        .collect();
    model.tube(Stuff::Handle, &path, 0.0);
    let end = handle_at(form, 1.0);
    if form.pommel == Pommel::None {
        let cap = (end, HANDLE_RADIUS + 1.5);
        let inner = (
            handle_at(form, 1.0 - 18.0 / form.handle as f32),
            HANDLE_RADIUS + 1.5,
        );
        model.tube(Stuff::Fittings, &[inner, cap], 0.0);
    }
    if form.handle < 400 {
        // Bands of wrapping round a grip.
        for band in 1..form.handle / 30 {
            let along = band as f32 * 30.0 / form.handle as f32;
            let width = 5.0 / form.handle as f32;
            let path = [
                (handle_at(form, along - width), HANDLE_RADIUS + 0.8),
                (handle_at(form, along + width), HANDLE_RADIUS + 0.8),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

pub(super) fn add_guard(model: &mut Model, form: &BladeForm) {
    let half = half_width(form);
    let reach = (form.blade as f32 * 0.16).clamp(half + 12.0, 80.0);
    let grip = form.handle as f32;
    match form.guard {
        Guard::None => {}
        Guard::Cross => add_cross(model, reach),
        Guard::Disc => model.tube(
            Stuff::Fittings,
            &[
                (Vec2::new(0.0, 3.0), reach * 0.85),
                (Vec2::new(0.0, -3.0), reach * 0.85),
            ],
            0.0,
        ),
        Guard::Cup => add_cup(model, reach),
        Guard::KnuckleBow => {
            add_cross(model, reach);
            add_bow(model, form, reach, 0.0);
        }
        Guard::Basket => {
            add_cup(model, reach);
            for depth in [-12.0, 0.0, 12.0] {
                add_bow(model, form, reach, depth);
            }
            let back = [
                (Vec2::new(-reach * 0.7, 0.0), 2.5),
                (Vec2::new(-reach * 0.75, -grip * 0.4), 2.5),
                (Vec2::new(-5.0, -grip * 0.9), 2.5),
            ];
            model.tube(Stuff::Fittings, &back, 0.0);
        }
    }
}

pub(super) fn add_cross(model: &mut Model, reach: f32) {
    let path = [
        (Vec2::new(-reach, 0.0), 6.0),
        (Vec2::new(0.0, 0.0), 7.0),
        (Vec2::new(reach, 0.0), 6.0),
    ];
    model.tube(Stuff::Fittings, &path, 0.0);
}

pub(super) fn add_cup(model: &mut Model, reach: f32) {
    let path = [
        (Vec2::new(0.0, 16.0), reach * 0.9),
        (Vec2::new(0.0, 6.0), reach * 0.75),
        (Vec2::new(0.0, -4.0), HANDLE_RADIUS + 2.0),
    ];
    model.tube(Stuff::Fittings, &path, 0.0);
}

/// A curved bar from the guard round the fingers to the pommel.
pub(super) fn add_bow(model: &mut Model, form: &BladeForm, reach: f32, depth: f32) {
    let grip = form.handle as f32;
    let end = handle_at(form, 0.95);
    let path = [
        (Vec2::new(reach * 0.8, 0.0), 3.0),
        (Vec2::new(reach * 1.05, -grip * 0.3), 3.0),
        (Vec2::new(reach * 0.9, -grip * 0.7), 3.0),
        (Vec2::new(end.x + 10.0, end.y), 3.0),
    ];
    model.tube(Stuff::Fittings, &path, depth);
}

pub(super) fn add_pommel(model: &mut Model, form: &BladeForm) {
    let end = handle_at(form, 1.0);
    match form.pommel {
        Pommel::None => {}
        Pommel::Plain => {
            let path = [
                (end + Vec2::Y * 2.0, 8.0),
                (end - Vec2::Y * 8.0, 19.0),
                (end - Vec2::Y * 20.0, 16.0),
                (end - Vec2::Y * 28.0, 6.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
        Pommel::Ring => {
            let centre = end - Vec2::Y * 20.0;
            let circle: Vec<(Vec2, f32)> = (0..=16)
                .map(|step| {
                    let angle = step as f32 / 16.0 * std::f32::consts::TAU;
                    (centre + Vec2::from_angle(angle) * 19.0, 5.0)
                })
                .collect();
            model.tube(Stuff::Fittings, &circle, 0.0);
        }
        Pommel::Wheel => model.disc(Stuff::Fittings, end - Vec2::Y * 14.0, 24.0, 9.0),
        Pommel::Disc => model.disc(Stuff::Fittings, end - Vec2::Y * 16.0, 28.0, 4.0),
        Pommel::Hooked => {
            let path = [
                (end + Vec2::Y * 4.0, 11.0),
                (end + Vec2::new(-12.0, -10.0), 10.0),
                (end + Vec2::new(-30.0, -12.0), 8.0),
                (end + Vec2::new(-42.0, -2.0), 5.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}
