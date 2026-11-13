//! What hair adds to the cap: a tail, two braids, a knot on top, a crest, curls. A
//! tail or braids come out from under a helmet's rim, if one is worn low.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;
use shared::body::HairStyle;

use super::super::{blob, limb};
use super::scalp::{GROW, cap, hairline};
use super::{Cover, Skull, Triangle};

/// Where hanging hair starts at this angle: at this height, or just under a helmet's
/// rim. None if the rim comes down past the jaw, as a sallet's.
fn root(skull: &Skull, cover: &Cover, turn: f32, height: f32) -> Option<Vec3> {
    let y = height.min(cover.rim_at(turn) - 12.0);
    let lowest = skull.middle.y - skull.up * 0.8;
    (y >= lowest).then(|| skull.at(turn, skull.down_to(y, GROW), GROW - 3.0))
}

/// How far back of the spine hanging hair keeps, to lie over the back (and armour).
fn behind(skull: &Skull, thick: f32) -> f32 {
    skull.neck_base.z - skull.chest_back - thick - 25.0
}

/// A tail from the back of the head, tied near its root, down to between the shoulders.
pub(super) fn tail(triangles: &mut Vec<Triangle>, skull: &Skull, cover: &Cover) {
    let Some(from) = root(skull, cover, PI, skull.middle.y - skull.up * 0.15) else {
        return;
    };
    let tie = from + Vec3::new(0.0, -25.0, -30.0);
    blob(triangles, tie, Vec3::splat(15.0));
    let low = skull.neck_base.y - 60.0;
    let back = |p: Vec3, thick: f32| Vec3::new(p.x, p.y, p.z.min(behind(skull, thick)));
    limb(
        triangles,
        &[
            (from, 16.0),
            (tie, 14.0),
            (back(tie + Vec3::new(0.0, -70.0, -25.0), 15.0), 15.0),
            (back(Vec3::new(from.x, low, tie.z), 12.0), 12.0),
            (back(Vec3::new(from.x, low - 50.0, tie.z), 5.0), 5.0),
        ],
    );
}

/// Two plaits from behind the ears down the back, knotted all along.
pub(super) fn braids(triangles: &mut Vec<Triangle>, skull: &Skull, cover: &Cover) {
    for side in [1.0, -1.0] {
        let turn = side * (PI - 0.6);
        let Some(from) = root(skull, cover, turn, skull.middle.y - skull.up * 0.35) else {
            continue;
        };
        let end = skull.neck_base.y - 170.0;
        let steps = 9;
        let path: Vec<(Vec3, f32)> = (0..=steps)
            .map(|step| {
                let along = step as f32 / steps as f32;
                let y = from.y + (end - from.y) * along;
                let clear = behind(skull, 12.0).min(from.z);
                let z = from.z + (clear - from.z) * along.sqrt();
                let thick = if step % 2 == 0 { 13.0 } else { 9.0 } * (1.0 - along * 0.4);
                (Vec3::new(from.x * (1.0 - along * 0.3), y, z), thick)
            })
            .collect();
        limb(triangles, &path);
    }
}

/// A knot on the crown, a little behind its top.
pub(super) fn topknot(triangles: &mut Vec<Triangle>, skull: &Skull) {
    let top = skull.at(PI, 0.25, GROW);
    blob(triangles, top + Vec3::Y * 18.0, Vec3::new(22.0, 20.0, 22.0));
}

/// A ridge of hair from the forehead over the crown to the nape, the sides bare.
pub(super) fn crest(triangles: &mut Vec<Triangle>, skull: &Skull) {
    let front = [0.85, 0.55, 0.25].map(|down| (0.0, down, 10.0));
    let back = [0.0, 0.3, 0.65, 1.0, 1.35, FRAC_PI_2 + 0.35].map(|down| (PI, down, 12.0));
    let path: Vec<(Vec3, f32)> = front
        .into_iter()
        .chain(back)
        .map(|(turn, down, thick)| (skull.at(turn, down, 12.0), thick * 1.3))
        .collect();
    limb(triangles, &path);
}

/// A mop of curls over a close cap, where hair grows and below any rim.
pub(super) fn curls(triangles: &mut Vec<Triangle>, skull: &Skull, cover: &Cover) {
    cap(triangles, skull, cover, HairStyle::Short);
    let mut place = |turn: f32, down: f32| {
        let at = skull.at(turn, down, GROW + 4.0);
        if at.y >= hairline(skull, turn) - 5.0 && at.y < cover.rim_at(turn) {
            blob(triangles, at, Vec3::splat(20.0));
        }
    };
    place(0.0, 0.0);
    for (row, (down, count)) in [(0.4, 6), (0.8, 10), (1.2, 12), (1.6, 12)]
        .into_iter()
        .enumerate()
    {
        // Every other row turned half a step, so the curls sit between.
        let shift = if row % 2 == 1 { 0.5 } else { 0.0 };
        for at in 0..count {
            let turn = (at as f32 + shift) / count as f32 * TAU;
            place(turn, down);
        }
    }
}
