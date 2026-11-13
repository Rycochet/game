//! A `Frame` is the view of a layout standing upright on two legs with two arms: every
//! worn thing is built on a frame, so turning one into a layout and back must give the
//! same frame (tested for every kind).

use super::layout::{Bone, Girdle, Head, Layout, Limb, MAX_LIMBS, Neck, Role, Spine, Station};
use super::rings::{RINGS, Ring, from_stations};
use super::{Frame, Parts, Side};

/// A bone going this way, `span` far, this thick along it and at its end.
fn bone(span: i32, way: [i32; 3], radius: i32, end_radius: i32) -> Option<Bone> {
    Some(Bone {
        span,
        way,
        radius,
        end_radius,
    })
}

impl Layout {
    /// The layout of a body standing on two legs, from its frame. Arms are wings if
    /// its parts say so.
    pub fn biped(frame: &Frame, parts: Parts) -> Layout {
        let f = frame;
        let from_hip = |height: i32| height - f.hip;
        let arm_role = match parts.wings {
            super::Wings::Arms => Role::WingArm,
            super::Wings::None | super::Wings::Folded => Role::Arm,
        };
        // The forearm and hand fall nearer straight down and a little forwards.
        let forearm_way = [f.forearm_spread, -1000, f.forearm_forward];
        let arm = |side: Side| Limb {
            role: arm_role,
            girdle: Girdle::Shoulders,
            side,
            apart: f.shoulder_half,
            bones: [
                bone(
                    f.elbow_length,
                    [f.arm_spread, -1000, 0],
                    f.upper_arm,
                    f.forearm,
                ),
                bone(f.wrist_length, forearm_way, f.forearm, f.forearm),
                bone(f.hand_length, forearm_way, f.forearm, 0),
                None,
            ],
            foot: 0,
        };
        let down = [0, -1000, 0];
        let leg = |side: Side| Limb {
            role: Role::Leg,
            girdle: Girdle::Hips,
            side,
            apart: f.hip_apart,
            bones: [
                bone(f.hip - f.knee, down, f.thigh, f.knee_radius),
                bone(f.knee - f.ankle, down, f.calf, f.ankle_radius),
                bone(f.ankle, down, f.ankle_radius, f.ankle_radius),
                None,
            ],
            foot: f.foot_length,
        };
        let mut limbs = [None; MAX_LIMBS];
        for (slot, limb) in limbs.iter_mut().zip([
            arm(Side::Left),
            arm(Side::Right),
            leg(Side::Left),
            leg(Side::Right),
        ]) {
            *slot = Some(limb);
        }
        let mut layout = Layout {
            spine: Spine {
                base: [0, f.hip, 0],
                pitch: 0,
                hips: Station {
                    along: 0,
                    half: f.hips_half,
                },
                waist: Station {
                    along: from_hip(f.waist),
                    half: f.waist_half,
                },
                chest: Station {
                    along: from_hip(f.chest),
                    half: f.chest_half,
                },
                shoulders: from_hip(f.shoulder),
                neck_base: from_hip(f.neck_base),
                rings: [Ring::default(); RINGS],
            },
            neck: Neck {
                length: f.chin - f.neck_base,
                radius: f.neck,
                pitch: 0,
            },
            head: Head {
                length: f.height - f.chin,
                half: f.head,
                shape: None,
            },
            limbs,
            parts,
            upper: None,
        };
        layout.spine.rings = from_stations(&layout);
        layout
    }

    /// Its frame, if it stands upright on two legs with two arms (or wing-arms) and its
    /// neck upright.
    pub fn frame(&self) -> Option<Frame> {
        if self.spine.pitch != 0 || self.neck.pitch != 0 || self.upper.is_some() {
            return None;
        }
        let arm = self
            .limb(Role::Arm, Side::Left)
            .or_else(|| self.limb(Role::WingArm, Side::Left))?;
        let leg = self.limb(Role::Leg, Side::Left)?;
        let [Some(upper), Some(fore), Some(hand), None] = arm.bones else {
            return None;
        };
        let [Some(thigh), Some(shin), Some(drop), None] = leg.bones else {
            return None;
        };
        let s = &self.spine;
        let hip = s.base[1];
        let chin = hip + s.neck_base + self.neck.length;
        Some(Frame {
            height: chin + self.head.length,
            chin,
            neck_base: hip + s.neck_base,
            shoulder: hip + s.shoulders,
            chest: hip + s.chest.along,
            waist: hip + s.waist.along,
            hip,
            knee: hip - thigh.span,
            ankle: drop.span,
            head: self.head.half,
            neck: self.neck.radius,
            shoulder_half: arm.apart,
            chest_half: s.chest.half,
            waist_half: s.waist.half,
            hips_half: s.hips.half,
            hip_apart: leg.apart,
            upper_arm: upper.radius,
            elbow_length: upper.span,
            forearm: fore.radius,
            wrist_length: fore.span,
            hand_length: hand.span,
            thigh: thigh.radius,
            knee_radius: thigh.end_radius,
            calf: shin.radius,
            ankle_radius: shin.end_radius,
            foot_length: leg.foot,
            arm_spread: upper.way[0],
            forearm_spread: fore.way[0],
            forearm_forward: fore.way[2],
        })
    }
}
