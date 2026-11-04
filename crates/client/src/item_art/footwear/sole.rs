//! Under the foot: the sole, from the ground up to the bare foot's underside, the heel
//! or teeth under it, and hobnails or irons.

use bevy::prelude::*;
use shared::footwear::{Heel, Toe, Under};

use super::shoe::Shoe;
use crate::item_art::armour::surface::grid;
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::blob;

/// How far the sole reaches out past the upper all round.
const WELT: f32 = 3.0;

/// A flat slab under the foot, following the outline of the last, from `low` to `high`
/// above the ground, between `back` and `front` along it.
fn slab(
    model: &mut Model,
    stuff: Stuff,
    shoe: &Shoe,
    (low, high): (f32, f32),
    (back, front): (f32, f32),
    narrow: f32,
) {
    let last = shoe.last(shoe.lift() + WELT);
    let spring = matches!(shoe.form.toe, Toe::Upturned)
        || shoe.form.build == shared::footwear::Build::Wooden;
    let flat = |u: f32, v: f32| {
        let z = back + (front - back) * v;
        let (wide, _, _) = last.section(z);
        let wide = wide.max(4.0) * narrow;
        // Clogs and upturned toes rise at the front.
        let rise = if spring {
            (v - 0.75).max(0.0) * 4.0 * 12.0
        } else {
            0.0
        };
        // A broad sole keeps clear of the other foot's.
        let sign = shoe.foot.side.sign() as f32;
        let x = shoe.foot.x() + wide * (2.0 * u - 1.0);
        let x = sign * (x * sign).max(4.0);
        (Vec3::new(x, (low + high) / 2.0 + rise, z), Vec3::Y)
    };
    plate(model.of(stuff), &grid(&flat, 12, 8), high - low, false);
}

/// The sole and what is under it.
pub(super) fn add_sole(model: &mut Model, shoe: &Shoe) {
    let form = shoe.form;
    let last = shoe.last(shoe.lift() + WELT);
    let (back, tip) = (last.back(), last.tip());
    let sole = shoe.sole();
    // The foot stands on it, so it is under the ground the frame stands on.
    slab(
        model,
        Stuff::Backing,
        shoe,
        (-sole, 0.0),
        (back + 1.0, tip - 1.0),
        1.0,
    );
    let ground = -sole;
    match form.heel {
        Heel::Flat => {}
        Heel::Teeth => add_teeth(model, shoe, (back, tip), ground),
        heel => {
            let high = form.heel_mm as f32;
            let length = shoe.length() * 0.27;
            // A Louis heel narrows towards the ground; a riding heel is set back.
            let narrow = if heel == Heel::Louis { 0.6 } else { 0.85 };
            slab(
                model,
                Stuff::Backing,
                shoe,
                (ground - high, ground),
                (back + 2.0, back + length),
                narrow,
            );
        }
    }
    let bottom = ground
        - if matches!(form.heel, Heel::Low | Heel::Riding | Heel::Louis) {
            form.heel_mm as f32
        } else {
            0.0
        };
    match form.under {
        Under::Nothing => {}
        Under::Hobnails => add_hobnails(model, shoe, (back, tip), ground, bottom),
        Under::Irons => {
            let length = shoe.length() * 0.2;
            slab(
                model,
                Stuff::Trim,
                shoe,
                (bottom - 2.0, bottom),
                (back + 4.0, back + length),
                0.9,
            );
            slab(
                model,
                Stuff::Trim,
                shoe,
                (ground - 2.0, ground),
                (tip - length, tip - 6.0),
                0.85,
            );
        }
    }
}

/// Two wooden teeth or blocks across the board: under the heel and the ball.
fn add_teeth(model: &mut Model, shoe: &Shoe, (back, tip): (f32, f32), board: f32) {
    let high = shoe.form.heel_mm as f32;
    let length = tip - back;
    for at in [0.18, 0.68] {
        let z = back + length * at;
        slab(
            model,
            Stuff::Backing,
            shoe,
            (board - high, board),
            (z - 9.0, z + 9.0),
            0.9,
        );
    }
}

/// Rows of iron studs round the edge of the sole and under the heel.
fn add_hobnails(model: &mut Model, shoe: &Shoe, (back, tip): (f32, f32), ground: f32, heel: f32) {
    let last = shoe.last(shoe.lift());
    let heel_end = back + shoe.length() * 0.27;
    for row in 0..10 {
        let z = back + 8.0 + (tip - back - 16.0) * row as f32 / 9.0;
        let (wide, _, _) = last.section(z);
        let y = if z < heel_end { heel } else { ground };
        for side in [-0.75, 0.0, 0.75] {
            let x = shoe.foot.x() + wide * side;
            blob(
                model.of(Stuff::Trim),
                Vec3::new(x, y - 1.0, z),
                Vec3::new(3.0, 1.5, 3.0),
            );
        }
    }
}
