//! An animal's measured head (`shared::body::HeadShape`) and the neck it sits on: the
//! neck lofted from the trunk's last ring up to the head, so it carries on from the
//! trunk with no step; the head lofted through its rings from the back of the skull
//! to the nose; ears by their shape, horns, and a hen's comb and wattles, placed on
//! the outline.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::body::{Ears, HEAD_RINGS, HeadShape, Horns, Layout, RINGS, Ring, Side};

use super::trunk::{self, Axes};
use super::{ROUND, Triangle, blob, limb, loft, point};

/// The head's own directions, from where it sits on the neck.
pub(in crate::item_art) struct HeadAxes {
    origin: Vec3,
    along: Vec3,
    up: Vec3,
}

impl HeadAxes {
    pub(in crate::item_art) fn of(layout: &Layout, shape: &HeadShape) -> Self {
        let pitch = (shape.pitch as f32).to_radians();
        HeadAxes {
            origin: point(layout.chin()),
            along: Vec3::new(0.0, -pitch.sin(), pitch.cos()),
            up: Vec3::new(0.0, pitch.cos(), pitch.sin()),
        }
    }

    /// The point this far along the head's line, this far up from it and across.
    pub(in crate::item_art) fn at(&self, along: f32, up: f32, across: f32) -> Vec3 {
        self.origin + self.along * along + self.up * up + Vec3::X * across
    }

    /// How far a point is along the head's line, up from it and across.
    pub(super) fn local(&self, at: Vec3) -> (f32, f32, f32) {
        let from = at - self.origin;
        (from.dot(self.along), from.dot(self.up), from.x)
    }

    /// The middle of a ring's outline.
    fn middle(&self, ring: &Ring) -> Vec3 {
        self.at(
            ring.along as f32,
            (ring.back - ring.belly) as f32 / 2.0,
            0.0,
        )
    }
}

/// The top of the skull: the ring that reaches highest in the head's back half.
pub(in crate::item_art) fn crown(shape: &HeadShape) -> Ring {
    shape.rings[..HEAD_RINGS / 2]
        .iter()
        .copied()
        .max_by_key(|ring| ring.back)
        .unwrap_or_default()
}

/// The head, with what is on it (the neck is drawn first, by `neck` for an animal).
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout, shape: &HeadShape) {
    let axes = HeadAxes::of(layout, shape);
    let rings: Vec<Vec<Vec3>> = shape
        .rings
        .iter()
        .map(|ring| {
            let middle = axes.middle(ring);
            let (across, deep) = (ring.half as f32, (ring.belly + ring.back) as f32 / 2.0);
            oval(middle, Vec3::X * across, axes.up * deep)
        })
        .collect();
    loft(triangles, &rings);
    ears(triangles, layout, shape, &axes);
    horns(triangles, layout, shape, &axes);
    if layout.parts.comb > 0 {
        comb(triangles, layout.parts.comb as f32, shape, &axes);
    }
}

/// Points round an oval with these half axes.
fn oval(middle: Vec3, across: Vec3, deep: Vec3) -> Vec<Vec3> {
    (0..ROUND)
        .map(|at| {
            let angle = at as f32 / ROUND as f32 * TAU;
            middle + across * angle.cos() + deep * angle.sin()
        })
        .collect()
}

/// From a little inside the trunk's last ring, as wide and deep as it, narrowing up to
/// the head as thick as the neck says, a little deeper than wide.
pub(super) fn neck(triangles: &mut Vec<Triangle>, layout: &Layout) {
    let rings = neck_rings(layout).map(|(middle, across, under)| oval(middle, across, under));
    loft(triangles, &rings);
}

/// The neck's three rings, from the trunk's last to the head: each its middle, its
/// half width across and its half depth out of its underside.
pub(super) fn neck_rings(layout: &Layout) -> [(Vec3, Vec3, Vec3); 3] {
    let axes = Axes::of(layout);
    let last = layout.spine.rings[RINGS - 1];
    let root = trunk::neck_root(layout);
    let chin = point(layout.chin());
    let up_neck = (chin - root).normalize_or(Vec3::Y);
    // Out of the neck's underside, square to it: down when it reaches forwards.
    let under = Vec3::X.cross(up_neck).normalize_or(Vec3::NEG_Y);
    let top = layout.neck.radius as f32;
    let (across, deep) = (last.half as f32, (last.belly + last.back) as f32 / 2.0);
    [
        (
            root - axes.along * 20.0,
            Vec3::X * across * 0.9,
            axes.front * deep * 0.9,
        ),
        (
            root.lerp(chin, 0.4),
            Vec3::X * (across * 0.45 + top * 0.55),
            under * (deep * 0.45 + top * 0.7),
        ),
        (chin + up_neck * 20.0, Vec3::X * top, under * top * 1.15),
    ]
}

/// Ears by their shape, standing on the skull's top or out from its sides.
fn ears(triangles: &mut Vec<Triangle>, layout: &Layout, shape: &HeadShape, axes: &HeadAxes) {
    let reach = layout.parts.ears as f32;
    if reach <= 0.0 {
        return;
    }
    let skull = crown(shape);
    let (along, top, half) = (skull.along as f32, skull.back as f32, skull.half as f32);
    for side in Side::BOTH {
        let out = side.sign() as f32;
        let path = match layout.parts.ear_shape {
            Ears::Upright => {
                let base = axes.at(along, top * 0.8, out * half * 0.55);
                vec![
                    (base, reach * 0.3),
                    (
                        base + Vec3::new(out * reach * 0.1, reach * 0.5, -reach * 0.05),
                        reach * 0.2,
                    ),
                    (
                        base + Vec3::new(out * reach * 0.2, reach, -reach * 0.12),
                        3.0,
                    ),
                ]
            }
            Ears::Sideways => {
                // Broad and rounded at the end, drooping a little.
                let base = axes.at(along, top * 0.5, out * half * 0.85);
                vec![
                    (base, reach * 0.12),
                    (
                        base + Vec3::new(out * reach * 0.45, -reach * 0.12, 0.0),
                        reach * 0.28,
                    ),
                    (
                        base + Vec3::new(out * reach * 0.85, -reach * 0.3, -reach * 0.05),
                        reach * 0.18,
                    ),
                    (
                        base + Vec3::new(out * reach, -reach * 0.38, -reach * 0.07),
                        4.0,
                    ),
                ]
            }
            Ears::Dropped => {
                let base = axes.at(along, top * 0.7, out * half * 0.85);
                vec![
                    (base, reach * 0.18),
                    (
                        base + Vec3::new(out * reach * 0.15, -reach * 0.45, 0.0),
                        reach * 0.24,
                    ),
                    (
                        base + Vec3::new(out * reach * 0.2, -reach, 0.0),
                        reach * 0.12,
                    ),
                ]
            }
            Ears::Swept => {
                let base = axes.at(along, top * 0.5, out * half * 0.9);
                vec![
                    (base, 12.0 + reach * 0.15),
                    (
                        base + Vec3::new(out * reach * 0.9, reach * 0.35, -reach * 0.2),
                        4.0,
                    ),
                ]
            }
        };
        limb(triangles, &path);
    }
}

/// Horns from the top of the skull, outside the ears.
fn horns(triangles: &mut Vec<Triangle>, layout: &Layout, shape: &HeadShape, axes: &HeadAxes) {
    let horn = layout.parts.horn_length as f32;
    let skull = crown(shape);
    let (along, top, half) = (skull.along as f32, skull.back as f32, skull.half as f32);
    let thick = (horn * 0.12).max(8.0);
    for side in Side::BOTH {
        let out = side.sign() as f32;
        let base = axes.at(along, top * 0.85, out * half * 0.7);
        let path = match layout.parts.horns {
            Horns::None => continue,
            Horns::Bull => [
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
            Horns::Back => [
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
        };
        limb(triangles, &path);
    }
}

/// A hen's comb, a row of lobes along the top of the skull, and two wattles hanging
/// under the beak.
fn comb(triangles: &mut Vec<Triangle>, tall: f32, shape: &HeadShape, axes: &HeadAxes) {
    let rings = &shape.rings;
    for (at, lobe) in [(2, 0.8), (3, 1.0), (4, 0.75)] {
        let ring = rings[at];
        let top = axes.at(
            ring.along as f32,
            ring.back as f32 + tall * 0.35 * lobe,
            0.0,
        );
        blob(
            triangles,
            top,
            Vec3::new(3.0, tall * 0.5 * lobe, tall * 0.3),
        );
    }
    let under = rings[HEAD_RINGS - 3];
    for side in Side::BOTH {
        let across = side.sign() as f32 * under.half as f32 * 0.4;
        let hang = axes.at(
            under.along as f32,
            -(under.belly as f32) - tall * 0.3,
            across,
        );
        blob(
            triangles,
            hang,
            Vec3::new(3.0, tall * 0.35 * (PI / 4.0), tall * 0.2),
        );
    }
}

#[cfg(test)]
mod tests;
