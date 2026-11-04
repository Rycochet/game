//! Bevors: a cup standing clear of the chin and throat, from the mouth (or the chin)
//! round to the ears and down onto the top of the breast, maybe with lames below it,
//! strapped round the back of the neck.

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::neck::NeckForm;

use super::around::{Around, Profile, dir, sideness};
use crate::item_art::armour::surface::{bands, grid, patch, studs};
use crate::item_art::armour::{Model, Stuff, plate};

/// How far round it reaches either side of the front: to the ears.
const REACH: f32 = 1.75;
/// Out over a breastplate.
const BREAST_OFF: f32 = 40.0;
const THICK: f32 = 2.0;
const LAME: f32 = 20.0;

pub(super) fn add_bevor(model: &mut Model, form: &NeckForm, around: &Around) {
    let (chin, nb, n) = (around.chin(), around.neck_base(), around.neck());
    let profile = |angle: f32| {
        let s = sideness(angle);
        let front_top = chin + if form.high { 55.0 } else { 15.0 };
        let top_y = front_top + (chin + 30.0 - front_top) * s;
        let top = around.head(angle, top_y, 45.0 - 20.0 * s);
        let middle = dir(angle) * (n + 40.0 - 12.0 * s) + Vec3::Y * (chin - 10.0);
        let point = if form.pointed {
            35.0 * (1.0 - angle.abs() / 0.9).max(0.0)
        } else {
            0.0
        };
        let front = around.trunk(angle, nb - 60.0 - point, BREAST_OFF);
        let side = around.over_shoulder(angle, around.base_across() + 30.0, 14.0);
        Profile {
            top,
            middle,
            bottom: front.lerp(side, s),
        }
    };
    let point = |u: f32, v: f32| profile(-REACH + 2.0 * REACH * u).at(v);
    let surface = |u: f32, v: f32| {
        let angle = -REACH + 2.0 * REACH * u;
        (point(u, v), (dir(angle) + Vec3::Y * (0.5 - v)).normalize())
    };
    plate(model.of(Stuff::Main), &grid(&surface, 12, 20), THICK, false);
    let edge = if form.trim.is_some() {
        Stuff::Trim
    } else {
        Stuff::Main
    };
    patch(model, edge, &surface, (0.0, 1.0), (0.0, 0.05), 1.6, 2.5);
    if form.pointed {
        // A ridge down the middle.
        patch(
            model,
            Stuff::Main,
            &surface,
            (0.49, 0.51),
            (0.1, 1.0),
            2.5,
            3.0,
        );
    }
    // Pivots at the ears.
    let pivots = |u: f32, v: f32| surface(u * 0.98 + 0.01, v * 0.1 + 0.05);
    studs(model, Stuff::Rivet, &pivots, (1, 2), 2.0, 4.0);
    if form.lames > 0 {
        // Hanging from the cup's lower edge, over the front of the breast.
        let lames = |u: f32, v: f32| {
            let angle = (u - 0.5) * 1.5;
            let bottom = profile(angle).bottom.y;
            let y = bottom + 8.0 - LAME * form.lames as f32 * v;
            (around.trunk(angle, y, BREAST_OFF + 2.0), dir(angle))
        };
        bands(model, Stuff::Main, &lames, form.lames as usize, 1.2, THICK);
    }
    // The strap round the back of the neck.
    let strap = |u: f32, v: f32| {
        let angle = REACH + (2.0 * PI - 2.0 * REACH) * u;
        let radius = n + 6.0;
        let at = Vec3::new(
            radius * angle.sin(),
            nb + 40.0 - 18.0 * v,
            radius * angle.cos(),
        );
        (at, dir(angle))
    };
    plate(model.of(Stuff::Backing), &grid(&strap, 1, 10), 2.0, false);
}
