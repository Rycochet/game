//! Japanese leg pieces, on cloth and tied on: suneate on the shins (splints with mail
//! between, three hinged plates, hexagons in cloth, or mail) with a tall knee plate,
//! and haidate, an apron from the waist with armour over the thighs.

use std::f32::consts::PI;

use shared::armour::legs::{Knee, LegsForm};
use shared::armour::{Construction, Style};

use super::SOFT_LIFT;
use super::path::{FRONT, Leg};
use super::shapes::{Tube, band, flat, strips};
use crate::item_art::armour::surface::{bands, grid, lines, patch, studs};
use crate::item_art::armour::{Model, Stuff, plate};

/// Suneate or haidate on one leg, as the form's cover says.
pub(super) fn add_japanese(model: &mut Model, form: &LegsForm, leg: &Leg) {
    if form.cover.thigh() {
        add_haidate(model, form, leg);
    } else {
        add_suneate(model, form, leg);
    }
}

/// Shin guards: the cloth from below the knee to the ankle, what is on it, the tall
/// knee plate and two cords.
fn add_suneate(model: &mut Model, form: &LegsForm, leg: &Leg) {
    let (knee, ankle) = (leg.knee(), leg.ankle());
    let top = flat(knee - 40.0);
    let cloth = Tube {
        leg,
        top: &top,
        bottom: ankle,
        turn: (-0.55 * PI, 0.8 * PI),
        lift: SOFT_LIFT,
        style: Style::Japanese,
        swell: 0.0,
    };
    cloth.add(model, Stuff::Cloth, 3.0);
    let over = Tube {
        turn: (-0.4 * PI, 0.65 * PI),
        lift: SOFT_LIFT + 3.0,
        ..cloth
    };
    let surface = |u: f32, v: f32| over.at(u, v);
    match form.construction {
        // Tsutsu-suneate: three plates hinged together.
        Construction::Plate => strips(model, Stuff::Main, &over, 3, 0.3, 0.0),
        Construction::PlatesInCloth => {
            studs(model, Stuff::Main, &surface, (9, 6), 1.5, 6.0);
        }
        Construction::Mail => {
            patch(
                model,
                Stuff::Shade,
                &surface,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                1.5,
            );
            lines(model, &surface, 30, 1.0);
        }
        // Shino suneate: splints with mail between.
        _ => {
            patch(
                model,
                Stuff::Shade,
                &surface,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                1.2,
            );
            strips(model, Stuff::Main, &over, 6, 0.1, 1.5);
        }
    }
    if form.knee == Knee::Tall {
        let lift = SOFT_LIFT + 9.0;
        let knee_plate = |u: f32, v: f32| {
            let angle = FRONT + (u - 0.5) * 0.7 * PI;
            leg.at(knee + 110.0 - 150.0 * v, angle, lift + 6.0 * (PI * v).sin())
        };
        patch(
            model,
            Stuff::Main,
            &knee_plate,
            (0.0, 1.0),
            (0.0, 1.0),
            0.0,
            1.5,
        );
    }
    for height in [knee - 70.0, ankle + 70.0] {
        let cords = (-PI, PI);
        band(
            model,
            Stuff::Backing,
            leg,
            (height + 6.0, height),
            cords,
            SOFT_LIFT + 1.0,
            Style::White,
            1.5,
        );
    }
}

/// An apron over the front of the thigh, hung from the hip, cloth above and armour
/// below.
fn add_haidate(model: &mut Model, form: &LegsForm, leg: &Leg) {
    let hip = leg.frame.hip as f32;
    let bottom = if form.cover.knee() {
        leg.knee() - 10.0
    } else {
        leg.knee() + 40.0
    };
    let apron = |u: f32, v: f32| {
        let angle = -0.4 * PI + 1.05 * PI * u;
        let height = hip + 20.0 - (hip + 20.0 - bottom) * v;
        // Standing out from the belly at the top, close to the thigh below.
        leg.at(height, angle, 20.0 - 10.0 * v)
    };
    plate(model.of(Stuff::Cloth), &grid(&apron, 10, 10), 2.5, false);
    let lower = |u: f32, v: f32| {
        let (point, out) = apron(u, 0.4 + 0.6 * v);
        (point + out * 2.5, out)
    };
    match form.construction {
        Construction::PlatesInCloth => {
            studs(model, Stuff::Main, &lower, (5, 8), 1.0, 5.5);
        }
        Construction::Mail => {
            patch(
                model,
                Stuff::Shade,
                &lower,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                1.5,
            );
            lines(model, &lower, 18, 1.0);
        }
        Construction::Lamellar => {
            bands(model, Stuff::Main, &lower, 5, 0.8, 1.2);
        }
        // Small plates in rows, joined by mail.
        _ => {
            patch(
                model,
                Stuff::Shade,
                &lower,
                (0.0, 1.0),
                (0.0, 1.0),
                0.0,
                1.0,
            );
            for row in 0..4 {
                for column in 0..6 {
                    let (u, v) = ((column as f32 + 0.1) / 6.0, (row as f32 + 0.1) / 4.0);
                    let small = |a: f32, b: f32| lower(u + a * 0.8 / 6.0, v + b * 0.7 / 4.0);
                    patch(model, Stuff::Main, &small, (0.0, 1.0), (0.0, 1.0), 1.5, 1.4);
                }
            }
        }
    }
    // The cord at the top, round to the back.
    let cord = |u: f32, v: f32| {
        let angle = -0.4 * PI + 1.4 * PI * u;
        leg.at(hip + 20.0 - 8.0 * v, angle, 22.0)
    };
    plate(model.of(Stuff::Backing), &grid(&cord, 1, 12), 2.0, false);
}
