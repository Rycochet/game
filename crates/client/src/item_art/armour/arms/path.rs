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

/// One arm of a frame. The upper arm hangs a little out from the shoulder and the
/// forearm, from the slightly bent elbow, nearer straight down, as the frame has them;
/// `bend` swings the forearm further forwards by that many radians at the elbow, for
/// pictures of a piece alone.
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

    /// Down the upper arm, and millimetres along it for each millimetre down.
    fn straight(&self) -> (Vec3, f32) {
        let shoulder = self.joint(Joint::Shoulder(self.side));
        let elbow = self.joint(Joint::Elbow(self.side));
        let per_down = (elbow - shoulder) / self.elbow();
        (per_down.normalize(), per_down.length())
    }

    /// Down the forearm: the frame's, swung forwards by `bend`.
    fn forearm(&self) -> Vec3 {
        let elbow = self.joint(Joint::Elbow(self.side));
        let fore = (self.joint(Joint::Wrist(self.side)) - elbow).normalize();
        let forwards = (Vec3::Z - fore * fore.z).normalize();
        fore * self.bend.cos() + forwards * self.bend.sin()
    }

    /// The angle between the upper arm and the forearm.
    fn angle(&self) -> f32 {
        self.straight().0.angle_between(self.forearm())
    }

    /// How far through the bend the arm is at `down`: 0 above it, 1 below it, evenly in
    /// between.
    fn through(&self, down: f32) -> f32 {
        let from = self.elbow() - BEND_SPREAD;
        ((down - from) / (2.0 * BEND_SPREAD)).clamp(0.0, 1.0)
    }

    /// The arm's middle at `down` millimetres below the shoulder joint: down the upper
    /// arm, round the bend turning evenly, then down the forearm.
    pub fn centre(&self, down: f32) -> Vec3 {
        let shoulder = self.joint(Joint::Shoulder(self.side));
        let (upper, per_down) = self.straight();
        let from = self.elbow() - BEND_SPREAD;
        let angle = self.angle();
        let offset = if angle < 1e-4 || down <= from {
            upper * down
        } else {
            // The way turns evenly from the upper arm's to the forearm's: added up over
            // the bend, that comes to these amounts of each.
            let t = self.through(down);
            let spread = 2.0 * BEND_SPREAD;
            let scale = spread / (angle * angle.sin());
            let of_upper = ((1.0 - t) * angle).cos() - angle.cos();
            let of_fore = 1.0 - (t * angle).cos();
            let past = (down - from - spread).max(0.0);
            upper * from
                + (upper * of_upper + self.forearm() * of_fore) * scale
                + self.forearm() * past
        };
        shoulder + offset * per_down
    }

    /// The way down the arm at `down`.
    pub fn down(&self, down: f32) -> Vec3 {
        let (upper, fore) = (self.straight().0, self.forearm());
        let angle = self.angle();
        if angle < 1e-4 {
            return upper;
        }
        let t = self.through(down);
        (upper * ((1.0 - t) * angle).sin() + fore * (t * angle).sin()) / angle.sin()
    }

    /// The way out from the arm at this turn, at `down`: round from its outside, away
    /// from the body, to its front.
    pub fn out(&self, down: f32, angle: f32) -> Vec3 {
        let along = self.down(down);
        let sign = self.side.sign() as f32;
        let outside = Vec3::Z.cross(along).normalize() * sign;
        let front = along.cross(outside) * sign;
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
