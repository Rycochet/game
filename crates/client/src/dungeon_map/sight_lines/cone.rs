//! The edge of sight for someone facing a way (`shared::facing::Cone`): the
//! outline as it is inside the cone, and only out to the near circle behind.

use std::f64::consts::{PI, TAU};

use shared::facing::{Cone, TURN};
use shared::spot::CELL_SPAN;

use super::{BESIDE_CORNER, Vec2d, WallMap};

impl WallMap {
    /// What can be seen from `from` facing as `cone` says (all round without
    /// one): rays inside the cone end on the first wall, others at the near
    /// circle if they reach it. Rays go along both edges of the cone, so the
    /// edge is straight.
    pub fn outline_in(&self, from: Vec2d, cone: Option<&Cone>) -> Vec<Vec2d> {
        let Some(cone) = cone.filter(|cone| !cone.field.is_all_round()) else {
            return self.outline(from);
        };
        let heading = f64::from(cone.heading.radians());
        let half = f64::from(cone.field.half) / f64::from(TURN) * TAU;
        let near = cone.field.near as f64 / CELL_SPAN as f64;
        let mut angles = self.angles(from);
        for edge in [heading - half, heading + half] {
            angles.extend([edge - BESIDE_CORNER, edge + BESIDE_CORNER]);
        }
        let mut angles: Vec<f64> = angles.into_iter().map(round_to_half_turn).collect();
        angles.sort_by(f64::total_cmp);
        angles
            .into_iter()
            .map(|angle| {
                let end = self.ray_end(from, angle);
                if round_to_half_turn(angle - heading).abs() <= half {
                    return end;
                }
                let (dx, dy) = (end.x - from.x, end.y - from.y);
                let length = dx.hypot(dy);
                if length <= near {
                    return end;
                }
                Vec2d {
                    x: from.x + dx / length * near,
                    y: from.y + dy / length * near,
                }
            })
            .collect()
    }
}

/// An angle brought into -π to π.
fn round_to_half_turn(angle: f64) -> f64 {
    (angle + PI).rem_euclid(TAU) - PI
}
