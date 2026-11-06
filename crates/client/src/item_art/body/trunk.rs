//! The trunk, lofted along its spine from the crotch to the neck, at whatever tilt the
//! spine has; and its surface as drawn, for armour to keep clear of.

use bevy::prelude::*;
use shared::body::{Frame, Layout, NONE, Role};

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

/// The thigh where it joins the trunk: the crotch is this far below the hips.
fn thigh(layout: &Layout) -> f32 {
    layout
        .limbs()
        .find(|limb| matches!(limb.role, Role::Leg | Role::Hindleg))
        .and_then(|limb| limb.bones[0])
        .map_or(layout.spine.hips.half.1 as f32 * 0.5, |bone| {
            bone.radius as f32
        })
}

/// The trunk's levels from the crotch up to the neck: each how far along the spine, with
/// its half widths across and front to back.
pub(super) fn levels(layout: &Layout) -> [(f32, (f32, f32)); 6] {
    let f = |value: i32| value as f32;
    let half = |(across, deep): (i32, i32)| (f(across), f(deep));
    let s = &layout.spine;
    // The shoulders as wide as the arms (or forelegs) hang apart.
    let shoulders = layout
        .limbs()
        .find(|limb| matches!(limb.role, Role::Arm | Role::WingArm | Role::Foreleg))
        .map_or(f(s.chest.half.0), |limb| f(limb.apart));
    let neck = f(layout.neck.radius);
    [
        (-thigh(layout), half(s.hips.half)),
        (0.0, half(s.hips.half)),
        (f(s.waist.along), half(s.waist.half)),
        (f(s.chest.along), half(s.chest.half)),
        (f(s.shoulders), (shoulders, f(s.chest.half.1) * 0.7)),
        (f(s.neck_base), (neck * 1.6, neck * 1.2)),
    ]
}

/// The trunk's skin.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout) {
    let axes = Axes::of(layout);
    let rings: Vec<Vec<Vec3>> = levels(layout)
        .into_iter()
        .map(|(along, (across, deep))| {
            let middle = axes.at(along);
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
    let levels = levels(&Layout::biped(frame, NONE)).map(|(along, half)| (hip + along, half));
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
