//! The trunk, lofted along its spine from the crotch to the neck, at whatever tilt the
//! spine has; and its surface as drawn, for armour to keep clear of.

use bevy::prelude::*;
use shared::body::{Frame, Layout, NONE, RINGS, Ring};

use super::{ROUND, Triangle, loft};

/// Where the spine is, as drawn: its points, which way it runs, and which way is in
/// front of it.
pub(super) struct Axes {
    base: Vec3,
    /// Along the spine, from the hips towards the neck.
    pub(super) along: Vec3,
    /// Out of its front: forwards when upright, down when level.
    pub(super) front: Vec3,
}

impl Axes {
    pub(super) fn of(layout: &Layout) -> Self {
        let pitch = (layout.spine.pitch as f32).to_radians();
        let [x, y, z] = layout.spine.base;
        Axes {
            base: Vec3::new(x as f32, y as f32, z as f32),
            along: Vec3::new(0.0, pitch.cos(), pitch.sin()),
            front: Vec3::new(0.0, -pitch.sin(), pitch.cos()),
        }
    }

    /// The point this far along the spine from the middle of the hips.
    pub(super) fn at(&self, along: f32) -> Vec3 {
        self.base + self.along * along
    }
}

/// The rings worth drawing: a ring halfway between its neighbours (as a body worked
/// out from its stations has) adds nothing to the surface, so it is left out, and the
/// trunk is the same triangles it always was.
fn drawn(rings: &[Ring; RINGS]) -> Vec<Ring> {
    let mut kept = vec![rings[0]];
    for at in 1..RINGS - 1 {
        let (before, ring, after) = (rings[at - 1], rings[at], rings[at + 1]);
        let between = |a: i32, x: i32, b: i32| (a + b - 2 * x).abs() <= 2;
        let straight = between(before.along, ring.along, after.along)
            && between(before.half, ring.half, after.half)
            && between(before.belly, ring.belly, after.belly)
            && between(before.back, ring.back, after.back);
        if !straight {
            kept.push(ring);
        }
    }
    kept.push(rings[RINGS - 1]);
    kept
}

/// The point at the middle of a ring's outline, between its front and its back.
pub(super) fn middle_of(axes: &Axes, ring: &Ring) -> Vec3 {
    axes.at(ring.along as f32) + axes.front * (ring.belly - ring.back) as f32 / 2.0
}

/// Where the neck is drawn from: the middle of the trunk's last ring, which on a body
/// worked out from its stations is the spine's end, and on a measured beast is where
/// its trunk has already begun to climb into the neck.
pub(in crate::item_art) fn neck_root(layout: &Layout) -> Vec3 {
    middle_of(&Axes::of(layout), &layout.spine.rings[RINGS - 1])
}

/// The trunk's skin: an oval round each ring, from its front to its back and as wide
/// as it says.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout) {
    let axes = Axes::of(layout);
    let rings: Vec<Vec<Vec3>> = drawn(&layout.spine.rings)
        .iter()
        .map(|ring| {
            let middle = middle_of(&axes, ring);
            let (across, deep) = (ring.half as f32, (ring.belly + ring.back) as f32 / 2.0);
            (0..ROUND)
                .map(|at| {
                    let angle = at as f32 / ROUND as f32 * std::f32::consts::TAU;
                    middle + Vec3::X * across * angle.cos() + axes.front * deep * angle.sin()
                })
                .collect()
        })
        .collect();
    loft(triangles, &rings);
}

/// The trunk's half widths across and front to back at a height, as it is drawn on an
/// upright frame: what armour on it is offset from.
pub(in crate::item_art) fn trunk_half(frame: &Frame, y: f32) -> (f32, f32) {
    let hip = frame.hip as f32;
    let levels: Vec<(f32, (f32, f32))> = drawn(&Layout::biped(frame, NONE).spine.rings)
        .iter()
        .map(|ring| {
            let f = |value: i32| value as f32;
            (hip + f(ring.along), (f(ring.half), f(ring.belly)))
        })
        .collect();
    let (first, last) = (levels[0], levels[levels.len() - 1]);
    if y <= first.0 {
        return first.1;
    }
    if y >= last.0 {
        return last.1;
    }
    let at = levels
        .windows(2)
        .find(|pair| y <= pair[1].0)
        .expect("between the ends");
    let t = (y - at[0].0) / (at[1].0 - at[0].0);
    (
        at[0].1.0 + (at[1].1.0 - at[0].1.0) * t,
        at[0].1.1 + (at[1].1.1 - at[0].1.1) * t,
    )
}

/// How far in front of the middle (or behind it) the trunk's skin is at this height and
/// this far across: 0 past its side.
pub(in crate::item_art) fn trunk_depth(frame: &Frame, x: f32, y: f32) -> f32 {
    let (across, deep) = trunk_half(frame, y);
    deep * (1.0 - (x / across).powi(2)).max(0.0).sqrt()
}
