//! A body's legs and feet, by what it stands on: a person's heels down, a lizard's or a
//! dog's raised heels, hooves, or a bird's toes.

use bevy::prelude::*;
use shared::body::{Feet, Frame, Joint, Side};

use super::{Triangle, blob, limb, point};

/// One leg and its foot.
pub(super) fn build(triangles: &mut Vec<Triangle>, frame: &Frame, feet: Feet, side: Side) {
    let f = |value: i32| value as f32;
    let joint = |joint: Joint| point(frame.joint(joint));
    let (hip, knee, ankle) = (
        joint(Joint::Hip(side)),
        joint(Joint::Knee(side)),
        joint(Joint::Ankle(side)),
    );
    let (thigh, knee_radius, calf, ankle_radius) = (
        f(frame.thigh),
        f(frame.knee_radius),
        f(frame.calf),
        f(frame.ankle_radius),
    );
    let foot = f(frame.foot_length);
    if feet == Feet::Digitigrade {
        // The knee a little forwards, the heel raised well off the ground behind it, and
        // the body on the balls of its feet.
        let knee = knee + Vec3::Z * foot * 0.35;
        let heel = Vec3::new(ankle.x, ankle.y + (knee.y - ankle.y) * 0.35, -foot * 0.3);
        let ball = Vec3::new(ankle.x, ankle.y * 0.6, foot * 0.25);
        limb(
            triangles,
            &[
                (hip, thigh),
                (knee, knee_radius),
                (heel, ankle_radius * 1.1),
                (ball, ankle_radius),
            ],
        );
        blob(
            triangles,
            Vec3::new(ball.x, ankle.y * 0.3, ball.z + foot * 0.2),
            Vec3::new(ankle_radius * 1.3, ankle.y * 0.3, foot * 0.3),
        );
        return;
    }
    limb(
        triangles,
        &[
            (hip, thigh),
            (knee, knee_radius),
            (knee.lerp(ankle, 0.3), calf),
            (ankle, ankle_radius),
        ],
    );
    let (toes, heel) = (joint(Joint::Toes(side)), joint(Joint::Heel(side)));
    match feet {
        Feet::Hooves => {
            // Widening down to the ground, a little forwards.
            limb(
                triangles,
                &[
                    (ankle, ankle_radius * 1.1),
                    (Vec3::new(ankle.x, 0.0, foot * 0.08), ankle_radius * 1.5),
                ],
            );
        }
        Feet::Talons => {
            // Three toes forwards and one back, thinning to their claws.
            let spread = ankle_radius * 1.2;
            for (across, forwards) in [
                (-spread, foot * 0.6),
                (0.0, foot * 0.7),
                (spread, foot * 0.6),
                (0.0, -foot * 0.3),
            ] {
                let tip = Vec3::new(ankle.x + across, 0.0, forwards);
                limb(
                    triangles,
                    &[(ankle, ankle_radius * 0.6), (tip, ankle_radius * 0.15)],
                );
            }
        }
        _ => {
            blob(
                triangles,
                Vec3::new(toes.x, ankle.y / 2.0, (toes.z + heel.z) / 2.0),
                Vec3::new(ankle_radius * 1.3, ankle.y / 2.0, (toes.z - heel.z) / 2.0),
            );
        }
    }
}
