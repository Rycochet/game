//! Pieces several kinds of foot armour share: sheets over the last, rings round the
//! ankle and the foot, and the strap under it.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::last::Last;
use crate::item_art::armour::legs::Leg;
use crate::item_art::armour::surface::{Surface, grid, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};

/// A sheet over part of a surface, finely enough to follow a toe's curves.
pub(super) fn sheet(model: &mut Model, stuff: Stuff, surface: Surface, thick: f32) {
    plate(model.of(stuff), &grid(surface, 12, 12), thick, false);
}

/// A band round the leg at the ankle, from `low` at the sides to `high` above the
/// ground, this far out from the leg's middle, over `turn` either side of the front
/// (π: all round). Its lower edge rises over the instep and the heel, saddle-shaped,
/// so it rides over the foot.
pub(super) fn ankle_band(
    model: &mut Model,
    stuff: Stuff,
    leg: &Leg,
    (low, high): (f32, f32),
    radius: f32,
    turn: f32,
    thick: f32,
) {
    let band = |u: f32, v: f32| {
        let angle = -turn + 2.0 * turn * u;
        let bottom = (low + 32.0 * angle.cos().powi(2)).min(high - 6.0);
        leg.round(high + (bottom - high) * v, angle, radius)
    };
    if turn >= PI {
        plate(model.of(stuff), &ring_grid(&band, 3, 16), thick, true);
    } else {
        plate(model.of(stuff), &grid(&band, 3, 12), thick, false);
    }
}

/// A band right round the foot at z, `wide` along it: a strap under the shoe, or a cord.
pub(super) fn foot_band(model: &mut Model, stuff: Stuff, last: &Last, z: f32, wide: f32) {
    let band = |u: f32, v: f32| last.point_out(z - wide / 2.0 + wide * v, -PI + 2.0 * PI * u);
    plate(model.of(stuff), &ring_grid(&band, 1, 16), 1.2, true);
}

/// A flat sole on the ground under the whole last.
pub(super) fn sole(model: &mut Model, last: &Last) {
    let (back, tip) = (last.back(), last.tip());
    let flat = |u: f32, v: f32| {
        let z = back + (tip - back) * v;
        let (wide, _, _) = last.section(z);
        let edge = last.at(z, -PI / 2.0 + PI * u);
        let x = last.foot.x() + (edge.x - last.foot.x()).signum() * wide * (2.0 * u - 1.0).abs();
        (Vec3::new(x, 2.0, z), Vec3::NEG_Y)
    };
    plate(model.of(Stuff::Backing), &grid(&flat, 10, 6), 3.0, false);
}
