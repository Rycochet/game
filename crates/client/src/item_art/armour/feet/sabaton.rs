//! Sabatons: a toe cap shaped to the toe, lames across the top of the foot stepping
//! back to the ankle, each over the one in front, an ankle plate, sometimes a heel cap
//! or a cuff over the ankle (armoured boots), and a strap under the shoe. Scales and
//! splints are laid over a leather sabaton of the same shape.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::feet::{FeetForm, Toe};

use super::last::{Foot, Last, OVER_TOP};
use super::parts::{ankle_band, foot_band, sheet};
use crate::item_art::armour::legs::Leg;
use crate::item_art::armour::surface::{bands, patch, scales, studs};
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::{blob, limb};

/// Where the lames start, in front of the ankle, and how far the toe cap reaches back
/// from the toes' tips.
const LAMES_FROM: f32 = 8.0;
const CAP_BACK: f32 = 40.0;

pub(super) fn add_sabaton(model: &mut Model, form: &FeetForm, last: &Last, foot: &Foot) {
    let cap = foot.toes() - CAP_BACK;
    let (tip, rigid) = (last.tip(), form.rigid());
    let thick = if form.construction == Construction::Leather {
        2.5
    } else {
        1.4
    };
    let top = last.patch((LAMES_FROM, tip), OVER_TOP);
    match form.construction {
        Construction::Scale => {
            sheet(model, Stuff::Backing, &top, 2.0);
            scales(model, Stuff::Main, &top, 7, 6);
        }
        Construction::Splint => {
            sheet(model, Stuff::Backing, &top, 2.0);
            add_splints(model, form, &top);
        }
        _ => {
            sheet(model, Stuff::Main, &last.patch((cap, tip), OVER_TOP), thick);
            let lames = last.patch((LAMES_FROM, cap + 10.0), OVER_TOP);
            bands(model, Stuff::Main, &lames, form.lames as usize, 0.9, thick);
            if rigid {
                add_lame_rivets(model, form, last, cap);
            }
            if form.fluted {
                add_flutes(model, &lames, form.lames as f32);
            }
            if form.studded {
                studs(model, Stuff::Rivet, &top, (3, 4), thick + 1.0, 2.0);
            }
        }
    }
    let leg = Leg {
        frame: foot.frame,
        side: foot.side,
    };
    let ankle = foot.frame.ankle as f32;
    let radius = foot.frame.ankle_radius as f32 + last.lift;
    if form.high {
        ankle_band(
            model,
            Stuff::Main,
            &leg,
            (55.0, ankle + 60.0),
            radius,
            4.0,
            thick,
        );
    } else {
        ankle_band(
            model,
            Stuff::Main,
            &leg,
            (62.0, ankle + 20.0),
            radius,
            1.9,
            thick,
        );
    }
    if form.heel {
        let heel = Last {
            lift: last.lift + 1.5,
            ..*last
        };
        sheet(
            model,
            Stuff::Main,
            &heel.patch((heel.back(), -5.0), 2.4),
            thick,
        );
    }
    foot_band(model, Stuff::Backing, last, 35.0, 12.0);
    if form.trim.is_some() {
        patch(
            model,
            Stuff::Trim,
            &top,
            (0.0, 0.03),
            (0.0, 1.0),
            thick,
            1.0,
        );
        patch(
            model,
            Stuff::Trim,
            &top,
            (0.97, 1.0),
            (0.0, 1.0),
            thick,
            1.0,
        );
    }
    if form.detachable {
        let catch = last.at(foot.toes() + last.lift, 0.0);
        blob(
            model.of(Stuff::Rivet),
            catch + Vec3::Y * 2.0,
            Vec3::splat(4.5),
        );
    }
    if form.toe == Toe::Claw {
        add_claws(model, last, tip);
    }
}

/// A rivet at each end of each lame.
fn add_lame_rivets(model: &mut Model, form: &FeetForm, last: &Last, cap: f32) {
    let count = form.lames.max(1) as f32;
    for lame in 0..form.lames {
        let z = LAMES_FROM + (cap + 10.0 - LAMES_FROM) * (lame as f32 + 0.5) / count;
        for end in [-1.0, 1.0] {
            let (point, out) = last.point_out(z, end * OVER_TOP * 0.9);
            let lift = (count - lame as f32) * 0.9 + 2.0;
            blob(model.of(Stuff::Rivet), point + out * lift, Vec3::splat(2.4));
        }
    }
}

/// Parallel ridges along the lames, Maximilian's flutes.
fn add_flutes(model: &mut Model, lames: &dyn Fn(f32, f32) -> (Vec3, Vec3), count: f32) {
    for flute in 0..5 {
        let u = 0.22 + flute as f32 * 0.14;
        patch(
            model,
            Stuff::Main,
            lames,
            (u - 0.025, u + 0.025),
            (0.0, 1.0),
            count * 0.9 + 1.4,
            1.2,
        );
    }
}

/// Splints along the foot, riveted at both ends.
fn add_splints(model: &mut Model, form: &FeetForm, top: &dyn Fn(f32, f32) -> (Vec3, Vec3)) {
    let count = form.lames.max(1) as usize;
    for splint in 0..count {
        let u = (splint as f32 + 0.5) / count as f32;
        let half = 0.32 / count as f32;
        patch(
            model,
            Stuff::Main,
            top,
            (u - half, u + half),
            (0.0, 0.9),
            2.5,
            1.5,
        );
        for v in [0.06, 0.84] {
            let (point, out) = top(u, v);
            blob(model.of(Stuff::Rivet), point + out * 4.5, Vec3::splat(2.2));
        }
    }
}

/// Three claws curving forward and down from the front of the toe.
fn add_claws(model: &mut Model, last: &Last, tip: f32) {
    for round in [-0.55, 0.0, 0.55] {
        let (base, out) = last.point_out(tip - 8.0, round);
        let ahead = base + Vec3::Z * 30.0 + out * 6.0;
        let end = Vec3::new(ahead.x, (ahead.y - 18.0).max(2.0), ahead.z + 18.0);
        limb(
            model.of(Stuff::Main),
            &[(base, 10.0), (ahead, 6.5), (end, 1.0)],
        );
    }
}
