//! Arm pieces of many parts on one arm: sleeves of mail, rings, scales, lamellae,
//! padding and brigandine, splints on leather, laminar hoops (the manica) and kote.

use std::f32::consts::PI;

use shared::armour::arms::{ArmsForm, Couter, Reach};
use shared::armour::{Construction, Style};

use super::path::{Arm, BACK, OUTSIDE};
use super::rigid::add_couter;
use super::shapes::{Tube, flat, ring, strips};
use super::{RIGID_LIFT, SOFT_LIFT};
use crate::item_art::armour::surface::{bands, lines, patch, scales, studs};
use crate::item_art::armour::{Model, Stuff};

/// Where a soft piece runs down the arm: a sleeve from the shoulder's seam, or as
/// far as its reach.
fn span(form: &ArmsForm, arm: &Arm) -> (f32, f32) {
    let (top, bottom) = form.reach.span(arm.frame);
    let top = if form.reach == Reach::Whole { 20 } else { top };
    (top as f32, bottom as f32)
}

/// Sleeves and forearm pieces that hang rather than hold their shape.
pub(super) fn add_sleeve(model: &mut Model, form: &ArmsForm, arm: &Arm) {
    let (top, bottom) = span(form, arm);
    let edge = flat(top);
    let tube = Tube {
        arm,
        top: &edge,
        bottom,
        turn: (-PI, PI),
        lift: SOFT_LIFT,
        style: Style::White,
    };
    let surface = |u: f32, v: f32| tube.at(u, v);
    let length = bottom - top;
    match form.construction {
        Construction::Mail => {
            tube.add(model, Stuff::Main, 2.0);
            lines(model, &surface, (length / 9.0) as usize, 1.2);
            if form.trim.is_some() {
                ring(
                    model,
                    Stuff::Trim,
                    arm,
                    (bottom - 8.0, bottom),
                    SOFT_LIFT + 1.0,
                    Style::White,
                    1.5,
                );
            }
        }
        Construction::RingMail => {
            tube.add(model, Stuff::Backing, 2.5);
            studs(
                model,
                Stuff::Main,
                &surface,
                ((length / 14.0) as usize, 18),
                2.0,
                3.0,
            );
        }
        Construction::Scale => {
            tube.add(model, Stuff::Backing, 2.0);
            scales(model, Stuff::Main, &surface, (length / 22.0) as usize, 14);
        }
        Construction::Lamellar => {
            tube.add(model, Stuff::Cloth, 2.0);
            bands(
                model,
                Stuff::Main,
                &surface,
                (length / 30.0) as usize,
                0.8,
                1.2,
            );
        }
        Construction::PlatesInCloth => {
            tube.add(model, Stuff::Cloth, 4.0);
            studs(
                model,
                Stuff::Rivet,
                &surface,
                ((length / 40.0) as usize, 10),
                2.5,
                2.5,
            );
        }
        _ => {
            // Padding: quilted every 30 mm.
            tube.add(model, Stuff::Main, 6.0);
            lines(model, &surface, (length / 30.0) as usize, 3.2);
        }
    }
    if form.construction == Construction::MailAndPlate {
        // Indo-Persian plated sleeves: plates down the outside of the forearm.
        let elbow = arm.elbow();
        let plates = Tube {
            arm,
            top: &flat(elbow + 40.0),
            bottom,
            turn: (BACK - 0.4 * PI, BACK + 0.9 * PI),
            lift: SOFT_LIFT + 3.0,
            style: form.style,
        };
        plates.add(model, Stuff::Main, 1.4);
    }
}

/// Splints down the arm on a leather backing, with a cup on the elbow.
pub(super) fn add_splints(model: &mut Model, form: &ArmsForm, arm: &Arm) {
    let (top, bottom) = span(form, arm);
    let elbow = arm.elbow();
    let mut stretches = Vec::new();
    if form.reach.upper() {
        stretches.push((top, elbow - 50.0));
    }
    if form.reach.forearm() {
        let start = if form.reach == Reach::Forearm {
            top
        } else {
            elbow + 45.0
        };
        stretches.push((start, bottom));
    }
    for (from, to) in stretches {
        let backing = Tube {
            arm,
            top: &flat(from),
            bottom: to,
            turn: (-0.8 * PI, 0.8 * PI),
            lift: RIGID_LIFT - 6.0,
            style: Style::White,
        };
        backing.add(model, Stuff::Backing, 3.0);
        let splints = Tube {
            turn: (-0.55 * PI, 0.55 * PI),
            ..backing
        };
        strips(model, Stuff::Main, &splints, 5, 0.1, 3.5);
    }
    if form.couter != Couter::None {
        add_couter(model, form, arm);
    }
}

/// Laminar hoops overlapping downwards round the outer three quarters of the arm: the
/// manica, or Steppe laminar vambraces.
pub(super) fn add_hoops(model: &mut Model, form: &ArmsForm, arm: &Arm) {
    let (top, bottom) = span(form, arm);
    let edge = flat(top);
    let tube = Tube {
        arm,
        top: &edge,
        bottom,
        turn: (-0.78 * PI, 0.78 * PI),
        lift: SOFT_LIFT + 4.0,
        style: form.style,
    };
    let surface = |u: f32, v: f32| tube.at(u, v);
    // Three leather straps inside hold them.
    let inside = Tube {
        turn: (0.7 * PI, 1.3 * PI),
        lift: SOFT_LIFT,
        ..tube
    };
    inside.add(model, Stuff::Backing, 2.0);
    bands(
        model,
        Stuff::Main,
        &surface,
        ((bottom - top) / 28.0) as usize,
        1.1,
        1.2,
    );
}

/// Kote: a padded cloth sleeve laced inside, with a gourd-shaped plate on the upper
/// arm, a round plate on the elbow and plates on the forearm, mail showing between.
/// Han kote are the forearm part alone.
pub(super) fn add_kote(model: &mut Model, form: &ArmsForm, arm: &Arm) {
    let (top, bottom) = span(form, arm);
    let elbow = arm.elbow();
    let edge = flat(top);
    let sleeve = Tube {
        arm,
        top: &edge,
        bottom,
        turn: (-PI, PI),
        lift: SOFT_LIFT,
        style: Style::Japanese,
    };
    sleeve.add(model, Stuff::Cloth, 4.0);
    let surface = |u: f32, v: f32| sleeve.at(u, v);
    // Mail over the outer half, under the plates.
    let mail = |u: f32, v: f32| surface(0.25 + u * 0.5, v);
    patch(model, Stuff::Shade, &mail, (0.0, 1.0), (0.0, 1.0), 1.5, 1.0);
    let lift = SOFT_LIFT + 4.0;
    if form.reach == Reach::Whole {
        let gourd = |u: f32, v: f32| {
            let angle = OUTSIDE + (u - 0.5) * 0.9 * PI * (0.6 + 0.4 * (PI * v).sin());
            let down = 110.0 + (elbow - 180.0) * v;
            arm.at(down, angle, lift + 3.0 * (PI * v).sin())
        };
        patch(model, Stuff::Main, &gourd, (0.0, 1.0), (0.0, 1.0), 0.0, 1.5);
        let disc = |u: f32, v: f32| {
            arm.at(
                elbow - 30.0 + 60.0 * v,
                BACK + (u - 0.5) * 0.7 * PI,
                lift + 4.0,
            )
        };
        patch(model, Stuff::Main, &disc, (0.0, 1.0), (0.0, 1.0), 0.0, 1.5);
    }
    let forearm = Tube {
        arm,
        top: &flat(top.max(elbow + 40.0)),
        bottom: bottom - 10.0,
        turn: (-0.5 * PI, 0.5 * PI),
        lift,
        style: Style::Japanese,
    };
    if form.construction == Construction::MailAndPlate {
        // Shino gote: splints over mail.
        strips(model, Stuff::Main, &forearm, 4, 0.12, 0.0);
    } else {
        // Tsutsu gote: one hinged plate.
        forearm.add(model, Stuff::Main, 1.5);
    }
}
