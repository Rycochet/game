//! How a closed shoe or boot is held on: laces and their facings, buckled straps,
//! buttons, toggles, elastic sides, thongs round the shin and drawstrings.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use shared::footwear::{Fastening, Height};

use super::shoe::Shoe;
use super::straps::band;
use super::upper::{add_shaft, on_shaft, sheet};
use crate::item_art::armour::surface::grid;
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::{blob, limb};

/// Round the leg towards the outside, where buttons and buckles go.
const OUTSIDE: f32 = FRAC_PI_2 * 0.8;

pub(super) fn add_fastening(model: &mut Model, shoe: &Shoe) {
    let form = shoe.form;
    let ankle = shoe.foot.frame.ankle as f32;
    match form.fastening {
        Fastening::ClosedLaces | Fastening::OpenLaces | Fastening::Loops => {
            add_laces(model, shoe);
            if form.fastening == Fastening::Loops {
                add_shaft(
                    model,
                    shoe,
                    Stuff::Shade,
                    (ankle - 3.0, ankle + 3.0),
                    1.0,
                    1.5,
                );
            }
        }
        Fastening::Buckle | Fastening::Strap => add_strap(model, shoe),
        Fastening::Buttons if form.boot() => {
            let mut y = ankle * 0.8;
            while y < shoe.top() - 8.0 {
                let (point, out) = on_shaft(shoe, y, OUTSIDE, 0.5);
                blob(model.of(Stuff::Rivet), point + out * 2.0, Vec3::splat(3.0));
                y += 15.0;
            }
        }
        Fastening::Toggles => {
            let last = shoe.last(shoe.lift() + shoe.thick());
            for z in [shoe.throat() - 25.0, shoe.throat() - 5.0] {
                let (point, out) = last.point_out(z, 1.2);
                blob(
                    model.of(Stuff::Rivet),
                    point + out * 2.0,
                    Vec3::new(3.0, 6.0, 3.0),
                );
            }
        }
        Fastening::ElasticSides => add_elastic(model, shoe),
        Fastening::Thongs => {
            let top = if form.boot() {
                shoe.top() - 6.0
            } else {
                ankle + 60.0
            };
            let mut y = ankle * 0.8;
            while y < top {
                add_shaft(model, shoe, Stuff::Shade, (y - 2.0, y + 2.0), 2.0, 1.5);
                y += 24.0;
            }
        }
        Fastening::Drawstring => {
            let top = shoe.top();
            add_shaft(
                model,
                shoe,
                Stuff::Shade,
                (top - 14.0, top - 10.0),
                1.0,
                1.5,
            );
            for side in [-0.12, 0.12] {
                let (point, out) = on_shaft(shoe, top - 12.0, side, 1.0);
                let end = point + out * 4.0 - Vec3::Y * 26.0;
                limb(model.of(Stuff::Shade), &[(point, 1.4), (end, 1.4)]);
                blob(model.of(Stuff::Shade), end, Vec3::splat(3.5));
            }
        }
        _ => {}
    }
}

/// Bars of lace across the instep, and up the front of a boot; open laces lie on
/// facings standing off the vamp.
fn add_laces(model: &mut Model, shoe: &Shoe) {
    let thick = shoe.thick();
    let last = shoe.last(shoe.lift() + thick);
    let throat = shoe.throat();
    let (from, to) = (throat - 15.0, throat + 40.0);
    if shoe.form.fastening == Fastening::OpenLaces {
        for side in [-1.0f32, 1.0] {
            let facing =
                |u: f32, v: f32| last.point_out(from + (to - from) * v, side * (0.12 + 0.45 * u));
            sheet(model, Stuff::Main, &facing, thick);
        }
    }
    for at in 0..5 {
        let z = from + (to - from) * at as f32 / 4.0;
        let (left, out) = last.point_out(z, -0.35);
        let (right, _) = last.point_out(z, 0.35);
        let lift = out * (thick + 1.5);
        limb(
            model.of(Stuff::Shade),
            &[(left + lift, 1.4), (right + lift, 1.4)],
        );
    }
    if shoe.form.boot() {
        let mut y = shoe.foot.frame.ankle as f32 * 0.9;
        while y < shoe.top() - 10.0 {
            let (left, _) = on_shaft(shoe, y, -0.35, 1.5);
            let (right, _) = on_shaft(shoe, y, 0.35, 1.5);
            limb(model.of(Stuff::Shade), &[(left, 1.4), (right, 1.4)]);
            y += 20.0;
        }
    }
}

/// A strap over the instep, buckled at the outside; boots have theirs round the ankle
/// (jodhpurs) or the top (engineer and knee boots) as well.
fn add_strap(model: &mut Model, shoe: &Shoe) {
    let last = shoe.last(shoe.lift() + shoe.thick() + 0.5);
    let buckled = shoe.form.fastening == Fastening::Buckle;
    let instep = 0.55;
    band(model, Stuff::Shade, shoe, &last, instep, 16.0);
    if buckled {
        let (heel, toes) = (shoe.foot.heel(), shoe.foot.toes());
        let (point, out) = last.point_out(heel + (toes - heel) * instep, 1.3);
        blob(
            model.of(Stuff::Trim),
            point + out * 3.0,
            Vec3::new(4.0, 7.0, 7.0),
        );
    }
    if !shoe.form.boot() {
        return;
    }
    let y = match shoe.form.height {
        Height::Ankle => shoe.foot.frame.ankle as f32 + 12.0,
        _ => shoe.top() - 30.0,
    };
    add_shaft(model, shoe, Stuff::Shade, (y - 8.0, y + 8.0), 2.0, 2.0);
    if buckled {
        let (point, out) = on_shaft(shoe, y, OUTSIDE, 2.0);
        blob(
            model.of(Stuff::Trim),
            point + out * 3.0,
            Vec3::new(7.0, 7.0, 4.0),
        );
    }
}

/// Darker gussets down each side of a Chelsea boot's shaft.
fn add_elastic(model: &mut Model, shoe: &Shoe) {
    let (from, to) = (shoe.foot.frame.ankle as f32 * 0.8, shoe.top() - 4.0);
    for side in [-1.0f32, 1.0] {
        let gusset = |u: f32, v: f32| {
            let y = from + (to - from) * v;
            let half = 0.12 + 0.3 * v;
            on_shaft(shoe, y, side * (FRAC_PI_2 + half * (2.0 * u - 1.0)), 0.3)
        };
        plate(model.of(Stuff::Shade), &grid(&gusset, 4, 4), 1.0, false);
    }
}
