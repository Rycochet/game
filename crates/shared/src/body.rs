//! Where things are on a standing body, in millimetres, for building what is worn on
//! it: every worn thing's model is made in place on one of these, so the same model
//! draws its own picture and, later, a character wearing it. See
//! `docs/context/shared/body.md`.
//!
//! Lengths are at an ordinary person's scale, as blades' are: an item's size goes on
//! top. Whole millimetres only, so both sides can work out weights from them alike.

use crate::creature::CreatureKind;

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
}

/// An ordinary person: about eight heads tall, as artists draw them.
const HUMAN: Frame = Frame {
    height: 1750,
    chin: 1530,
    neck_base: 1450,
    shoulder: 1420,
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
};

/// Heavier built, with a smaller head for its height.
const GIANT: Frame = Frame {
    chin: 1560,
    neck_base: 1480,
    shoulder: 1440,
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

/// A big head on short legs, leaning slightly forwards in life but not here.
const KOBOLD: Frame = Frame {
    chin: 1420,
    neck_base: 1340,
    shoulder: 1310,
    chest: 1170,
    waist: 1010,
    hip: 860,
    knee: 470,
    head: (110, 150),
    neck: 50,
    shoulder_half: 170,
    chest_half: (140, 100),
    waist_half: (120, 90),
    hips_half: (145, 100),
    hip_apart: 80,
    upper_arm: 40,
    elbow_length: 270,
    forearm: 34,
    wrist_length: 230,
    hand_length: 170,
    thigh: 68,
    knee_radius: 48,
    calf: 46,
    foot_length: 290,
    ..HUMAN
};

impl Frame {
    /// The shape of a kind that stands on two legs.
    pub fn of(kind: CreatureKind) -> Option<Frame> {
        match kind {
            CreatureKind::Human | CreatureKind::Beggar => Some(HUMAN),
            CreatureKind::Giant => Some(GIANT),
            CreatureKind::Kobold => Some(KOBOLD),
            CreatureKind::Centaur | CreatureKind::Chicken | CreatureKind::Horse => None,
        }
    }

    /// Where a joint is: x towards the body's left, y up, z forwards.
    pub fn joint(&self, joint: Joint) -> [i32; 3] {
        let out = |side: Side, x: i32| side.sign() * x;
        // An arm hangs out from its shoulder by its spread.
        let arm = |side: Side, down: i32, y: i32| {
            [
                out(side, self.shoulder_half + down * self.arm_spread / 1000),
                y,
                0,
            ]
        };
        match joint {
            Joint::Crown => [0, self.height, 0],
            Joint::Skull => [0, (self.height + self.chin) / 2, 0],
            Joint::Chin => [0, self.chin, self.head.1 / 2],
            Joint::NeckBase => [0, self.neck_base, 0],
            Joint::Shoulder(side) => [out(side, self.shoulder_half), self.shoulder, 0],
            Joint::Elbow(side) => arm(side, self.elbow_length, self.shoulder - self.elbow_length),
            Joint::Wrist(side) => {
                let down = self.elbow_length + self.wrist_length;
                arm(side, down, self.shoulder - down)
            }
            Joint::Fingertips(side) => {
                let down = self.elbow_length + self.wrist_length + self.hand_length;
                arm(side, down, self.shoulder - down)
            }
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
mod tests {
    use super::*;

    fn frames() -> impl Iterator<Item = Frame> {
        CreatureKind::ALL.into_iter().filter_map(Frame::of)
    }

    #[test]
    fn every_kind_on_two_legs_has_a_frame() {
        for kind in CreatureKind::ALL {
            let def = kind.def();
            let biped = def.part(crate::creature::Location::Legs).is_some()
                && def.part(crate::creature::Location::Arms).is_some();
            assert_eq!(Frame::of(kind).is_some(), biped, "{kind:?}");
        }
    }

    #[test]
    fn levels_go_down_in_order() {
        for frame in frames() {
            let levels = [
                frame.height,
                frame.chin,
                frame.neck_base,
                frame.shoulder,
                frame.chest,
                frame.waist,
                frame.hip,
                frame.knee,
                frame.ankle,
                0,
            ];
            assert!(levels.windows(2).all(|pair| pair[0] > pair[1]), "{frame:?}");
            // The head is as tall as it is said to be deep.
            assert!(frame.height - frame.chin >= frame.head.1 * 2);
        }
    }

    #[test]
    fn hands_hang_beside_the_thighs_and_sides_mirror() {
        for frame in frames() {
            let tips = frame.joint(Joint::Fingertips(Side::Left));
            assert!(tips[1] > frame.knee && tips[1] < frame.hip, "{frame:?}");
            assert!(tips[0] > frame.hips_half.0, "{frame:?}");
            let right = frame.joint(Joint::Elbow(Side::Right));
            let left = frame.joint(Joint::Elbow(Side::Left));
            assert_eq!(right, [-left[0], left[1], left[2]]);
        }
    }
}
