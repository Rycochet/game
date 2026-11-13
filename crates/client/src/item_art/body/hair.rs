//! Hair on the head and beards, drawn round the skull in the hair's own colour: a
//! surface a little off the scalp, cut at the hairline, with locks, knots and plaits
//! added to it. Under a helmet only what hangs below its rim is drawn.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;
use shared::body::{Beard, Hair, HairStyle, Layout};

use super::trunk::Axes;
use super::{Triangle, features};

mod beard;
mod locks;
mod scalp;

/// What a head's hair is under, in the layout's millimetres.
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::item_art) struct Cover {
    /// The heights of a helmet's lower edge, sixteen from the front round to the left
    /// (`HeadForm::rim`'s points): nothing above it is drawn.
    pub rim: Option<[f32; 16]>,
    /// Hidden altogether, as under an aventail or a coif hanging from the helmet.
    pub hidden: bool,
    /// The chin covered, as by a bevor: no beard.
    pub chin_covered: bool,
}

impl Cover {
    /// The rim's height at this angle from the front (towards the left as it grows),
    /// or no limit without a helmet.
    pub(super) fn rim_at(&self, turn: f32) -> f32 {
        let Some(rim) = self.rim else {
            return f32::MAX;
        };
        let at = turn.rem_euclid(TAU) / TAU * 16.0;
        let (first, along) = (at.floor() as usize % 16, at.fract());
        rim[first] + (rim[(first + 1) % 16] - rim[first]) * along
    }
}

/// The head as drawn: its middle and half sizes, and the levels below it hair reaches.
#[derive(Clone, Copy, Debug)]
pub(super) struct Skull {
    pub middle: Vec3,
    pub across: f32,
    pub up: f32,
    pub deep: f32,
    /// Where the neck meets the shoulders.
    pub neck_base: Vec3,
    /// How far in front of the spine the chest reaches, and behind it the back, above
    /// the waist.
    pub chest_front: f32,
    pub chest_back: f32,
}

impl Skull {
    fn of(layout: &Layout) -> Self {
        let f = |value: i32| value as f32;
        let waist = layout.spine.waist.along;
        let upper = layout
            .spine
            .rings
            .iter()
            .filter(move |ring| ring.along >= waist);
        Self {
            middle: features::skull(layout),
            across: f(layout.head.half.0),
            up: f(layout.head.length) / 2.0,
            deep: f(layout.head.half.1),
            neck_base: Axes::of(layout).at(f(layout.spine.neck_base)),
            chest_front: f(upper.clone().map(|ring| ring.belly).max().unwrap_or(0)),
            chest_back: f(upper.map(|ring| ring.back).max().unwrap_or(0)),
        }
    }

    /// A point `grow` millimetres off the head, `turn` round from the front (towards
    /// the left) and `down` from the crown (a quarter turn at the widest). Below the
    /// widest it hangs straight down, flaring out a little below the chin.
    pub fn at(&self, turn: f32, down: f32, grow: f32) -> Vec3 {
        let (across, up, deep) = (self.across + grow, self.up + grow, self.deep + grow);
        let (out, y) = if down <= FRAC_PI_2 {
            (down.sin(), up * down.cos())
        } else {
            let below = (down - FRAC_PI_2) * up;
            (1.0 + (below - up).max(0.0) * 0.003, -below)
        };
        self.middle + Vec3::new(across * out * turn.sin(), y, deep * out * turn.cos())
    }

    /// A point `grow` millimetres off the head all the way round it, `down` from the
    /// crown to the bottom of the chin (half a turn): for beards.
    pub fn round(&self, turn: f32, down: f32, grow: f32) -> Vec3 {
        let (across, up, deep) = (self.across + grow, self.up + grow, self.deep + grow);
        let out = down.sin();
        self.middle
            + Vec3::new(
                across * out * turn.sin(),
                up * down.cos(),
                deep * out * turn.cos(),
            )
    }

    /// How far down from the crown (as `at` measures it) this height is.
    pub fn down_to(&self, y: f32, grow: f32) -> f32 {
        let up = self.up + grow;
        let above = y - self.middle.y;
        if above >= 0.0 {
            (above / up).min(1.0).acos()
        } else {
            FRAC_PI_2 - above / up
        }
    }

    /// The angle round from the front of this point, as `at` measures it.
    pub fn turn_of(&self, point: Vec3) -> f32 {
        let from = point - self.middle;
        (from.x / self.across).atan2(from.z / self.deep)
    }
}

/// The hair on this layout's head, styled so, under this. Empty for none.
pub(in crate::item_art) fn hair(layout: &Layout, hair: Hair, cover: Cover) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    if cover.hidden || hair == Hair::NONE {
        return triangles;
    }
    let skull = Skull::of(layout);
    match hair.style {
        HairStyle::Bald => {}
        HairStyle::Crest => locks::crest(&mut triangles, &skull),
        HairStyle::Curly => locks::curls(&mut triangles, &skull, &cover),
        style => scalp::cap(&mut triangles, &skull, &cover, style),
    }
    match hair.style {
        HairStyle::TiedBack => locks::tail(&mut triangles, &skull, &cover),
        HairStyle::Braids => locks::braids(&mut triangles, &skull, &cover),
        HairStyle::Topknot => locks::topknot(&mut triangles, &skull),
        _ => {}
    }
    if hair.beard != Beard::None && !cover.chin_covered {
        beard::beard(&mut triangles, &skull, hair.beard);
    }
    // Anything left wholly inside the helmet is hidden by it.
    triangles.retain(|triangle| {
        triangle
            .iter()
            .any(|&corner| corner.y < cover.rim_at(skull.turn_of(corner)))
    });
    triangles
}

#[cfg(test)]
mod tests;
