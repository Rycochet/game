//! Where things are on a standing body, in millimetres, for building what is worn on
//! it: every worn thing's model is made in place on one of these, so the same model
//! draws its own picture and, later, a character wearing it. See
//! `docs/context/shared/body.md`.
//!
//! Lengths are at an ordinary person's scale, as blades' are: an item's size goes on
//! top. Whole millimetres only, so both sides can work out weights from them alike.

use crate::creature::CreatureKind;

mod beasts;
mod biped_view;
mod bipeds;
mod blend;
mod hair;
mod layout;
mod limbs;
mod parts;
mod proportions;
mod scaled;
mod taurs;
mod trig;

pub use beasts::{BEASTS, Beast, Bird, Fourfoot, Plan};
pub use bipeds::{Biped, FIRST_BIPEDS};
pub use blend::WHOLE as BLEND_WHOLE;
pub use hair::{Beard, Hair, HairStyle};
pub use layout::{
    Bone, Girdle, Head, Layout, Limb, MAX_BONES, MAX_LIMBS, Neck, Role, Spine, Station, Upper,
};
pub use limbs::has_hands;
pub use parts::{Feet, Horns, NONE, Parts, Snout, Wings};
pub use proportions::{PERSON, Proportions};
pub use taurs::{TAURS, Taur};

/// Which side of the body, as its owner sees it: left is towards the picture's right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub const BOTH: [Self; 2] = [Self::Left, Self::Right];

    /// Which way along x this side is: the body faces +z, so its left is +x.
    pub fn sign(self) -> i32 {
        match self {
            Self::Left => 1,
            Self::Right => -1,
        }
    }
}

/// A point on the body that worn things are placed by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Joint {
    /// The top of the head.
    Crown,
    /// The middle of the head.
    Skull,
    Chin,
    /// Where the neck meets the shoulders, in the middle.
    NeckBase,
    Shoulder(Side),
    Elbow(Side),
    Wrist(Side),
    Fingertips(Side),
    /// The middle of the chest, at the nipples.
    Chest,
    /// The narrowest, at the navel.
    Waist,
    /// The hip joint, where a leg turns.
    Hip(Side),
    Knee(Side),
    Ankle(Side),
    /// The tip of the foot, on the ground.
    Toes(Side),
    /// The back of the heel, on the ground.
    Heel(Side),
}

/// A body's shape, standing with its arms a little out from its sides. Heights are
/// from the ground; x is towards its left, y up, z forwards, between its feet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Frame {
    pub height: i32,
    /// Heights of the main levels.
    pub chin: i32,
    pub neck_base: i32,
    pub shoulder: i32,
    pub chest: i32,
    pub waist: i32,
    pub hip: i32,
    pub knee: i32,
    pub ankle: i32,
    /// Half widths across, and half depths front to back.
    pub head: (i32, i32),
    pub neck: i32,
    /// From the middle to the shoulder joint.
    pub shoulder_half: i32,
    pub chest_half: (i32, i32),
    pub waist_half: (i32, i32),
    pub hips_half: (i32, i32),
    /// From the middle to the hip joint.
    pub hip_apart: i32,
    pub upper_arm: i32,
    pub elbow_length: i32,
    pub forearm: i32,
    pub wrist_length: i32,
    pub hand_length: i32,
    pub thigh: i32,
    pub knee_radius: i32,
    pub calf: i32,
    pub ankle_radius: i32,
    pub foot_length: i32,
    /// How far out the arms hang, as so many millimetres out per metre down.
    pub arm_spread: i32,
    /// How the forearms hang from the slightly bent elbows: so many millimetres out, and
    /// forwards, per metre down, so they fall nearer straight down than the upper arms.
    pub forearm_spread: i32,
    pub forearm_forward: i32,
}

/// An ordinary person: about eight heads tall, as artists draw them. Every other frame
/// worked out from `Proportions` is measured against this one.
const HUMAN: Frame = Frame {
    height: 1750,
    chin: 1530,
    neck_base: 1450,
    shoulder: 1405,
    chest: 1260,
    waist: 1080,
    hip: 900,
    knee: 500,
    ankle: 80,
    head: (78, 100),
    neck: 55,
    shoulder_half: 190,
    chest_half: (160, 110),
    waist_half: (135, 95),
    hips_half: (170, 110),
    hip_apart: 90,
    upper_arm: 48,
    elbow_length: 300,
    forearm: 40,
    wrist_length: 260,
    hand_length: 190,
    thigh: 80,
    knee_radius: 55,
    calf: 55,
    ankle_radius: 35,
    foot_length: 260,
    arm_spread: 180,
    forearm_spread: 30,
    forearm_forward: 150,
};

/// Heavier built, with a smaller head for its height.
const GIANT: Frame = Frame {
    chin: 1560,
    neck_base: 1480,
    shoulder: 1425,
    head: (70, 90),
    neck: 75,
    shoulder_half: 230,
    chest_half: (200, 140),
    waist_half: (180, 130),
    hips_half: (200, 135),
    hip_apart: 105,
    upper_arm: 65,
    forearm: 55,
    thigh: 100,
    knee_radius: 70,
    calf: 72,
    ankle_radius: 45,
    arm_spread: 240,
    ..HUMAN
};

impl Frame {
    /// The shape of a kind that stands on two legs: the view of its layout.
    pub fn of(kind: CreatureKind) -> Option<Frame> {
        Layout::of(kind)?.frame()
    }

    /// How tall a kind on two legs usually stands, in millimetres. Its frame is always a
    /// person's height, so this is only for showing how big it is.
    pub fn usual_height(kind: CreatureKind) -> i32 {
        match kind {
            CreatureKind::Giant => 5000,
            _ => Biped::of(kind).map_or(HUMAN.height, |biped| biped.height),
        }
    }

    /// Where a joint is: x towards the body's left, y up, z forwards.
    pub fn joint(&self, joint: Joint) -> [i32; 3] {
        let out = |side: Side, x: i32| side.sign() * x;
        // An upper arm hangs out from its shoulder by its spread; the forearm, from the
        // slightly bent elbow, nearer straight down and a little forwards.
        let elbow = |side: Side| {
            let down = self.elbow_length;
            [
                out(side, self.shoulder_half + down * self.arm_spread / 1000),
                self.shoulder - down,
                0,
            ]
        };
        let below_elbow = |side: Side, down: i32| {
            let [x, y, _] = elbow(side);
            [
                x + out(side, down * self.forearm_spread / 1000),
                y - down,
                down * self.forearm_forward / 1000,
            ]
        };
        match joint {
            Joint::Crown => [0, self.height, 0],
            Joint::Skull => [0, (self.height + self.chin) / 2, 0],
            Joint::Chin => [0, self.chin, self.head.1 / 2],
            Joint::NeckBase => [0, self.neck_base, 0],
            Joint::Shoulder(side) => [out(side, self.shoulder_half), self.shoulder, 0],
            Joint::Elbow(side) => elbow(side),
            Joint::Wrist(side) => below_elbow(side, self.wrist_length),
            Joint::Fingertips(side) => below_elbow(side, self.wrist_length + self.hand_length),
            Joint::Chest => [0, self.chest, self.chest_half.1],
            Joint::Waist => [0, self.waist, self.waist_half.1],
            Joint::Hip(side) => [out(side, self.hip_apart), self.hip, 0],
            Joint::Knee(side) => [out(side, self.hip_apart), self.knee, 0],
            Joint::Ankle(side) => [out(side, self.hip_apart), self.ankle, 0],
            Joint::Toes(side) => [out(side, self.hip_apart), 0, self.foot_length * 3 / 4],
            Joint::Heel(side) => [out(side, self.hip_apart), 0, -self.foot_length / 4],
        }
    }
}

#[cfg(test)]
mod tests;
