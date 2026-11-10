//! Where things are round the hips: points by height and how far round the trunk, as
//! `item_art/body.rs` lofts it, carried straight down below the crotch so skirts hang
//! clear of the legs. Angles are the torso shell's: 0 is the body's left, a quarter turn
//! its front, half a turn its right, three quarters its back.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use shared::body::Frame;

use crate::item_art::body::trunk_half;

/// Round the trunk: the front, and the back.
pub(super) const FRONT: f32 = FRAC_PI_2;
pub(super) const BACK: f32 = 3.0 * FRAC_PI_2;

/// The hips of a frame.
#[derive(Clone, Copy)]
pub(in crate::item_art) struct Hips<'a> {
    pub frame: &'a Frame,
}

impl Hips<'_> {
    fn f(value: i32) -> f32 {
        value as f32
    }

    /// Where every piece starts, tucked under the torso piece's hem.
    pub fn top(&self) -> f32 {
        Self::f(self.frame.waist + 30)
    }

    pub fn waist(&self) -> f32 {
        Self::f(self.frame.waist)
    }

    pub fn hip(&self) -> f32 {
        Self::f(self.frame.hip)
    }

    /// Where the legs leave the trunk.
    pub fn crotch(&self) -> f32 {
        Self::f(self.frame.hip - self.frame.thigh)
    }

    /// A point `out` millimetres off the trunk's skin at this height and angle, with the
    /// way out from it. Below the crotch the trunk's lowest ring carries on down.
    pub fn at(&self, height: f32, angle: f32, out: f32) -> (Vec3, Vec3) {
        let (across, deep) = trunk_half(self.frame, height.max(self.crotch()));
        let (a, b) = (across + out, deep + out);
        let point = Vec3::new(a * angle.cos(), height, b * angle.sin());
        let normal = Vec3::new(angle.cos() / a, 0.0, angle.sin() / b).normalize_or(Vec3::Z);
        (point, normal)
    }
}

/// Angles from `from` to `to`, round towards the front, as u goes 0 to 1.
pub(super) fn between(from: f32, to: f32, u: f32) -> f32 {
    from + (to - from) * u
}
