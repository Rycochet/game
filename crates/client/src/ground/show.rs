//! Showing each bag as it is now: in colour, remembered or gone.

use bevy::prelude::*;

use super::{BagPart, OfBag, Sparkle, Star, TURN_PER_SECOND, TWINKLE, TWINKLE_PER_SECOND};
use crate::common::HoveredBag;
use crate::dungeon_map::Shown;
use crate::view3d::ViewCamera;

/// Shows each bag as it is now: in colour with its sparkle while in sight, or in grey
/// without one, as last seen; a wider outline if the pointer is on it.
#[allow(clippy::type_complexity)]
pub(super) fn show_bags(
    shown: Res<Shown>,
    hovered: Res<HoveredBag>,
    time: Res<Time<Real>>,
    camera: Single<&GlobalTransform, With<ViewCamera>>,
    mut parts: Query<(&BagPart, &OfBag, &mut Visibility), Without<Sparkle>>,
    mut sparkles: Query<(&Sparkle, &mut Transform, &mut Visibility), Without<Star>>,
    mut stars: Query<(&Star, &ChildOf, &mut Transform), Without<Sparkle>>,
) {
    let in_sight = |cell: (i64, i64)| {
        let sight = shown.sight.as_ref();
        sight.is_some_and(|sight| sight.is_seen(cell.0, cell.1))
    };
    for (part, of_bag, mut visibility) in &mut parts {
        let seen = in_sight(of_bag.0);
        let over = hovered.0 == Some(of_bag.0);
        let show = match part {
            BagPart::Lit => seen,
            BagPart::Remembered => !seen,
            BagPart::HoverOutline => over,
            BagPart::SparkleHoverOutline => over && seen,
        };
        visibility.set_if_neq(if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }

    let now = time.elapsed_secs();
    let camera_turn = camera.rotation();
    let forward = camera.forward();
    // Straight behind on the ground, so that it shows above the bag.
    let behind = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    for (sparkle, mut transform, mut visibility) in &mut sparkles {
        visibility.set_if_neq(if in_sight(sparkle.cell) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        transform.translation = Vec3::Y * sparkle.up + behind * sparkle.behind;
        transform.rotation = camera_turn;
        transform.scale = Vec3::splat(sparkle.scale);
    }
    for (star, parent, mut transform) in &mut stars {
        let Ok((sparkle, ..)) = sparkles.get(parent.parent()) else {
            continue;
        };
        let at = now + sparkle.phase;
        let pulse = 1.0 + TWINKLE * (at * TWINKLE_PER_SECOND + star.phase).sin();
        transform.scale = Vec3::splat(star.scale * pulse);
        transform.rotation = Quat::from_rotation_z(at * TURN_PER_SECOND + star.phase);
    }
}
