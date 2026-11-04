//! Foot guards over the instep, and often the toes, tied on: kogake (plates on a cloth
//! tabi, or plates joined by mail), leather guards, and the Greek bronze feet with
//! their toes.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::feet::FeetForm;

use super::last::{Foot, Last};
use super::parts::{ankle_band, foot_band, sheet};
use crate::item_art::armour::legs::Leg;
use crate::item_art::armour::surface::{lines, patch, studs};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::blob;

/// How far round the sides a guard reaches.
const ROUND: f32 = 1.3;

pub(super) fn add_guard(model: &mut Model, form: &FeetForm, last: &Last, foot: &Foot) {
    let end = if form.toes {
        last.tip() - 4.0
    } else {
        foot.toes() - 55.0
    };
    if form.bare_toes() {
        sheet(
            model,
            Stuff::Main,
            &last.patch((5.0, foot.toes() - 20.0), ROUND),
            1.4,
        );
        add_toes(model, last, foot);
        return;
    }
    if form.japanese() {
        add_kogake(model, form, last, foot, end);
        return;
    }
    let guard = last.patch((15.0, end), ROUND);
    sheet(model, Stuff::Main, &guard, 2.5);
    if form.studded {
        studs(model, Stuff::Rivet, &guard, (3, 3), 3.0, 2.0);
    }
    foot_band(model, Stuff::Backing, last, 55.0, 10.0);
}

/// A cloth tabi under plates, tied round the ankle and the foot.
fn add_kogake(model: &mut Model, form: &FeetForm, last: &Last, foot: &Foot, end: f32) {
    let tabi = Last {
        lift: last.lift - 3.0,
        ..*last
    };
    sheet(
        model,
        Stuff::Cloth,
        &tabi.patch((tabi.back() + 25.0, tabi.tip()), 1.9),
        1.5,
    );
    let top = last.patch((30.0, end), ROUND * 0.85);
    match form.construction {
        Construction::MailAndPlate => {
            // An instep plate and a toe plate, mail between them.
            let toe_from = if form.toes { 0.72 } else { 1.0 };
            patch(
                model,
                Stuff::Main,
                &top,
                (0.15, 0.85),
                (0.0, 0.58),
                2.0,
                1.6,
            );
            patch(
                model,
                Stuff::Shade,
                &top,
                (0.1, 0.9),
                (0.58, toe_from),
                1.2,
                1.2,
            );
            lines(model, &top, 8, 1.6);
            if form.toes {
                patch(
                    model,
                    Stuff::Main,
                    &top,
                    (0.2, 0.8),
                    (toe_from, 1.0),
                    2.0,
                    1.6,
                );
            }
        }
        Construction::Plate => patch(model, Stuff::Main, &top, (0.1, 0.9), (0.0, 1.0), 2.0, 1.6),
        // Small plates in rows, sewn on.
        _ => {
            for row in 0..4 {
                for column in 0..2 {
                    let (u, v) = (0.22 + column as f32 * 0.3, row as f32 * 0.25);
                    patch(
                        model,
                        Stuff::Main,
                        &top,
                        (u, u + 0.26),
                        (v, v + 0.22),
                        2.0,
                        1.6,
                    );
                }
            }
        }
    }
    let leg = Leg {
        frame: foot.frame,
        side: foot.side,
    };
    let ankle = foot.frame.ankle as f32;
    let radius = foot.frame.ankle_radius as f32 + tabi.lift + 1.0;
    ankle_band(
        model,
        Stuff::Cloth,
        &leg,
        (ankle + 5.0, ankle + 12.0),
        radius,
        4.0,
        1.2,
    );
    foot_band(model, Stuff::Cloth, last, 90.0, 5.0);
}

/// Five toes at the front of a bronze foot, the big one on the inside.
fn add_toes(model: &mut Model, last: &Last, foot: &Foot) {
    let backs = [0.0, 6.0, 11.0, 17.0, 24.0];
    let sizes = [11.0, 8.5, 8.0, 7.0, 6.0];
    for toe in 0..5 {
        let round = -0.85 + toe as f32 * 0.42;
        let z = foot.toes() + last.lift - 14.0 - backs[toe];
        let (point, out) = last.point_out(z, round);
        let size = sizes[toe];
        blob(
            model.of(Stuff::Main),
            point + out * size * 0.55,
            Vec3::new(size, size * 0.8, size * 1.5),
        );
    }
}
