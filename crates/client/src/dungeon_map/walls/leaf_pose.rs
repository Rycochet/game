//! How a door's leaf is drawn: how far it is turned, which way, and how far it
//! hangs askew; and the poses on the way from one to another, for a swing.

use bevy::prelude::*;
use shared::door::{Side, Swing};
use shared::dungeon::{Cell, Level};

use super::door_leaf::sides_of;

/// How far an open door's leaf is turned from shut.
pub(super) const LEAF_OPEN: f32 = std::f32::consts::FRAC_PI_2;

/// How far a broken leaf hangs askew from its top hinge.
const BROKEN_TILT: f32 = 0.25;

/// A leaf as drawn: `turned` from shut (radians), the `way` it turns (1 or -1
/// along the wall's way out), and how far it hangs askew (`tilt`: 0 whole, 1
/// broken).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafPose {
    pub turned: f32,
    pub way: f32,
    pub tilt: f32,
}

impl LeafPose {
    /// The door at `at` as it hangs (`Level::swing`): shut in the hole, open
    /// flat to the wall on its side (half as far for a secret door), or broken,
    /// hanging askew. A door whose swing isn't known is open, the way a hash
    /// of its cell says.
    pub fn of(level: &Level, at: (i64, i64)) -> Self {
        // A gate's two leaves turn the same way.
        let first = shared::door::gate_first_leaf(level, at.0, at.1);
        let (_, hashed) = sides_of(first.0, first.1);
        let secret = level.cell(at.0, at.1) == Cell::SecretDoor;
        let way = |side: Side| match side {
            Side::South | Side::East => 1.0,
            Side::North | Side::West => -1.0,
        };
        let (turned, way, tilt) = match level.swing(at.0, at.1) {
            None => (LEAF_OPEN, hashed, 0.0),
            Some(Swing::Closed) => (0.0, hashed, 0.0),
            Some(Swing::Open(side)) if secret => (LEAF_OPEN / 2.0, way(side), 0.0),
            Some(Swing::Open(side)) => (LEAF_OPEN, way(side), 0.0),
            Some(Swing::Broken(side)) => (LEAF_OPEN, way(side), 1.0),
        };
        Self { turned, way, tilt }
    }

    /// The pose `share` (0 to 1) of the way from `from` to `to`: turning
    /// through shut if it goes from one side to the other.
    pub fn between(from: Self, to: Self, share: f32) -> Self {
        let share = share.clamp(0.0, 1.0);
        let signed = |pose: Self| pose.turned * pose.way;
        let angle = signed(from) + (signed(to) - signed(from)) * share;
        Self {
            turned: angle.abs(),
            // Shut, either way looks the same: keep the one it is going to.
            way: if angle == 0.0 { to.way } else { angle.signum() },
            tilt: from.tilt + (to.tilt - from.tilt) * share,
        }
    }
}

/// A leaf's outline turned askew about its top at the hinge by `tilt` of a
/// broken leaf's lean, its lowest point kept on the floor.
pub(super) fn askew(profile: &[Vec2], tilt: f32) -> Vec<Vec2> {
    let top = profile.iter().map(|p| p.y).fold(0.0, f32::max);
    let (sin, cos) = (BROKEN_TILT * tilt).sin_cos();
    let turned: Vec<Vec2> = profile
        .iter()
        .map(|p| {
            let (dx, dy) = (p.x, p.y - top);
            Vec2::new(dx * cos + dy * sin, top - dx * sin + dy * cos)
        })
        .collect();
    let lowest = turned.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    turned.into_iter().map(|p| p - Vec2::Y * lowest).collect()
}
