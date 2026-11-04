//! Where things are on a leg: points by height from the ground and how far round, as
//! `item_art/body.rs` lofts the bare leg, straight down from its hip joint.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use shared::armour::Style;
use shared::body::{Frame, Side};

/// Round the leg: 0 is its front; a quarter turn its outside, away from the other
/// leg; half a turn (π) its back; minus a quarter its inside.
pub(super) const FRONT: f32 = 0.0;
pub(super) const OUTSIDE: f32 = FRAC_PI_2;
/// Where a knee's fan and the outer ridges face: the outside, turned a little to the
/// front, so they show from the front.
pub(super) const WING: f32 = 0.35 * PI;

/// One leg of a frame, straight down from its hip.
pub(in crate::item_art) struct Leg<'a> {
    pub frame: &'a Frame,
    pub side: Side,
}

impl Leg<'_> {
    fn f(value: i32) -> f32 {
        value as f32
    }

    /// The leg's middle at this height.
    fn centre(&self, height: f32) -> Vec3 {
        let x = self.side.sign() as f32 * Self::f(self.frame.hip_apart);
        Vec3::new(x, height, 0.0)
    }

    /// The way out from the leg at this turn.
    pub fn out(&self, angle: f32) -> Vec3 {
        Vec3::new(self.side.sign() as f32 * angle.sin(), 0.0, angle.cos())
    }

    /// The way across the leg at this turn, round it towards the outside.
    pub fn across(&self, angle: f32) -> Vec3 {
        self.out(angle + FRAC_PI_2)
    }

    /// A point this far from the leg's middle, with the way out from it.
    pub fn round(&self, height: f32, angle: f32, radius: f32) -> (Vec3, Vec3) {
        let out = self.out(angle);
        (self.centre(height) + out * radius, out)
    }

    /// A point `lift` millimetres off the skin, with the way out from it.
    pub fn at(&self, height: f32, angle: f32, lift: f32) -> (Vec3, Vec3) {
        self.round(height, angle, self.skin(height) + lift)
    }

    pub fn knee(&self) -> f32 {
        Self::f(self.frame.knee)
    }

    pub fn ankle(&self) -> f32 {
        Self::f(self.frame.ankle)
    }

    /// Where the leg leaves the trunk.
    pub fn crotch(&self) -> f32 {
        Self::f(self.frame.hip - self.frame.thigh)
    }

    /// The skin's radius at this height, as the bare leg is lofted: the thigh's at the
    /// hip, the knee's, the calf's a third of the way down to the ankle, the ankle's.
    pub fn skin(&self, height: f32) -> f32 {
        let frame = self.frame;
        let (hip, knee, ankle) = (Self::f(frame.hip), self.knee(), self.ankle());
        let calf = knee + (ankle - knee) * 0.3;
        let points = [
            (hip, Self::f(frame.thigh)),
            (knee, Self::f(frame.knee_radius)),
            (calf, Self::f(frame.calf)),
            (ankle, Self::f(frame.ankle_radius)),
        ];
        if height >= hip {
            return points[0].1;
        }
        for pair in points.windows(2) {
            let ((high, upper), (low, lower)) = (pair[0], pair[1]);
            if height >= low {
                return lower + (upper - lower) * (height - low) / (high - low);
            }
        }
        points[3].1
    }
}

/// How far out a style's plates stand at a turn, against round: Dwarven are eight-sided
/// and Kastenbrust boxy, their flats as far out as round and their corners further;
/// others round.
pub(super) fn profile(style: Style, angle: f32) -> f32 {
    let polygon = |sides: f32| {
        let step = PI * 2.0 / sides;
        let half = step / 2.0;
        1.0 / ((angle + half).rem_euclid(step) - half).cos()
    };
    match style {
        Style::Dwarven => polygon(8.0),
        Style::Kastenbrust => 1.0 + (polygon(4.0) - 1.0) * 0.5,
        _ => 1.0,
    }
}
