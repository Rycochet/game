//! Small extras on an animal's body: a cow's dewlap hanging under its neck, and its
//! udder under the belly in front of the hind legs.

use bevy::prelude::*;
use shared::body::{Layout, Ring};

use super::head::neck_rings;
use super::trunk::{self, Axes};
use super::{Triangle, blob, limb};

/// Lobes along the dewlap.
const LOBES: usize = 6;

/// Whatever extras this body has.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout) {
    if layout.parts.dewlap > 0 {
        dewlap(triangles, layout, layout.parts.dewlap as f32);
    }
    if layout.parts.udder > 0 {
        udder(triangles, layout, layout.parts.udder as f32);
    }
}

/// A thin fold of skin hanging from the neck's underside, deepest a little below its
/// middle: flat lobes along it, from the chest to the throat.
fn dewlap(triangles: &mut Vec<Triangle>, layout: &Layout, hang: f32) {
    let rings = neck_rings(layout);
    // The neck's underside, a share of the way from the trunk to the head.
    let underside = |t: f32| {
        let (from, to, t) = if t < 0.5 {
            (rings[0], rings[1], t * 2.0)
        } else {
            (rings[1], rings[2], t * 2.0 - 1.0)
        };
        (from.0 + from.2).lerp(to.0 + to.2, t)
    };
    let thin = layout.neck.radius as f32 * 0.15;
    let step = (underside(1.0) - underside(0.0)).length() / LOBES as f32;
    for lobe in 0..LOBES {
        let t = (lobe as f32 + 0.5) / LOBES as f32;
        let deep = hang * (t * std::f32::consts::PI).sin().powf(0.7);
        let top = underside(t * 0.85 + 0.05);
        blob(
            triangles,
            top - Vec3::Y * deep * 0.5,
            Vec3::new(thin, deep * 0.5 + 15.0, step * 0.7),
        );
    }
}

/// An udder under the belly just in front of the hind legs, with four teats.
fn udder(triangles: &mut Vec<Triangle>, layout: &Layout, size: f32) {
    let axes = Axes::of(layout);
    let along = layout.spine.shoulders as f32 * 0.18;
    // The ring nearest that far along, for where the belly is.
    let ring: Ring = *layout
        .spine
        .rings
        .iter()
        .min_by_key(|ring| (ring.along as f32 - along).abs() as i32)
        .expect("rings");
    let belly = trunk::middle_of(&axes, &ring)
        + axes.front * (ring.belly + ring.back) as f32 / 2.0
        + axes.along * (along - ring.along as f32);
    let middle = belly + axes.front * size * 0.3;
    blob(
        triangles,
        middle,
        Vec3::new(size * 0.5, size * 0.45, size * 0.6),
    );
    for (x, z) in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
        let from = middle + Vec3::new(x * size * 0.22, -size * 0.3, z * size * 0.25);
        limb(
            triangles,
            &[
                (from, size * 0.07),
                (from - Vec3::Y * size * 0.35, size * 0.05),
            ],
        );
    }
}
