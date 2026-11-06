//! Two frames put together number by number, for a taur's upper body as it blends
//! or grows.

use super::super::Frame;

/// A frame whose every number is `f` of the two frames' same numbers.
pub(in crate::body) fn zip(a: &Frame, b: &Frame, f: impl Fn(i32, i32) -> i32) -> Frame {
    let pair = |x: (i32, i32), y: (i32, i32)| (f(x.0, y.0), f(x.1, y.1));
    Frame {
        height: f(a.height, b.height),
        chin: f(a.chin, b.chin),
        neck_base: f(a.neck_base, b.neck_base),
        shoulder: f(a.shoulder, b.shoulder),
        chest: f(a.chest, b.chest),
        waist: f(a.waist, b.waist),
        hip: f(a.hip, b.hip),
        knee: f(a.knee, b.knee),
        ankle: f(a.ankle, b.ankle),
        head: pair(a.head, b.head),
        neck: f(a.neck, b.neck),
        shoulder_half: f(a.shoulder_half, b.shoulder_half),
        chest_half: pair(a.chest_half, b.chest_half),
        waist_half: pair(a.waist_half, b.waist_half),
        hips_half: pair(a.hips_half, b.hips_half),
        hip_apart: f(a.hip_apart, b.hip_apart),
        upper_arm: f(a.upper_arm, b.upper_arm),
        elbow_length: f(a.elbow_length, b.elbow_length),
        forearm: f(a.forearm, b.forearm),
        wrist_length: f(a.wrist_length, b.wrist_length),
        hand_length: f(a.hand_length, b.hand_length),
        thigh: f(a.thigh, b.thigh),
        knee_radius: f(a.knee_radius, b.knee_radius),
        calf: f(a.calf, b.calf),
        ankle_radius: f(a.ankle_radius, b.ankle_radius),
        foot_length: f(a.foot_length, b.foot_length),
        arm_spread: f(a.arm_spread, b.arm_spread),
        forearm_spread: f(a.forearm_spread, b.forearm_spread),
        forearm_forward: f(a.forearm_forward, b.forearm_forward),
    }
}

/// A frame this many thousandths of its size: its lengths, not its ways (millimetres
/// per metre), which stay as they are at any size.
pub(in crate::body) fn scaled(frame: &Frame, share: i32) -> Frame {
    Frame {
        arm_spread: frame.arm_spread,
        forearm_spread: frame.forearm_spread,
        forearm_forward: frame.forearm_forward,
        ..zip(frame, frame, |x, _| x * share / super::WHOLE)
    }
}
