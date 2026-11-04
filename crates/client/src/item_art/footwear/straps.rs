//! What holds a sandal or a wooden sole on: straps over the foot, nets of loops, toe
//! thongs and knobs, and caligae's lattice up the shin.

use bevy::prelude::*;
use shared::footwear::{Fastening, Height};
use shared::material::Material;

use super::shoe::Shoe;
use super::upper::{add_shaft, sheet};
use crate::item_art::armour::feet::Last;
use crate::item_art::armour::{Model, Stuff};
use crate::item_art::body::{blob, limb};

/// How far round from the top a strap over the foot reaches: down to the sole.
const OVER: f32 = 2.5;

/// A strap over the foot at this share of the way from the heel to the toes, `wide`
/// millimetres along it.
pub(super) fn band(
    model: &mut Model,
    stuff: Stuff,
    shoe: &Shoe,
    last: &Last,
    share: f32,
    wide: f32,
) {
    let (heel, toes) = (shoe.foot.heel(), shoe.foot.toes());
    let z = heel + (toes - heel) * share;
    sheet(
        model,
        stuff,
        &last.patch((z - wide / 2.0, z + wide / 2.0), OVER),
        2.0,
    );
}

/// A strap round the ankle.
fn ankle_strap(model: &mut Model, shoe: &Shoe, wide: f32) {
    let ankle = shoe.foot.frame.ankle as f32;
    add_shaft(
        model,
        shoe,
        Stuff::Main,
        (ankle - wide / 2.0, ankle + wide / 2.0),
        0.0,
        2.0,
    );
}

/// Everything holding an open sole on.
pub(super) fn add_straps(model: &mut Model, shoe: &Shoe) {
    let last = shoe.last(shoe.lift());
    match shoe.form.fastening {
        Fastening::ToeThong => add_toe_thong(model, shoe, &last),
        Fastening::Buckle => {
            band(model, Stuff::Main, shoe, &last, 0.78, 22.0);
            band(model, Stuff::Main, shoe, &last, 0.5, 18.0);
            ankle_strap(model, shoe, 14.0);
            let (point, out) = last.point_out(shoe.foot.heel() + 20.0, 1.3);
            blob(model.of(Stuff::Trim), point + out * 6.0, Vec3::splat(5.0));
        }
        Fastening::Strap => band(model, Stuff::Main, shoe, &last, 0.72, 45.0),
        Fastening::Loops => {
            for share in [0.86, 0.74, 0.62, 0.5] {
                band(model, Stuff::Main, shoe, &last, share, 9.0);
            }
            ankle_strap(model, shoe, 10.0);
        }
        _ if shoe.form.height > Height::Low => add_lattice(model, shoe, &last),
        _ => {
            // Waraji's cords: over the toes, round the ankle, and down to the sides.
            band(model, Stuff::Main, shoe, &last, 0.8, 6.0);
            ankle_strap(model, shoe, 6.0);
            let ankle = shoe.foot.frame.ankle as f32;
            for side in [-1.4, 1.4] {
                let (low, _) = last.point_out(shoe.foot.toes() - 70.0, side);
                let (high, _) = shoe.leg.at(ankle, side * 0.9, shoe.lift());
                limb(model.of(Stuff::Main), &[(low, 2.0), (high, 2.0)]);
            }
        }
    }
}

/// A post between the first two toes, and a V of straps from it to the sides; a
/// paduka's is a knob with no straps.
fn add_toe_thong(model: &mut Model, shoe: &Shoe, last: &Last) {
    let foot = shoe.foot;
    let inside = -(foot.side.sign() as f32);
    let base = Vec3::new(foot.x() + inside * 14.0, 0.0, foot.toes() - 32.0);
    let top = base + Vec3::Y * 22.0;
    limb(model.of(Stuff::Main), &[(base, 3.0), (top, 3.0)]);
    if shoe.form.upper == Material::Wood {
        blob(model.of(Stuff::Main), top, Vec3::splat(9.0));
        return;
    }
    let z = foot.heel() + (foot.toes() - foot.heel()) * 0.6;
    for side in [-1.75, 1.75] {
        let (end, _) = last.point_out(z, side);
        limb(model.of(Stuff::Main), &[(top, 3.5), (end, 3.5)]);
    }
}

/// Caligae: straps across the foot, and rings up the shin to the top, joined by a
/// strap up the front.
fn add_lattice(model: &mut Model, shoe: &Shoe, last: &Last) {
    for share in [0.82, 0.68, 0.54, 0.4] {
        band(model, Stuff::Main, shoe, last, share, 10.0);
    }
    let ankle = shoe.foot.frame.ankle as f32;
    let top = shoe.top();
    let mut y = ankle - 10.0;
    while y < top {
        add_shaft(model, shoe, Stuff::Main, (y - 5.0, y + 5.0), 0.0, 2.0);
        y += 32.0;
    }
    let (low, _) = last.point_out(shoe.foot.toes() - 60.0, 0.0);
    let (high, _) = shoe.leg.at(top - 5.0, 0.0, shoe.lift() + 2.0);
    let (middle, _) = shoe.leg.at(ankle, 0.0, shoe.lift() + 2.0);
    limb(
        model.of(Stuff::Main),
        &[(low, 4.0), (middle, 4.0), (high, 4.0)],
    );
}
