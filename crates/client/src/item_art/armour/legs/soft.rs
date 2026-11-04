//! Leg pieces of many parts on one leg: chausses of mail (with feet, and with plates
//! strapped over them), ring mail, scales, lamellae, laminar hoops, brigandine,
//! padding, splints on leather, and plates joined by mail (the dizçek).

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::legs::{Cover, Knee, LegsForm, Reinforce};
use shared::armour::{Construction, Style};
use shared::body::Joint;

use super::path::{FRONT, Leg};
use super::rigid::add_knee;
use super::shapes::{Tube, flat, strips, turn};
use super::{RIGID_LIFT, SOFT_LIFT};
use crate::item_art::armour::surface::{bands, lines, patch, scales, studs};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::blob;

/// From and to where a soft piece runs: its cover's span, the thigh's top following
/// the crotch's slope as a cuisse does.
fn top_of(form: &LegsForm, leg: &Leg) -> impl Fn(f32) -> f32 + use<> {
    let (top, _) = form.cover.span(leg.frame);
    let (top, thigh) = (top as f32, form.cover.thigh());
    move |angle: f32| {
        if thigh {
            top + 50.0 * angle.sin().max(0.0) - 40.0 * (-angle.cos()).max(0.0)
        } else {
            top
        }
    }
}

/// The piece's surface on this leg.
fn tube<'a>(form: &LegsForm, leg: &'a Leg, top: &'a dyn Fn(f32) -> f32, lift: f32) -> Tube<'a> {
    let (_, bottom) = form.cover.span(leg.frame);
    Tube {
        leg,
        top,
        bottom: bottom as f32,
        turn: turn(form.closed),
        lift,
        style: Style::White,
        swell: 0.0,
    }
}

/// Pieces that hang rather than hold their shape.
pub(super) fn add_soft(model: &mut Model, form: &LegsForm, leg: &Leg) {
    let top = top_of(form, leg);
    let lift = match form.construction {
        Construction::Padded => 6.0,
        Construction::Laminar => SOFT_LIFT + 4.0,
        _ => SOFT_LIFT,
    };
    let tube = tube(form, leg, &top, lift);
    let surface = |u: f32, v: f32| tube.at(u, v);
    // Scales, lamellae and hoops sit over their backing, not in it.
    let over = |u: f32, v: f32| {
        let (point, out) = tube.at(u, v);
        (point + out * 2.0, out)
    };
    let length = top(FRONT) - tube.bottom;
    match form.construction {
        Construction::Mail => {
            tube.add(model, Stuff::Main, 2.0);
            lines(model, &surface, (length / 9.0) as usize, 1.2);
            if form.footed() {
                add_mail_foot(model, leg);
            }
        }
        Construction::RingMail => {
            tube.add(model, Stuff::Backing, 2.5);
            let rows = (length / 14.0) as usize;
            studs(model, Stuff::Main, &surface, (rows, 20), 2.0, 3.0);
        }
        Construction::Scale => {
            tube.add(model, Stuff::Backing, 2.0);
            scales(model, Stuff::Main, &over, (length / 24.0) as usize, 16);
        }
        Construction::Lamellar => {
            tube.add(model, Stuff::Cloth, 2.0);
            bands(
                model,
                Stuff::Main,
                &over,
                (length / 32.0) as usize,
                0.8,
                1.2,
            );
        }
        Construction::Laminar => {
            // Leather straps inside hold the hoops.
            Tube {
                turn: (0.75 * PI, 1.25 * PI),
                lift: SOFT_LIFT,
                ..tube
            }
            .add(model, Stuff::Backing, 2.0);
            bands(
                model,
                Stuff::Main,
                &over,
                (length / 28.0) as usize,
                1.1,
                1.2,
            );
        }
        Construction::PlatesInCloth => {
            tube.add(model, Stuff::Cloth, 4.0);
            let rows = (length / 40.0) as usize;
            studs(model, Stuff::Rivet, &surface, (rows, 12), 2.5, 2.5);
        }
        Construction::MailAndPlate => add_plated(model, form, &tube),
        Construction::Splint => add_splints(model, form, leg, &tube),
        _ => {
            // Padding, quilted in channels down the leg.
            tube.add(model, Stuff::Main, 6.0);
            strips(model, Stuff::Shade, &tube, 18, 0.012, 3.2);
        }
    }
    if form.knee != Knee::None {
        // Over the thick padding of gamboised cuisses, which had poleyns strapped on.
        let (over, stuff) = if form.construction == Construction::Padded {
            (10.0, Stuff::Rivet)
        } else {
            (6.0, Stuff::Main)
        };
        add_knee(model, form, leg, lift + over, stuff);
    }
    if form.reinforce != Reinforce::None {
        add_reinforcements(model, form, leg);
    }
}

/// Columns of small plates joined by mail, as on the dizçek, over a mail backing.
fn add_plated(model: &mut Model, _form: &LegsForm, tube: &Tube) {
    tube.add(model, Stuff::Shade, 2.0);
    let rows = 5;
    let columns = 8;
    for row in 0..rows {
        for column in 0..columns {
            let u = (column as f32 + 0.1) / columns as f32;
            let v = (row as f32 + 0.1) / rows as f32;
            let plate =
                |a: f32, b: f32| tube.at(u + a * 0.8 / columns as f32, v + b * 0.75 / rows as f32);
            patch(model, Stuff::Main, &plate, (0.0, 1.0), (0.0, 1.0), 2.5, 1.4);
        }
    }
}

/// Splints down the leg on a leather backing.
fn add_splints(model: &mut Model, form: &LegsForm, leg: &Leg, tube: &Tube) {
    let backing = Tube {
        lift: RIGID_LIFT - 6.0,
        ..*tube
    };
    backing.add(model, Stuff::Backing, 3.0);
    let count = if form.closed { 9 } else { 6 };
    let width = if form.closed { 0.07 } else { 0.1 };
    if form.cover.thigh() && form.cover.shin() {
        // A thigh's splints and a shin's, apart at the knee.
        let knee = leg.knee();
        let upper = Tube {
            bottom: knee + 75.0,
            ..backing
        };
        let lower = Tube {
            top: &flat(knee - 60.0),
            ..backing
        };
        strips(model, Stuff::Main, &upper, count, width, 3.5);
        strips(model, Stuff::Main, &lower, count, width, 3.5);
    } else {
        strips(model, Stuff::Main, &backing, count, width, 3.5);
    }
}

/// A mail foot over the shoe, from the ankle down: the bare foot a little bigger.
fn add_mail_foot(model: &mut Model, leg: &Leg) {
    let frame = leg.frame;
    let point = |joint| {
        let [x, y, z] = frame.joint(joint);
        Vec3::new(x as f32, y as f32, z as f32)
    };
    let (toes, heel) = (point(Joint::Toes(leg.side)), point(Joint::Heel(leg.side)));
    let ankle = frame.ankle as f32;
    let lift = SOFT_LIFT + 4.0;
    blob(
        model.of(Stuff::Main),
        // Its sole just off the ground.
        Vec3::new(
            toes.x,
            ankle / 2.0 + lift / 2.0 + 1.0,
            (toes.z + heel.z) / 2.0,
        ),
        Vec3::new(
            frame.ankle_radius as f32 * 1.3 + lift,
            ankle / 2.0 + lift / 2.0,
            (toes.z - heel.z) / 2.0 + lift,
        ),
    );
}

/// Plates strapped over mail chausses: poleyns, schynbalds, quilted thighs.
fn add_reinforcements(model: &mut Model, form: &LegsForm, leg: &Leg) {
    let lift = SOFT_LIFT + 8.0;
    let poleyns = LegsForm {
        construction: Construction::Plate,
        knee: Knee::Cop,
        ..*form
    };
    if form.reinforce == Reinforce::QuiltedThighs {
        let top = top_of(
            &LegsForm {
                cover: Cover::Thigh,
                ..*form
            },
            leg,
        );
        let thigh = Tube {
            leg,
            top: &top,
            bottom: leg.knee() + 60.0,
            turn: (-PI, PI),
            lift,
            style: Style::White,
            swell: 0.0,
        };
        thigh.add(model, Stuff::Cloth, 6.0);
        strips(model, Stuff::Cloth, &thigh, 14, 0.014, 3.0);
    }
    add_knee(model, &poleyns, leg, lift + 4.0, Stuff::Main);
    if form.reinforce == Reinforce::PoleynsAndSchynbalds {
        let schynbald = Tube {
            leg,
            top: &flat(leg.knee() - 55.0),
            bottom: leg.ankle() + 20.0,
            turn: turn(false),
            lift: lift + 2.0,
            style: form.style,
            swell: 0.0,
        };
        schynbald.add(model, Stuff::Main, 1.3);
    }
}
