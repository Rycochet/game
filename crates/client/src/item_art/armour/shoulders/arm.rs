//! `Arm`: where the upper arm is, which shoulder pieces are built around.

use bevy::prelude::*;
use shared::body::{Frame, Joint, Side};

/// An arm's own directions at the shoulder: down the upper arm, out from the body
/// square to it, and forwards.
pub(super) struct Arm {
    pub(super) side: Side,
    pub(super) sign: f32,
    pub(super) joint: Vec3,
    pub(super) down: Vec3,
    pub(super) out: Vec3,
    pub(super) forward: Vec3,
    /// The upper arm's radius where the bare body is widest at the shoulder.
    pub(super) radius: f32,
    /// How far down the arm the shoulder pieces reach.
    pub(super) reach: f32,
}

impl Arm {
    pub(super) fn of(frame: &Frame, side: Side) -> Self {
        let point = |mm: [i32; 3]| Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32);
        let joint = point(frame.joint(Joint::Shoulder(side)));
        let down = (point(frame.joint(Joint::Elbow(side))) - joint).normalize();
        let sideways = Vec3::X * side.sign() as f32;
        let out = (sideways - down * sideways.dot(down)).normalize();
        Self {
            side,
            sign: side.sign() as f32,
            joint,
            down,
            out,
            forward: down.cross(out).normalize() * down.cross(out).z.signum(),
            radius: frame.upper_arm as f32 * 1.15,
            reach: frame.elbow_length as f32 * 130.0 / 300.0,
        }
    }

    /// Round the arm: `along` down it from the joint, `angle` from straight out towards
    /// the front.
    pub(super) fn round(&self, along: f32, angle: f32, radius: f32) -> (Vec3, Vec3) {
        let out = self.out * angle.cos() + self.forward * angle.sin();
        (self.joint + self.down * along + out * radius, out)
    }

    /// Over the shoulder as a dome: `over` from its top (0) down to where it meets the
    /// arm (π/2), half widths across and up.
    pub(super) fn dome(&self, angle: f32, over: f32, across: f32, up: f32) -> (Vec3, Vec3) {
        let out = self.out * angle.cos() + self.forward * angle.sin();
        let point = self.joint - self.down * (up * over.cos()) + out * (across * over.sin());
        let normal = (-self.down * over.cos() / up + out * over.sin() / across).normalize();
        (point, normal)
    }
}
