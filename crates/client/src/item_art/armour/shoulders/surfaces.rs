//! The surfaces pieces are laid on: over the cop, the arm, a wing or a guard.

use bevy::prelude::*;
use shared::armour::shoulders::GuardShape;
use shared::body::Frame;
use std::f32::consts::PI;

use super::{Arm, HANGING_CLEAR, Look, WING_CLEAR, edge_drop, flute};
use crate::item_art::body::trunk_depth;

/// The cop: a dome over the joint, from `start` to `end` round the arm.
pub(super) fn cop_surface<'a>(
    arm: &'a Arm,
    look: Look,
    (start, end): (f32, f32),
    (across, up): (f32, f32),
    to: f32,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    move |u: f32, v: f32| {
        let angle = start + (end - start) * u;
        let over = 0.12 + (to - 0.12) * v;
        // A leaf peak at the top, straight out.
        let peak = look.leaf * angle.cos().max(0.0).powi(8) * (1.0 - v);
        let grow = flute(look, angle) * over.sin();
        let (point, normal) = arm.dome(angle, over, across + grow, up * look.squat + peak);
        (point, normal)
    }
}

/// A band round the arm from `top` to `bottom` down it, between two angles.
pub(super) fn arm_surface<'a>(
    arm: &'a Arm,
    look: Look,
    (start, end): (f32, f32),
    (top, bottom): (f32, f32),
    radius: f32,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    move |u: f32, v: f32| {
        let angle = start + (end - start) * u;
        let along = top + (bottom - top) * v + edge_drop(look, angle) * v.powi(4);
        arm.round(along, angle, radius + flute(look, angle))
    }
}

/// The front or back of the trunk, well clear of it, from `inner` across to `outer`
/// and down from `top` by `drop`, rounded at the bottom.
pub(super) fn wing_surface<'a>(
    frame: &'a Frame,
    sign: f32,
    front: bool,
    (outer, inner): (f32, f32),
    top: f32,
    drop: f32,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    move |u: f32, v: f32| {
        let x = outer + (inner - outer) * u;
        let y = top - v * drop * (0.55 + 0.45 * (PI * u).sin());
        let deep = trunk_depth(frame, x, y, front) + WING_CLEAR;
        let z = if front { deep } else { -deep };
        let normal = Vec3::new(sign * x * 0.004, 0.25, z.signum()).normalize();
        (Vec3::new(sign * x, y, z), normal)
    }
}

/// The trunk's top between the neck and the shoulder, as the body is drawn.
pub(super) fn shoulder_top(frame: &Frame, x: f32) -> f32 {
    let neck = frame.neck as f32 * 1.6;
    let joint = frame.shoulder_half as f32;
    let slope = frame.neck_base as f32
        - (frame.neck_base - frame.shoulder) as f32 * ((x - neck) / (joint - neck)).clamp(0.0, 1.0);
    let ball = frame.upper_arm as f32 * 1.15;
    let over_ball = frame.shoulder as f32 + (ball * ball - (x - joint).powi(2)).max(0.0).sqrt();
    slope.max(over_ball)
}

/// A panel hanging over the upper arm, its middle `HANGING_CLEAR` out from it.
pub(super) fn guard_surface<'a>(
    arm: &'a Arm,
    frame: &'a Frame,
    (wide, high): (f32, f32),
    shape: GuardShape,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    // Flatter the wider it is.
    let near = arm.radius + HANGING_CLEAR;
    let bend = (wide / 1.6).max(near);
    let scale = frame.elbow_length as f32 / 300.0;
    move |u: f32, v: f32| {
        let width = wide
            * match shape {
                GuardShape::Straight => 1.0,
                GuardShape::Tapered => 1.0 - 0.3 * v,
                GuardShape::Flared => 1.0 + 0.2 * v,
            };
        let angle = (u - 0.5) * width / bend;
        let along = -45.0 * scale + v * high;
        let axis = arm.joint + arm.down * along - arm.out * (bend - near);
        let out = arm.out * angle.cos() + arm.forward * angle.sin();
        (axis + out * bend, out)
    }
}
