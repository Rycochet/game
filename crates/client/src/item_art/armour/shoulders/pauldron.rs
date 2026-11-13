//! Pauldrons: the cop and lames, besagews and discs.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::shoulders::{Finish, ShouldersForm};
use shared::body::Frame;
use std::f32::consts::{PI, TAU};

use super::super::surface::{bands, patch, ring_grid, studs};
use super::super::{Model, Stuff, plate};
use super::{ACROSS, Arm, Look, RIGID_CLEAR, WING_CLEAR, arm_surface, cop_surface, sheet};
use crate::item_art::body::{blob, trunk_depth};

/// One pauldron or spaulder: cop, lames, and what goes with them.
#[allow(clippy::too_many_arguments)]
pub(super) fn add_cop_and_lames(
    model: &mut Model,
    form: &ShouldersForm,
    frame: &Frame,
    arm: &Arm,
    look: Look,
    (start, end): (f32, f32),
    lames: u8,
    besagews: bool,
) {
    let across = arm.radius + RIGID_CLEAR;
    // Broader than it is high, as a cop is.
    let up = across * 0.72;
    let stuff = Stuff::Main;
    let laminar = form.construction == Construction::Laminar;
    let cop = cop_surface(arm, look, (start, end), (across, up), PI / 2.0);
    if laminar {
        // Roman: the cop itself in lames, each lower one further out.
        let count = (lames as usize + 2).max(4);
        bands(model, stuff, &cop, count, -look.step, look.thick);
    } else {
        sheet(model, stuff, &cop, look.thick);
    }
    if look.runes {
        patch(
            model,
            Stuff::Etch,
            &cop,
            (0.1, 0.9),
            (0.82, 0.92),
            look.thick / 2.0 + 0.4,
            0.6,
        );
    }
    if form.finish == Finish::Studded {
        studs(model, Stuff::Rivet, &cop, (3, 7), look.thick / 2.0, 3.5);
    }
    if lames == 0 {
        return;
    }
    let lames = lames as usize;
    let length = arm.reach / lames as f32;
    for lame in 0..lames {
        let top = lame as f32 * length - 8.0;
        let bottom = (lame + 1) as f32 * length;
        // Each lame over the one below.
        // Flaring out down the arm.
        let radius = across + lame as f32 * 5.0 + (lames - lame) as f32 * look.step;
        let band = arm_surface(arm, look, (start, end), (top, bottom), radius);
        sheet(model, stuff, &band, look.thick);
        if look.etched {
            patch(
                model,
                Stuff::Etch,
                &band,
                (0.0, 1.0),
                (0.62, 0.86),
                look.thick / 2.0 + 0.3,
                0.6,
            );
        }
        if form.finish == Finish::Studded {
            studs(model, Stuff::Rivet, &band, (1, 9), look.thick / 2.0, 3.0);
        } else if form.construction == Construction::Plate {
            let rivet = if look.runes { 4.5 } else { 2.6 };
            studs(model, Stuff::Rivet, &band, (1, 2), look.thick / 2.0, rivet);
        }
        if lame == lames - 1 && look.rim > 0.0 {
            patch(
                model,
                stuff,
                &band,
                (0.0, 1.0),
                (0.86, 1.0),
                look.rim,
                look.thick,
            );
        }
    }
    if besagews {
        add_besagew(model, frame, arm, look);
    }
}

/// A disc hung in front of the armpit.
pub(super) fn add_besagew(model: &mut Model, frame: &Frame, arm: &Arm, look: Look) {
    let x = arm.joint.x.abs() - arm.radius * 0.8;
    let y = arm.joint.y - arm.reach * 0.55;
    let z = trunk_depth(frame, x, y, true) + WING_CLEAR + 8.0;
    let centre = Vec3::new(arm.sign * x, y, z);
    let facing = Vec3::new(arm.sign * 0.3, 0.1, 1.0).normalize();
    add_disc(
        model,
        Stuff::Main,
        centre,
        facing,
        arm.radius * 1.15,
        8.0,
        look.thick,
    );
    blob(
        model.of(Stuff::Main),
        centre + facing * 14.0,
        Vec3::new(5.0, 5.0, 5.0),
    );
}

/// A shallow dome facing `facing`.
pub(super) fn add_disc(
    model: &mut Model,
    stuff: Stuff,
    centre: Vec3,
    facing: Vec3,
    radius: f32,
    dome: f32,
    thick: f32,
) {
    let up = (Vec3::Y - facing * Vec3::Y.dot(facing)).normalize_or(Vec3::Z);
    let side = facing.cross(up);
    let surface = |u: f32, v: f32| {
        let angle = u * TAU;
        let out = 0.04 + 0.96 * v;
        let point = centre
            + (up * angle.cos() + side * angle.sin()) * radius * out
            + facing * dome * (1.0 - out * out);
        (point, facing)
    };
    plate(
        model.of(stuff),
        &ring_grid(&surface, 4, ACROSS + 4),
        thick,
        true,
    );
}
