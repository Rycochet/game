//! An animal's long hair, drawn apart from its coat in a darker shade of it: a mane
//! along the top of the neck and falling down both sides, and the long hair of a
//! tail (a horse's whole tail, a cow's switch at the end).

use bevy::prelude::*;
use shared::body::Layout;

use super::trunk;
use super::{HeadAxes, Triangle, crown, features, limb, point};

/// Steps along the neck for the mane.
const STEPS: usize = 8;

/// The mane and tail hair this layout has, if any.
pub(in crate::item_art) fn coat_hair(layout: &Layout) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    if layout.parts.mane > 0 {
        mane(&mut triangles, layout, layout.parts.mane as f32);
    }
    if layout.parts.tail_hair > 0 && layout.parts.tail > 0 {
        tail_hair(&mut triangles, layout, layout.parts.tail_hair as f32);
    }
    triangles
}

/// Along the top of the neck from the withers to behind the ears, a ridge falling a
/// little way down each side; both faces drawn, as feathers are.
fn mane(triangles: &mut Vec<Triangle>, layout: &Layout, hang: f32) {
    let base = trunk::neck_root(layout);
    // To behind the ears: the top of a measured skull, or above the chin.
    let top = match layout.head.shape {
        Some(shape) => {
            let skull = crown(&shape);
            HeadAxes::of(layout, &shape).at(skull.along as f32 * 0.5, skull.back as f32 * 0.5, 0.0)
        }
        None => point(layout.chin()) + Vec3::Y * layout.head.length as f32 * 0.35,
    };
    let along = (top - base).normalize_or(Vec3::Y);
    // Out of the top of the neck: up, and back where the neck leans forwards.
    let up = (Vec3::Y - along * Vec3::Y.dot(along)).normalize_or(Vec3::NEG_Z);
    let side = along.cross(up).normalize_or(Vec3::X);
    let radius = layout.neck.radius as f32;
    let rows: Vec<[Vec3; 3]> = (0..=STEPS)
        .map(|step| {
            let t = step as f32 / STEPS as f32;
            // Longest in the middle of the neck, shorter at the withers and the poll.
            let long = hang * (0.5 + 0.5 * (t * std::f32::consts::PI).sin());
            let on = base.lerp(top, t);
            let crest = on + up * (radius + 55.0);
            let fall = |out: f32| on + side * out * (radius + 12.0) + up * (radius * 0.6 - long);
            [fall(1.0), crest, fall(-1.0)]
        })
        .collect();
    for pair in rows.windows(2) {
        for edge in 0..2 {
            let (a, b) = (pair[0], pair[1]);
            for [p, q, r] in [
                [a[edge], b[edge], b[edge + 1]],
                [a[edge], b[edge + 1], a[edge + 1]],
            ] {
                triangles.push([p, q, r]);
                triangles.push([p, r, q]);
            }
        }
    }
}

/// Long hair over the tail and hanging from it, nearly to the ground for a horse; a
/// shorter tuft at the end of a cow's.
fn tail_hair(triangles: &mut Vec<Triangle>, layout: &Layout, hair: f32) {
    let path = features::tail_path(layout, layout.parts.tail as f32);
    // Hair as long as half the tail or more covers it from near the root.
    let from = if hair * 2.0 >= layout.parts.tail as f32 {
        1
    } else {
        2
    };
    let (end, _) = path[3];
    let tip = end + Vec3::new(0.0, -hair * 0.45, -hair * 0.1);
    let tip = Vec3::new(tip.x, tip.y.max(40.0), tip.z);
    let mut hanging: Vec<(Vec3, f32)> = path[from..]
        .iter()
        .map(|&(at, thick)| (at, thick + hair * 0.05))
        .collect();
    hanging.push((end.lerp(tip, 0.5), hair * 0.07));
    hanging.push((tip, 6.0));
    limb(triangles, &hanging);
}
