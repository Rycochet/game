//! A kind's body as a few numbers beside an ordinary person's, which its whole frame is
//! worked out from: a new kind is a row of these rather than a frame of its own. Whole
//! numbers only, as everything on a frame is.

use super::{Frame, HUMAN};

/// How a body differs from an ordinary person's. Everything but `heads` is in
/// thousandths of a person's, so 1000 is the same and 1250 a quarter more.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Proportions {
    /// How many heads tall, in tenths, as artists measure: a person is 80.
    pub heads: i32,
    /// Breadth across the shoulders, chest, waist and hips.
    pub breadth: i32,
    /// Thickness of the neck, arms and legs.
    pub limbs: i32,
    /// Length of the arms and hands.
    pub arms: i32,
    /// Length of the legs: how high the hips are.
    pub legs: i32,
}

/// An ordinary person: the frame every other is measured against.
pub const PERSON: Proportions = Proportions {
    heads: 80,
    breadth: 1000,
    limbs: 1000,
    arms: 1000,
    legs: 1000,
};

/// `value` changed by `thousandths` (1000 leaves it as it is).
fn scaled(value: i32, thousandths: i32) -> i32 {
    value * thousandths / 1000
}

impl Proportions {
    /// The frame these make, at an ordinary person's height. Levels between the hips
    /// and the neck are spread over whatever the head and legs leave of the height.
    pub fn frame(&self) -> Frame {
        let h = HUMAN;
        let head = PERSON.heads * 1000 / self.heads;
        let chin = h.height - scaled(h.height - h.chin, head);
        let neck_base = chin - (h.chin - h.neck_base);
        let hip = scaled(h.hip, self.legs);
        let trunk = |y: i32| hip + (y - h.hip) * (neck_base - hip) / (h.neck_base - h.hip);
        let wide = |v: i32| scaled(v, self.breadth);
        let thick = |v: i32| scaled(v, self.limbs);
        let long = |v: i32| scaled(v, self.arms);
        let pair = |(across, deep): (i32, i32)| (wide(across), wide(deep));
        Frame {
            height: h.height,
            chin,
            neck_base,
            shoulder: trunk(h.shoulder),
            chest: trunk(h.chest),
            waist: trunk(h.waist),
            hip,
            knee: scaled(h.knee, self.legs),
            ankle: scaled(h.ankle, self.legs),
            head: (scaled(h.head.0, head), scaled(h.head.1, head)),
            neck: thick(h.neck),
            shoulder_half: wide(h.shoulder_half),
            chest_half: pair(h.chest_half),
            waist_half: pair(h.waist_half),
            hips_half: pair(h.hips_half),
            hip_apart: wide(h.hip_apart),
            upper_arm: thick(h.upper_arm),
            elbow_length: long(h.elbow_length),
            forearm: thick(h.forearm),
            wrist_length: long(h.wrist_length),
            hand_length: long(h.hand_length),
            thigh: thick(h.thigh),
            knee_radius: thick(h.knee_radius),
            calf: thick(h.calf),
            ankle_radius: thick(h.ankle_radius),
            foot_length: scaled(h.foot_length, self.legs),
            arm_spread: wide(h.arm_spread),
        }
    }
}
