//! Shapes arm pieces are made of: tubes and parts of tubes round the arm, the elbow's
//! cup and fan, and spikes.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::Style;

use super::path::{Arm, BACK, WING, profile};
use crate::item_art::armour::surface::{grid, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

/// Points round a whole tube.
const ROUND: usize = 24;

/// Part of a tube round the arm: from `top` (which may change round it, for points)
/// down to `bottom`, round from one turn to another (a whole turn closes it), `lift`
/// off the skin and shaped by the style.
pub(super) struct Tube<'a> {
    pub arm: &'a Arm<'a>,
    pub top: &'a dyn Fn(f32) -> f32,
    pub bottom: f32,
    pub turn: (f32, f32),
    pub lift: f32,
    pub style: Style,
}

impl Tube<'_> {
    pub fn whole(&self) -> bool {
        self.turn.1 - self.turn.0 >= TAU - 0.01
    }

    /// The point at `u` round and `v` down it, and the way out.
    pub fn at(&self, u: f32, v: f32) -> (Vec3, Vec3) {
        let angle = self.turn.0 + (self.turn.1 - self.turn.0) * u;
        let top = (self.top)(angle);
        let down = top + (self.bottom - top) * v;
        let radius = (self.arm.skin(down) + self.lift) * profile(self.style, angle);
        self.arm.round(down, angle, radius)
    }

    /// The tube as a plate of this thickness.
    pub fn add(&self, model: &mut Model, stuff: Stuff, thick: f32) {
        let rows = (((self.bottom - (self.top)(self.turn.0)) / 25.0).ceil() as usize).max(2);
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
pub(super) fn flat(down: f32) -> impl Fn(f32) -> f32 {
    move |_| down
}

/// A whole tube from `top` to `bottom`, flat at both ends.
pub(super) fn ring(
    model: &mut Model,
    stuff: Stuff,
    arm: &Arm,
    (top, bottom): (f32, f32),
    lift: f32,
    style: Style,
    thick: f32,
) {
    let edge = flat(top);
    let tube = Tube {
        arm,
        top: &edge,
        bottom,
        turn: (-PI, PI),
        lift,
        style,
    };
    tube.add(model, stuff, thick);
}

/// The elbow's cup over its point, from 60 mm above to 60 below, bulging out.
pub(super) fn cup(model: &mut Model, stuff: Stuff, arm: &Arm, lift: f32, style: Style) {
    let elbow = arm.elbow();
    let surface = |u: f32, v: f32| {
        let angle = BACK + (u - 0.5) * PI * 1.15;
        let down = elbow - 60.0 + 120.0 * v;
        let bulge = 14.0 * (PI * v).sin() * (1.0 - (2.0 * u - 1.0).powi(2));
        let radius = (arm.skin(down) + lift + bulge) * profile(style, angle);
        arm.round(down, angle, radius)
    };
    plate(model.of(stuff), &grid(&surface, 6, 10), 1.6, false);
}

/// The fan on the outside of the elbow, turned a little forwards: `size` mm across the middle, pointed up and
/// down when `pointed`, curving round the arm towards its edge.
pub(super) fn fan(model: &mut Model, stuff: Stuff, arm: &Arm, lift: f32, size: f32, pointed: bool) {
    let elbow = arm.elbow();
    let (centre, out) = arm.at(elbow, WING, lift + 6.0);
    let down = arm.down(elbow);
    let across = out.cross(down).normalize();
    let rows: Vec<Vec<(Vec3, Vec3)>> = (0..=4)
        .map(|row| {
            let reach = row as f32 / 4.0;
            (0..16)
                .map(|column| {
                    let turn = column as f32 / 16.0 * TAU;
                    let point = if pointed {
                        1.0 + 0.6 * turn.cos().abs().powi(6)
                    } else {
                        1.0
                    };
                    let half = size / 2.0 * reach * point;
                    let curl = -8.0 * reach * reach;
                    let at = centre + down * half * turn.cos() + across * half * turn.sin() * 0.8;
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

/// Narrow strips down the arm, spread round between two turns: splints, flutes, ridges.
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
