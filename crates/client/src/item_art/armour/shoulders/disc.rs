//! Shoulder discs, ailettes and wings.

use bevy::prelude::*;
use shared::armour::shoulders::{AiletteShape, Charge, ShouldersForm};
use shared::armour::{Construction, Style};
use shared::body::Frame;
use shared::material::Material;
use std::f32::consts::{PI, TAU};

use super::super::surface::{bands, grid, lines, patch, ring_grid};
use super::super::{Model, Stuff, plate};
use super::{Arm, RIGID_CLEAR, add_disc, arm_surface, cop_surface, look_of, sheet, shoulder_top};
use crate::item_art::body::blob;

/// Where a shoulder disc's middle is, and which way it faces: out and a little up.
pub(super) fn disc_place(arm: &Arm) -> (Vec3, Vec3) {
    let centre = arm.joint + arm.out * (arm.radius + RIGID_CLEAR - 6.0) - arm.down * 8.0;
    (centre, (arm.out - arm.down * 0.35).normalize())
}

pub(super) fn add_shoulder_disc(
    model: &mut Model,
    form: &ShouldersForm,
    arm: &Arm,
    diameter: f32,
    boss: bool,
) {
    let (centre, facing) = disc_place(arm);
    let radius = diameter / 2.0;
    if form.construction == Construction::MailAndPlate {
        let mail = arm_surface(
            arm,
            look_of(Style::White),
            (-2.0, 2.0),
            (-20.0, arm.reach),
            arm.radius + 20.0,
        );
        sheet(model, Stuff::Shade, &mail, 1.5);
        lines(model, &mail, 10, 0.8);
        let cop = cop_surface(
            arm,
            look_of(Style::White),
            (-2.0, 2.0),
            (arm.radius + 20.0, arm.radius + 10.0),
            PI / 2.0,
        );
        sheet(model, Stuff::Shade, &cop, 1.5);
        lines(model, &cop, 6, 0.8);
    }
    add_disc(
        model,
        Stuff::Main,
        centre,
        facing,
        radius,
        radius * 0.15,
        2.2,
    );
    let rim = if form.trim.is_some() {
        Stuff::Trim
    } else {
        Stuff::Main
    };
    let surface = |u: f32, v: f32| {
        let up = (Vec3::Y - facing * Vec3::Y.dot(facing)).normalize();
        let side = facing.cross(up);
        let angle = u * TAU;
        let out = radius * (0.88 + 0.12 * v);
        (
            centre + (up * angle.cos() + side * angle.sin()) * out + facing * 2.0,
            facing,
        )
    };
    plate(model.of(rim), &ring_grid(&surface, 1, 28), 3.0, true);
    if form.construction == Construction::Mirror {
        let inner = |u: f32, v: f32| {
            let up = (Vec3::Y - facing * Vec3::Y.dot(facing)).normalize();
            let side = facing.cross(up);
            let angle = u * TAU;
            let out = radius * (0.5 + 0.06 * v);
            (
                centre
                    + (up * angle.cos() + side * angle.sin()) * out
                    + facing * (radius * 0.15 * 0.75 + 1.5),
                facing,
            )
        };
        plate(model.of(Stuff::Shade), &ring_grid(&inner, 1, 24), 1.2, true);
    }
    if boss {
        blob(
            model.of(Stuff::Main),
            centre + facing * (radius * 0.15 + 4.0),
            Vec3::splat(radius * 0.13),
        );
    }
}

/// A board standing on the shoulder, facing out and a little forwards, tipped back.
pub(super) fn add_ailette(
    model: &mut Model,
    form: &ShouldersForm,
    frame: &Frame,
    arm: &Arm,
    shape: AiletteShape,
    (wide, high): (f32, f32),
    charge: Charge,
) {
    let joint = frame.shoulder_half as f32;
    let base = Vec3::new(
        arm.sign * (joint - 12.0),
        shoulder_top(frame, joint - 12.0) + 18.0,
        -10.0,
    );
    let facing = Vec3::new(arm.sign * 0.8, 0.0, 0.6).normalize();
    let across = Vec3::Y.cross(facing).normalize();
    let tip = 15f32.to_radians();
    let up = (Vec3::Y * tip.cos() - facing * tip.sin()).normalize();
    let facing = across.cross(up).normalize() * across.cross(up).dot(facing).signum();
    let width = move |v: f32| match shape {
        AiletteShape::Rectangle => 1.0,
        AiletteShape::Rounded => (1.0 - ((v - 0.7).max(0.0) / 0.3).powi(2))
            .max(0.0)
            .sqrt()
            .max(0.3),
        AiletteShape::Shield => (v / 0.45).min(1.0).sqrt().max(0.06),
        AiletteShape::Lozenge => (1.0 - (2.0 * v - 1.0).abs()).max(0.06),
    };
    let at = move |u: f32, v: f32| base + across * (u * wide / 2.0 * width(v)) + up * (v * high);
    let board = |u: f32, v: f32| (at(u * 2.0 - 1.0, v), facing);
    let thick = match (form.construction, form.material) {
        (Construction::Leather, _) => 6.0,
        (_, Material::Wood) => 8.0,
        _ => 2.0,
    };
    plate(model.of(Stuff::Field), &grid(&board, 10, 8), thick, false);
    // The charge as bars on the front, kept inside the outline.
    let lift = thick / 2.0 + 0.5;
    let bar = |model: &mut Model, from: Vec2, to: Vec2, half: f32| {
        let along = to - from;
        let square = Vec2::new(-along.y, along.x).normalize() * half;
        let surface = |u: f32, v: f32| {
            let point = from + along * v + square * (u * 2.0 - 1.0);
            let v = point.y.clamp(0.0, 1.0);
            let limit = width(v) * 0.9;
            (at(point.x.clamp(-limit, limit), v) + facing * lift, facing)
        };
        plate(model.of(Stuff::Charge), &grid(&surface, 8, 2), 0.8, false);
    };
    let x = |sign: f32| sign * arm.sign;
    match charge {
        Charge::Plain => {}
        Charge::Cross => {
            bar(model, Vec2::new(0.0, 0.05), Vec2::new(0.0, 0.95), 0.12);
            bar(model, Vec2::new(-0.95, 0.6), Vec2::new(0.95, 0.6), 0.07);
        }
        Charge::Saltire => {
            bar(model, Vec2::new(-0.9, 0.95), Vec2::new(0.9, 0.05), 0.1);
            bar(model, Vec2::new(0.9, 0.95), Vec2::new(-0.9, 0.05), 0.1);
        }
        Charge::Pale => bar(model, Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 0.3),
        Charge::Fess => bar(model, Vec2::new(-1.0, 0.55), Vec2::new(1.0, 0.55), 0.1),
        Charge::Bend => bar(
            model,
            Vec2::new(x(1.0), 0.95),
            Vec2::new(x(-1.0), 0.05),
            0.12,
        ),
        Charge::Chevron => {
            bar(model, Vec2::new(-0.95, 0.15), Vec2::new(0.0, 0.7), 0.1);
            bar(model, Vec2::new(0.95, 0.15), Vec2::new(0.0, 0.7), 0.1);
        }
    }
}

/// One huge board of planks over the left shoulder, from the ear to the waist, wider at
/// the top, laced on two straps.
pub(super) fn add_wing(model: &mut Model, frame: &Frame) {
    let f = |value: i32| value as f32;
    let side = 1.0;
    let joint = f(frame.shoulder_half);
    let bend = joint * 1.2;
    let centre = joint - 60.0;
    let (bottom, top) = (f(frame.waist), f(frame.chin) + 30.0);
    let surface = |u: f32, v: f32| {
        let reach = 0.62 * (0.65 + 0.35 * (1.0 - v));
        let angle = (u * 2.0 - 1.0) * reach;
        let y = top - v * (top - bottom);
        let out = Vec3::new(side * angle.cos(), 0.0, angle.sin());
        (Vec3::new(side * centre, y, 0.0) + out * bend, out)
    };
    bands(model, Stuff::Main, &surface, 9, 2.5, 8.0);
    for strap in [0.3, 0.7] {
        patch(
            model,
            Stuff::Backing,
            &surface,
            (strap - 0.03, strap + 0.03),
            (0.0, 1.0),
            9.0 * 2.5 + 5.0,
            2.0,
        );
    }
}
