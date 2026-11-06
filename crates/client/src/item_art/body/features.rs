//! What a body has beyond a person's, drawn on its layout: snouts and beaks, pointed ears,
//! horns, tusks, a tail, and feathers along arms that are wings.

use bevy::prelude::*;
use shared::body::{Horns, Layout, Parts, Role, Side, Snout, Wings};

use super::trunk::Axes;
use super::{Triangle, limb, point};

/// Everything in `parts` but the legs, feet and feathers.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout) {
    let parts = &layout.parts;
    head(triangles, layout, parts);
    if parts.tail > 0 {
        tail(triangles, layout, parts.tail as f32);
    }
}

/// The middle of the head, on top of the neck.
pub(super) fn skull(layout: &Layout) -> Vec3 {
    let axes = Axes::of(layout);
    let chin = axes.at(layout.spine.neck_base as f32) + Vec3::Y * layout.neck.length as f32;
    chin + Vec3::Y * (layout.head.length / 2) as f32
}

/// The feathers of wings, if it has any: drawn apart from the skin, in their own colour.
pub(in crate::item_art) fn wings(layout: &Layout) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    if layout.parts.wings == Wings::Arms {
        for side in Side::BOTH {
            if let Some(arm) = layout.limb(Role::WingArm, side) {
                let path: Vec<Vec3> = layout.joints(arm).into_iter().map(point).collect();
                let reach = arm.bones[0].map_or(0, |bone| bone.span) as f32;
                feathers(&mut triangles, &path, reach, side);
            }
        }
    }
    triangles
}

/// The snout, ears, horns and tusks, placed by the head's middle and half sizes.
fn head(triangles: &mut Vec<Triangle>, layout: &Layout, parts: &Parts) {
    let f = |value: i32| value as f32;
    let middle = skull(layout);
    let (across, up, deep) = (
        f(layout.head.half.0),
        f(layout.head.length) / 2.0,
        f(layout.head.half.1),
    );
    let at = |x: f32, y: f32, z: f32| middle + Vec3::new(x, y, z);
    let length = f(parts.snout_length);
    // From where it leaves the face, forwards and a little down, thinning to its end.
    let snout = match parts.snout {
        Snout::Flat => None,
        Snout::Muzzle => Some((
            at(0.0, -up * 0.35, deep * 0.5),
            -up * 0.1,
            across * 0.55,
            across * 0.45,
        )),
        Snout::Long => Some((
            at(0.0, -up * 0.3, deep * 0.4),
            -up * 0.2,
            across * 0.5,
            across * 0.15,
        )),
        Snout::Beak => Some((
            at(0.0, -up * 0.1, deep * 0.7),
            -up * 0.25,
            across * 0.35,
            2.0,
        )),
    };
    if let Some((from, drop, wide, tip)) = snout.filter(|_| length > 0.0) {
        limb(
            triangles,
            &[(from, wide), (from + Vec3::new(0.0, drop, length), tip)],
        );
    }
    for side in Side::BOTH {
        let out = side.sign() as f32;
        if parts.ears > 0 {
            let reach = f(parts.ears);
            let base = at(out * across * 0.9, up * 0.05, -deep * 0.1);
            limb(
                triangles,
                &[
                    (base, 12.0 + reach * 0.15),
                    (
                        base + Vec3::new(out * reach * 0.9, reach * 0.35, -reach * 0.2),
                        4.0,
                    ),
                ],
            );
        }
        let horn = f(parts.horn_length);
        let thick = (horn * 0.12).max(8.0);
        match parts.horns {
            Horns::None => {}
            Horns::Back => {
                let base = at(out * across * 0.5, up * 0.75, deep * 0.1);
                limb(
                    triangles,
                    &[
                        (base, thick),
                        (
                            base + Vec3::new(out * horn * 0.15, horn * 0.4, -horn * 0.5),
                            thick * 0.6,
                        ),
                        (
                            base + Vec3::new(out * horn * 0.25, horn * 0.45, -horn * 0.95),
                            2.0,
                        ),
                    ],
                );
            }
            Horns::Bull => {
                let base = at(out * across * 0.85, up * 0.6, 0.0);
                limb(
                    triangles,
                    &[
                        (base, thick),
                        (
                            base + Vec3::new(out * horn * 0.55, horn * 0.05, horn * 0.05),
                            thick * 0.7,
                        ),
                        (
                            base + Vec3::new(out * horn * 0.75, horn * 0.45, horn * 0.1),
                            3.0,
                        ),
                    ],
                );
            }
        }
        if parts.tusks > 0 {
            let tusk = f(parts.tusks);
            let base = at(out * across * 0.35, -up * 0.75, deep * 0.75);
            limb(
                triangles,
                &[
                    (base, 6.0),
                    (base + Vec3::new(out * tusk * 0.1, tusk, tusk * 0.2), 2.0),
                ],
            );
        }
    }
}

/// A tail from the base of the back, out behind and curving down, kept off the ground.
fn tail(triangles: &mut Vec<Triangle>, layout: &Layout, length: f32) {
    let f = |value: i32| value as f32;
    let axes = Axes::of(layout);
    let base = axes.at(0.0) - axes.front * f(layout.spine.hips.half.1) * 0.8;
    let thigh = layout
        .limbs()
        .find(|limb| matches!(limb.role, Role::Leg | Role::Hindleg))
        .and_then(|limb| limb.bones[0])
        .map_or(f(layout.spine.hips.half.0) * 0.5, |bone| f(bone.radius));
    let thick = thigh * 0.55;
    let along = |down: f32, back: f32| {
        let p = base + Vec3::new(0.0, -length * down, -length * back);
        Vec3::new(p.x, p.y.max(30.0), p.z)
    };
    limb(
        triangles,
        &[
            (base, thick),
            (along(0.25, 0.35), thick * 0.7),
            (along(0.45, 0.75), thick * 0.4),
            (along(0.5, 1.0), 8.0),
        ],
    );
}

/// Feathers along an arm that is a wing, spread out to the side and back from it, a
/// little down, longest at the hand. Both faces are drawn, as the vane is thin.
fn feathers(triangles: &mut Vec<Triangle>, path: &[Vec3], reach: f32, side: Side) {
    let edge: Vec<(Vec3, Vec3)> = (0..=9)
        .map(|step| {
            let t = step as f32 / 9.0 * 3.0;
            let (at, along) = ((t as usize).min(2), t.fract());
            let on_arm = path[at].lerp(path[at + 1], if t >= 3.0 { 1.0 } else { along });
            let long = reach * (0.6 + 0.6 * t / 3.0);
            let tip =
                on_arm + Vec3::new(side.sign() as f32 * long * 0.7, -long * 0.25, -long * 0.55);
            (on_arm, tip)
        })
        .collect();
    for pair in edge.windows(2) {
        let [(a, a_tip), (b, b_tip)] = [pair[0], pair[1]];
        for [p, q, r] in [[a, b, b_tip], [a, b_tip, a_tip]] {
            triangles.push([p, q, r]);
            triangles.push([p, r, q]);
        }
    }
}
