//! Arms (and wing-arms) from their joints: upper arm and forearm as one limb, a hand,
//! and a round shoulder.

use bevy::prelude::*;
use shared::body::{Layout, Limb};

use super::{Triangle, blob, limb, point};

/// One arm: its bones are the upper arm, the forearm and the hand.
pub(super) fn build(triangles: &mut Vec<Triangle>, layout: &Layout, arm: &Limb) {
    let joints: Vec<Vec3> = layout.joints(arm).into_iter().map(point).collect();
    let [Some(upper), Some(fore), Some(hand), ..] = arm.bones else {
        return;
    };
    let (shoulder, elbow, wrist, tips) = (joints[0], joints[1], joints[2], joints[3]);
    let (upper, fore) = (upper.radius as f32, fore.radius as f32);
    limb(
        triangles,
        &[(shoulder, upper), (elbow, fore * 1.05), (wrist, fore * 0.7)],
    );
    blob(
        triangles,
        wrist.lerp(tips, 0.5),
        Vec3::new(fore * 0.45, hand.span as f32 / 2.0, fore * 1.1),
    );
    blob(triangles, shoulder, Vec3::splat(upper * 1.15));
}
