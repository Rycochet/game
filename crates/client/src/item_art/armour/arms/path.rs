//! Where things are on an arm: points by how far down from the shoulder joint and how
//! far round, as `item_art/body.rs` lofts the bare arm.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;
use shared::armour::Style;
use shared::body::{Frame, Joint, Side};

/// Round the arm: 0 is its outside, away from the body; a quarter turn its front; half
/// a turn its inside, against the body; minus a quarter its back, the point of the
/// elbow.
pub(super) const OUTSIDE: f32 = 0.0;
pub(super) const INSIDE: f32 = PI;
pub(super) const BACK: f32 = -FRAC_PI_2;
/// Where a couter's fan and the outer ridges face: the outside, turned forwards a
/// little as when the arm bends, so they show from the front.
pub(super) const WING: f32 = 0.3 * PI;

/// How far either side of the elbow a bend is spread, in millimetres down the arm, so
/// a sleeve curves round it rather than folding.
const BEND_SPREAD: f32 = 45.0;

/// One arm of a frame. The arm hangs straight from the shoulder, a little out, as the
/// frame has it; `bend` swings the forearm forwards by that many radians at the elbow,
/// for pictures of a piece alone.
pub(super) struct Arm<'a> {
    pub frame: &'a Frame,
    pub side: Side,
    pub bend: f32,
}

impl Arm<'_> {
    fn joint(&self, joint: Joint) -> Vec3 {
        let [x, y, z] = self.frame.joint(joint);
        Vec3::new(x as f32, y as f32, z as f32)
    }

    /// Down the straight arm, and millimetres along it for each millimetre down.
    fn straight(&self) -> (Vec3, f32) {
        let shoulder = self.joint(Joint::Shoulder(self.side));
        let elbow = self.joint(Joint::Elbow(self.side));
        let per_down = (elbow - shoulder) / self.elbow();
        (per_down.normalize(), per_down.length())
    }

    /// How far the arm has turned forwards at `down`: none above the elbow, all of the
    /// bend below it, turning evenly in between.
    fn turned(&self, down: f32) -> f32 {
        let from = self.elbow() - BEND_SPREAD;
        self.bend * ((down - from) / (2.0 * BEND_SPREAD)).clamp(0.0, 1.0)
    }

    /// The arm's middle at `down` millimetres below the shoulder joint: how far it has
    /// gone down the straight arm and forwards, added up over the bend.
    fn centre(&self, down: f32) -> Vec3 {
        let shoulder = self.joint(Joint::Shoulder(self.side));
        let (along, per_down) = self.straight();
        let from = self.elbow() - BEND_SPREAD;
        let (onward, forward) = if self.bend.abs() < 1e-4 || down <= from {
            (down, 0.0)
        } else {
            // Turning evenly over the spread, then straight on.
            let rate = self.bend / (2.0 * BEND_SPREAD);
            let turned = self.turned(down);
            let past = (down - from - 2.0 * BEND_SPREAD).max(0.0);
            (
                from + turned.sin() / rate + past * self.bend.cos(),
                (1.0 - turned.cos()) / rate + past * self.bend.sin(),
            )
        };
        shoulder + (along * onward + Vec3::Z * forward) * per_down
    }

    /// The way down the arm at `down`.
    pub fn down(&self, down: f32) -> Vec3 {
        let (along, _) = self.straight();
        let turned = self.turned(down);
        along * turned.cos() + Vec3::Z * turned.sin()
    }

    /// The way out from the arm at this turn, at `down`: round from its outside, which
    /// stays the same however it bends, to its front.
    pub fn out(&self, down: f32, angle: f32) -> Vec3 {
        let (along, _) = self.straight();
        let outside = Vec3::Z.cross(along) * self.side.sign() as f32;
        let turned = self.turned(down);
        let front = Vec3::Z * turned.cos() - along * turned.sin();
        outside * angle.cos() + front * angle.sin()
    }

    /// A point `lift` millimetres off the skin, with the way out from it.
    pub fn at(&self, down: f32, angle: f32, lift: f32) -> (Vec3, Vec3) {
        self.round(down, angle, self.skin(down) + lift)
    }

    /// A point this far from the arm's middle, with the way out from it.
    pub fn round(&self, down: f32, angle: f32, radius: f32) -> (Vec3, Vec3) {
        let out = self.out(down, angle);
        (self.centre(down) + out * radius, out)
    }

    /// How far down the arm the elbow is.
    pub fn elbow(&self) -> f32 {
        self.frame.elbow_length as f32
    }

    /// The skin's radius at `down`, as the bare arm is lofted: the upper arm's at the
    /// shoulder, a little over the forearm's at the elbow, 0.7 of it at the wrist.
    pub fn skin(&self, down: f32) -> f32 {
        let frame = self.frame;
        let (upper, fore) = (frame.upper_arm as f32, frame.forearm as f32);
        let elbow = frame.elbow_length as f32;
        let wrist = elbow + frame.wrist_length as f32;
        if down <= elbow {
            upper + (fore * 1.05 - upper) * (down / elbow).clamp(0.0, 1.0)
        } else {
            let along = ((down - elbow) / (wrist - elbow)).clamp(0.0, 1.0);
            fore * 1.05 + (fore * 0.7 - fore * 1.05) * along
        }
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
