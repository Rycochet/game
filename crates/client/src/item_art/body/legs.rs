//! A body's legs and feet, by what it stands on: a person's heels down, a lizard's or a
//! dog's raised heels, hooves, or a bird's toes.

use bevy::prelude::*;
use shared::body::{Feet, Layout, Limb};

use super::{Triangle, blob, limb, point};

/// One leg and its foot: its joints are the hip, the knee, the ankle and the ground
/// below it.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout, leg: &Limb, feet: Feet) {
    let joints: Vec<Vec3> = layout.joints(leg).into_iter().map(point).collect();
    if leg.bones.iter().flatten().count() != 3 {
        beast(triangles, leg, &joints, feet);
        return;
    }
    let [Some(upper), Some(lower), ..] = leg.bones else {
        return;
    };
    let (hip, knee, ankle) = (joints[0], joints[1], joints[joints.len() - 2]);
    let (thigh, knee_radius, calf, ankle_radius) = (
        upper.radius as f32,
        upper.end_radius as f32,
        lower.radius as f32,
        lower.end_radius as f32,
    );
    let foot = leg.foot as f32;
    if feet == Feet::Digitigrade {
        // The knee a little forwards, the heel raised well off the ground behind it, and
        // the body on the balls of its feet.
        let knee = knee + Vec3::Z * foot * 0.35;
        let heel = Vec3::new(
            ankle.x,
            ankle.y + (knee.y - ankle.y) * 0.35,
            ankle.z - foot * 0.3,
        );
        let ball = Vec3::new(ankle.x, ankle.y * 0.6, ankle.z + foot * 0.25);
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
    // The foot along the ground below the ankle, three quarters of it in front.
    let (toes, heel) = (
        Vec3::new(ankle.x, 0.0, ankle.z + (foot * 0.75).floor()),
        Vec3::new(ankle.x, 0.0, ankle.z - (foot / 4.0).floor()),
    );
    match feet {
        Feet::Hooves => {
            // Widening down to the ground, a little forwards.
            limb(
                triangles,
                &[
                    (ankle, ankle_radius * 1.1),
                    (
                        Vec3::new(ankle.x, 0.0, ankle.z + foot * 0.08),
                        ankle_radius * 1.5,
                    ),
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
                let tip = Vec3::new(ankle.x + across, 0.0, ankle.z + forwards);
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

/// A beast's leg, bent at every joint down to its last above the ground, then a hoof
/// widening to the ground or a paw on it.
fn beast(triangles: &mut Vec<Triangle>, leg: &Limb, joints: &[Vec3], feet: Feet) {
    let bones: Vec<_> = leg.bones.iter().flatten().collect();
    let Some(first) = bones.first() else {
        return;
    };
    let mut path = vec![(joints[0], first.radius as f32)];
    for (at, bone) in bones.iter().enumerate().take(bones.len() - 1) {
        path.push((joints[at + 1], bone.end_radius as f32));
    }
    limb(triangles, &path);
    let (ankle, radius) = path[path.len() - 1];
    let ground = joints[joints.len() - 1];
    let foot = leg.foot as f32;
    if feet == Feet::Hooves {
        limb(
            triangles,
            &[
                (ankle, radius),
                (ground + Vec3::Z * foot * 0.3, radius * 1.4),
            ],
        );
    } else {
        limb(triangles, &[(ankle, radius), (ground, radius * 0.9)]);
        blob(
            triangles,
            ground + Vec3::new(0.0, radius, foot * 0.25),
            Vec3::new(radius * 1.3, radius, foot * 0.5),
        );
    }
}
