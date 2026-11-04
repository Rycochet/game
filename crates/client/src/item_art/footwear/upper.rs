//! Over the foot: a low shoe's vamp and quarters, up to its opening, or a boot's foot
//! and its shaft up the leg, with the top edge the form has.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::footwear::Top;

use super::shoe::Shoe;
use crate::item_art::armour::surface::{Surface, grid, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};

/// How far round from the top the upper reaches: down to the sole.
pub(super) const SIDES: f32 = 2.65;
/// Where a boot's shaft starts, as a share of the ankle's height.
const SHAFT_FROM: f32 = 0.7;

/// A sheet over part of a surface, finely enough to follow a toe's curves.
pub(super) fn sheet(model: &mut Model, stuff: Stuff, surface: Surface, thick: f32) {
    plate(model.of(stuff), &grid(surface, 14, 16), thick, false);
}

/// The upper of a closed shoe or boot.
pub(super) fn add_upper(model: &mut Model, shoe: &Shoe) {
    let last = shoe.last(shoe.lift());
    let (back, tip) = (last.back(), last.tip());
    let thick = shoe.thick();
    if shoe.form.boot() {
        sheet(model, Stuff::Main, &last.patch((back, tip), SIDES), thick);
        let from = shoe.foot.frame.ankle as f32 * SHAFT_FROM;
        add_shaft(model, shoe, Stuff::Main, (from, 0.0), 0.0, thick);
        add_top(model, shoe, thick);
        return;
    }
    let throat = shoe.throat();
    sheet(model, Stuff::Main, &last.patch((throat, tip), SIDES), thick);
    if shoe.form.backless {
        return;
    }
    let collar = shoe.top();
    for side in [-1.0f32, 1.0] {
        let quarter = |u: f32, v: f32| {
            let z = back + (throat + 8.0 - back) * v;
            let (_, high, middle) = last.section(z);
            let open = ((collar - middle) / high.max(1.0)).clamp(-1.0, 1.0).acos();
            last.point_out(z, side * (open + (SIDES - open) * u))
        };
        sheet(model, Stuff::Main, &quarter, thick);
    }
}

/// How high the shaft's top edge is at this turn round the leg (0 at the front).
pub(super) fn top_at(shoe: &Shoe, angle: f32) -> f32 {
    let (high, length) = (shoe.top(), shoe.length());
    match shoe.form.top {
        Top::Notch => high - 0.14 * length * angle.cos().max(0.0).powi(6),
        Top::Scalloped => high - 0.07 * length * angle.cos().powi(2),
        Top::Uneven => high + 0.1 * length * angle.cos() - 0.05 * length,
        _ => high,
    }
}

/// How far a bucket top flares out at this height and turn: little on the inside, so
/// a pair's tops do not meet between the knees.
fn flare(shoe: &Shoe, y: f32, angle: f32) -> f32 {
    if shoe.form.top != Top::Bucket {
        return 0.0;
    }
    let depth = shoe.length() * 0.4;
    let into = ((y - (shoe.top() - depth)) / depth).max(0.0);
    40.0 * into * into * (0.6 + 0.4 * angle.sin())
}

/// A tube round the leg from `from` up to the top edge (or `to`, if more than 0), this
/// far further out than the upper.
pub(super) fn add_shaft(
    model: &mut Model,
    shoe: &Shoe,
    stuff: Stuff,
    (from, to): (f32, f32),
    out: f32,
    thick: f32,
) {
    let lift = shoe.lift() + out;
    let tube = |u: f32, v: f32| {
        let angle = u * TAU;
        let top = if to > 0.0 { to } else { top_at(shoe, angle) };
        let y = from + (top - from) * v;
        shoe.leg.at(y, angle, lift + flare(shoe, y, angle))
    };
    plate(model.of(stuff), &ring_grid(&tube, 8, 28), thick, true);
}

/// A cuff turned down in the second colour, or fur, round the top.
fn add_top(model: &mut Model, shoe: &Shoe, thick: f32) {
    let (high, length) = (shoe.top(), shoe.length());
    match shoe.form.top {
        Top::Cuff => add_shaft(
            model,
            shoe,
            Stuff::Cloth,
            (high - 0.28 * length, high + 2.0),
            thick + 1.5,
            thick,
        ),
        Top::Fur => add_shaft(
            model,
            shoe,
            Stuff::Charge,
            (high - 0.16 * length, high + 8.0),
            thick + 4.0,
            9.0,
        ),
        _ => {}
    }
}

/// Points round the leg at a height, for things laid on a boot's shaft.
pub(super) fn on_shaft(shoe: &Shoe, y: f32, angle: f32, out: f32) -> (Vec3, Vec3) {
    shoe.leg.at(
        y,
        angle,
        shoe.lift() + shoe.thick() + out + flare(shoe, y, angle),
    )
}
