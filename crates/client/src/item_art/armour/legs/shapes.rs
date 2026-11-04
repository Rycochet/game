//! Shapes leg pieces are made of: tubes and parts of tubes round the leg, the knee's
//! cop and fan, and spikes and fins.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::Style;

use super::path::{FRONT, Leg, WING, profile};
use crate::item_art::armour::surface::{grid, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

/// Points round a whole tube.
const ROUND: usize = 24;

/// Part of a tube round the leg: from `top` (a height, which may change round it, for
/// points and slopes) down to `bottom`, round from one turn to another (a whole turn
/// closes it), `lift` off the skin and shaped by the style. `swell` adds that many
/// millimetres at the front of the calf, for shaped Classical greaves.
#[derive(Clone, Copy)]
pub(super) struct Tube<'a> {
    pub leg: &'a Leg<'a>,
    pub top: &'a dyn Fn(f32) -> f32,
    pub bottom: f32,
    pub turn: (f32, f32),
    pub lift: f32,
    pub style: Style,
    pub swell: f32,
}

impl Tube<'_> {
    pub fn whole(&self) -> bool {
        self.turn.1 - self.turn.0 >= TAU - 0.01
    }

    /// The point at `u` round and `v` down it, and the way out.
    pub fn at(&self, u: f32, v: f32) -> (Vec3, Vec3) {
        let angle = self.turn.0 + (self.turn.1 - self.turn.0) * u;
        let top = (self.top)(angle);
        let height = top + (self.bottom - top) * v;
        let calf = {
            let leg = self.leg;
            let middle = leg.knee() + (leg.ankle() - leg.knee()) * 0.3;
            let along = (1.0 - ((height - middle) / 150.0).powi(2)).max(0.0);
            // Over the front and the sides, as the calf muscles and shin bone show.
            self.swell * along * (0.6 + 0.4 * (2.0 * angle).cos().abs())
        };
        let radius = (self.leg.skin(height) + self.lift + calf) * profile(self.style, angle);
        self.leg.round(height, angle, radius)
    }

    /// The tube as a plate of this thickness.
    pub fn add(&self, model: &mut Model, stuff: Stuff, thick: f32) {
        let rows = ((((self.top)(self.turn.0) - self.bottom) / 25.0).ceil() as usize).max(2);
        let surface = |u: f32, v: f32| self.at(u, v);
        if self.whole() {
            plate(
                model.of(stuff),
                &ring_grid(&surface, rows, ROUND),
                thick,
                true,
            );
        } else {
            let share = (self.turn.1 - self.turn.0) / TAU;
            let columns = ((ROUND as f32 * share).ceil() as usize).max(3);
            plate(
                model.of(stuff),
                &grid(&surface, rows, columns),
                thick,
                false,
            );
        }
    }
}

/// A flat top.
pub(super) fn flat(height: f32) -> impl Fn(f32) -> f32 {
    move |_| height
}

/// Round the whole leg, or over its front and outside (`closed` false), as a piece
/// strapped on behind leaves the back and inside bare.
pub(super) fn turn(closed: bool) -> (f32, f32) {
    if closed {
        (-PI, PI)
    } else {
        (-0.4 * PI, 0.85 * PI)
    }
}

/// A band round the leg from `top` to `bottom` (heights), flat at both ends.
#[allow(clippy::too_many_arguments)]
pub(super) fn band(
    model: &mut Model,
    stuff: Stuff,
    leg: &Leg,
    (top, bottom): (f32, f32),
    turned: (f32, f32),
    lift: f32,
    style: Style,
    thick: f32,
) {
    let edge = flat(top);
    let tube = Tube {
        leg,
        top: &edge,
        bottom,
        turn: turned,
        lift,
        style,
        swell: 0.0,
    };
    tube.add(model, stuff, thick);
}

/// A cop over the front of the knee, from 70 mm above it to 70 below, bulging out,
/// `size` across as a share of a half turn.
pub(super) fn cop(model: &mut Model, stuff: Stuff, leg: &Leg, lift: f32, style: Style, size: f32) {
    let knee = leg.knee();
    let surface = |u: f32, v: f32| {
        let angle = FRONT + (u - 0.5) * PI * size;
        let height = knee + 70.0 - 140.0 * v;
        let bulge = 18.0 * (PI * v).sin() * (1.0 - (2.0 * u - 1.0).powi(2));
        let radius = (leg.skin(height) + lift + bulge) * profile(style, angle);
        leg.round(height, angle, radius)
    };
    plate(model.of(stuff), &grid(&surface, 6, 10), 1.6, false);
}

/// The fan on the outside of the knee, turned a little forwards: `size` mm across,
/// pointed up and down when `pointed`, curving round the leg towards its edge.
pub(super) fn fan(model: &mut Model, stuff: Stuff, leg: &Leg, lift: f32, size: f32, pointed: bool) {
    let knee = leg.knee();
    let (centre, out) = leg.at(knee, WING, lift + 8.0);
    let across = leg.across(WING);
    let rows: Vec<Vec<(Vec3, Vec3)>> = (0..=4)
        .map(|row| {
            let reach = row as f32 / 4.0;
            (0..16)
                .map(|column| {
                    let turn = column as f32 / 16.0 * TAU;
                    let point = if pointed {
                        1.0 + 0.6 * turn.sin().abs().powi(6)
                    } else {
                        1.0
                    };
                    let half = size / 2.0 * reach * point;
                    let curl = -8.0 * reach * reach;
                    let at =
                        centre + Vec3::Y * half * turn.sin() + across * half * turn.cos() * 0.8;
                    (at + out * curl, out)
                })
                .collect()
        })
        .collect();
    plate(model.of(stuff), &rows, 1.5, true);
}

/// A spike standing out from a point, this long.
pub(super) fn spike(model: &mut Model, stuff: Stuff, base: Vec3, out: Vec3, long: f32) {
    limb(
        model.of(stuff),
        &[(base, long * 0.22), (base + out * long, 0.6)],
    );
}

/// Narrow strips down the leg, spread round a tube: splints, flutes, ridges.
pub(super) fn strips(
    model: &mut Model,
    stuff: Stuff,
    tube: &Tube,
    count: usize,
    width: f32,
    raise: f32,
) {
    for strip in 0..count {
        let middle = (strip as f32 + 0.5) / count as f32;
        let edge = |u: f32, v: f32| {
            let (point, out) = tube.at(middle + (u - 0.5) * width, v);
            (point + out * raise, out)
        };
        plate(model.of(stuff), &grid(&edge, 4, 1), 1.4, false);
    }
}
