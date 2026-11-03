//! The blade itself: its faces along the length, and the marks on it.

use bevy::prelude::*;
use shared::blade::{BladeForm, Curve, Decoration, Point, Ricasso};

use super::{
    BLADE_STEPS, Model, Stuff, WAVY_STEPS, across_at, half_width, ricasso_length, section_points,
    thickness,
};

/// A blade from where it meets the handle, as long as a form says.
pub(super) fn add_blade(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    let section = section_points(form);
    let steps = if form.curve == Curve::Wavy {
        WAVY_STEPS
    } else {
        BLADE_STEPS
    };
    let thick = thickness(half);
    let rings: Vec<Vec<Vec3>> = (0..=steps)
        .map(|step| {
            let t = step as f32 / steps as f32;
            let at = across_at(form, t);
            let thin = ((at.back + at.edge) / (2.0 * half)).clamp(0.15, 1.0);
            section
                .iter()
                .map(|point| {
                    let x = -at.back + (at.back + at.edge) * (point.x + 1.0) / 2.0;
                    Vec3::new(at.middle + x, t * length, point.y * thick * thin)
                })
                .collect()
        })
        .collect();
    model.loft(Stuff::Blade, &rings);
    if form.point == Point::Notched {
        let end = across_at(form, 1.0);
        for side in [-0.55, 0.55] {
            let from = Vec2::new(end.middle + side * half, length);
            let to = from + Vec2::new(side * half * 0.5, half);
            model.spike(Stuff::Blade, from, to, half * 0.4);
        }
    }
    if form.fuller {
        add_along_blade(model, form, Stuff::Dark, (0.03, 0.72), 0.22, 1.05);
    }
    match form.decoration {
        Decoration::Engraved => add_marks(model, form),
        Decoration::Inlaid => add_along_blade(model, form, Stuff::Inlay, (0.05, 0.3), 0.12, 1.1),
        Decoration::Plain | Decoration::Tasselled => {}
    }
    if form.ricasso == Ricasso::WithHooks {
        let at = ricasso_length(form);
        for side in [-1.0, 1.0] {
            let from = Vec2::new(side * half * 0.5, at);
            let to = Vec2::new(side * half * 1.8, at - half * 0.5);
            model.spike(Stuff::Blade, from, to, 4.0);
        }
    }
}

/// A strip following a blade's middle, between two places along it.
pub(super) fn add_along_blade(
    model: &mut Model,
    form: &BladeForm,
    stuff: Stuff,
    between: (f32, f32),
    wide: f32,
    depth: f32,
) {
    let length = form.blade as f32;
    let half = half_width(form);
    let surface = thickness(half) * depth;
    for step in 0..12 {
        let place = |step: f32| between.0 + (between.1 - between.0) * step / 12.0;
        let (first, next) = (place(step as f32), place(step as f32 + 1.0));
        let centre = |t: f32| {
            let at = across_at(form, t);
            Vec2::new(at.middle + (at.edge - at.back) / 2.0, t * length)
        };
        let room = across_at(form, first);
        let strip = (room.back + room.edge) / 2.0 * wide;
        model.ribbon(stuff, centre(first), centre(next), strip, surface);
    }
}

/// Bands cut across a blade near its base.
pub(super) fn add_marks(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    let surface = thickness(half) * 1.05;
    for mark in 0..5 {
        let t = 0.08 + mark as f32 * 0.05;
        let at = across_at(form, t);
        let centre = at.middle + (at.edge - at.back) / 2.0;
        let reach = (at.back + at.edge) / 2.0 * 0.6;
        let y = t * length;
        let from = Vec2::new(centre - reach, y);
        let to = Vec2::new(centre + reach, y + reach * 0.5);
        model.ribbon(Stuff::Dark, from, to, 1.2, surface);
    }
}
