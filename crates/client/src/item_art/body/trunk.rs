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

/// The ring this far along: between the two either side of it, or the end one.
pub(super) fn ring_at(rings: &[Ring], along: f32) -> Ring {
    let after = rings.iter().position(|ring| ring.along as f32 >= along);
    let (before, after) = match after {
        None => return rings[rings.len() - 1],
        Some(0) => return rings[0],
        Some(at) => (rings[at - 1], rings[at]),
    };
    let share = (along - before.along as f32) / (after.along - before.along).max(1) as f32;
    let mix = |a: i32, b: i32| a + ((b - a) as f32 * share).round() as i32;
    Ring {
        along: mix(before.along, after.along),
        half: mix(before.half, after.half),
        belly: mix(before.belly, after.belly),
        back: mix(before.back, after.back),
    }
}

/// How far a point of the trunk is towards its front from the middle of its ring
/// there, in the ring's half depths: 1 at a beast's belly, -1 at its back.
pub(super) fn underside(layout: &Layout, at: Vec3) -> f32 {
    let axes = Axes::of(layout);
    let ring = ring_at(&layout.spine.rings, (at - axes.base).dot(axes.along));
    let deep = ((ring.belly + ring.back) as f32 / 2.0).max(1.0);
    (at - middle_of(&axes, &ring)).dot(axes.front) / deep
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

/// The trunk's skin round each ring, as wide as it says, from its front to its back.
/// Upright, it is half an oval in front of the spine and half behind, meeting at the
/// sides, as a person's back is flatter than their chest; level (a beast's), one oval
/// round the middle of the ring; leaning, between the two.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout) {
    let axes = Axes::of(layout);
    let level = layout.spine.pitch.clamp(0, 90) as f32 / 90.0;
    let rings: Vec<Vec<Vec3>> = drawn(&layout.spine.rings)
        .iter()
        .map(|ring| {
            let spine = axes.at(ring.along as f32);
            let middle = middle_of(&axes, ring);
            let (across, deep) = (ring.half as f32, (ring.belly + ring.back) as f32 / 2.0);
            (0..ROUND)
                .map(|at| {
                    let angle = at as f32 / ROUND as f32 * std::f32::consts::TAU;
                    let side = Vec3::X * across * angle.cos();
                    let oval = middle + side + axes.front * deep * angle.sin();
                    let halves = spine + side + axes.front * Girth::of(ring).z(angle.sin());
                    halves.lerp(oval, level)
                })
                .collect()
        })
        .collect();
    loft(triangles, &rings);
}

/// The trunk's size at one height of an upright frame, in millimetres from the spine:
/// half its width across, and how far its front and its back are from the spine. Round
/// it, the skin is half an oval in front and half an oval behind, as it is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::item_art) struct Girth {
    pub across: f32,
    pub front: f32,
    pub back: f32,
}

impl Girth {
    fn of(ring: &Ring) -> Self {
        Girth {
            across: ring.half as f32,
            front: ring.belly as f32,
            back: ring.back as f32,
        }
    }

    /// Its front, or its back.
    pub fn deep(&self, front: bool) -> f32 {
        if front { self.front } else { self.back }
    }

    /// How far forwards the skin is where it is `sin` of the way round towards the
    /// front (negative: behind).
    pub fn z(&self, sin: f32) -> f32 {
        self.deep(sin >= 0.0) * sin
    }

    /// The same shape `out` millimetres further out all round.
    pub fn grown(&self, out: f32) -> Self {
        Girth {
            across: self.across + out,
            front: self.front + out,
            back: self.back + out,
        }
    }

    /// A point on the shape at this angle round from the body's left, through its front
    /// (a quarter turn), at height `y`, and the way out from it there.
    pub fn at(&self, angle: f32, y: f32) -> (Vec3, Vec3) {
        let (sin, cos) = angle.sin_cos();
        let deep = self.deep(sin >= 0.0);
        let point = Vec3::new(self.across * cos, y, deep * sin);
        let normal = Vec3::new(cos / self.across, 0.0, sin / deep).normalize_or(Vec3::Z);
        (point, normal)
    }
}

/// The trunk's girth at a height, as it is drawn on an upright frame: what armour on it
/// is offset from. Below the crotch and above the neck's base it carries straight on.
pub(in crate::item_art) fn trunk_half(frame: &Frame, y: f32) -> Girth {
    let hip = frame.hip as f32;
    let levels: Vec<(f32, Girth)> = drawn(&Layout::biped(frame, NONE).spine.rings)
        .iter()
        .map(|ring| (hip + ring.along as f32, Girth::of(ring)))
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
    let (a, b) = (at[0].1, at[1].1);
    let mix = |p: f32, q: f32| p + (q - p) * t;
    Girth {
        across: mix(a.across, b.across),
        front: mix(a.front, b.front),
        back: mix(a.back, b.back),
    }
}

/// How far in front of the spine (`front`) or behind it the trunk's skin is at this
/// height and this far across: 0 past its side.
pub(in crate::item_art) fn trunk_depth(frame: &Frame, x: f32, y: f32, front: bool) -> f32 {
    let girth = trunk_half(frame, y);
    girth.deep(front) * (1.0 - (x / girth.across).powi(2)).max(0.0).sqrt()
}
