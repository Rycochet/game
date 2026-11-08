//! The jagged shape of a bolt of lightning, rolled from its seed, so everyone
//! sees the same one: a main stroke from the cloud to the ground and a few
//! forks off it.

use bevy::prelude::*;
use shared::rng::Seed;

/// Steps down the main stroke.
const STEPS: usize = 12;

/// How far each step may wander sideways, as a share of the bolt's height.
const JAG: f32 = 0.07;

/// Forks off the main stroke: at most this many, each this many steps long,
/// this much thinner.
const MOST_FORKS: u64 = 3;
const FORK_STEPS: (u64, u64) = (3, 6);
const FORK_WIDTH: f32 = 0.5;

/// A straight piece of a bolt, in a bolt one unit high that strikes the
/// origin, and how wide it is against the main stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Segment {
    pub from: Vec3,
    pub to: Vec3,
    pub width: f32,
}

/// The pieces of the bolt with this seed: the main stroke first, from the top
/// (height 1) to the origin, then the forks.
pub(super) fn segments(seed: u64) -> Vec<Segment> {
    let mut rng = Seed::new(seed).rng();
    let mut jag = || (rng.next_f64() as f32 * 2.0 - 1.0) * JAG;
    // A random walk down, then leant so that it ends on the origin.
    let mut points = vec![Vec3::Y];
    let mut side = Vec2::ZERO;
    for step in 1..=STEPS {
        side += Vec2::new(jag(), jag());
        let y = 1.0 - step as f32 / STEPS as f32;
        points.push(Vec3::new(side.x, y, side.y));
    }
    for (step, point) in points.iter_mut().enumerate() {
        let lean = side * (step as f32 / STEPS as f32);
        point.x -= lean.x;
        point.z -= lean.y;
    }
    let mut pieces: Vec<Segment> = points
        .windows(2)
        .map(|pair| Segment {
            from: pair[0],
            to: pair[1],
            width: 1.0,
        })
        .collect();

    let mut rng = Seed::new(seed).derive("forks").rng();
    for _ in 0..rng.range_u64(1, MOST_FORKS + 1) {
        let start = rng.range_u64(2, STEPS as u64 - 3) as usize;
        let angle = rng.next_f64() as f32 * std::f32::consts::TAU;
        let away = Vec2::from_angle(angle) * JAG * 1.5;
        let mut at = points[start];
        for _ in 0..rng.range_u64(FORK_STEPS.0, FORK_STEPS.1 + 1) {
            let wobble = Vec2::new(rng.next_f64() as f32 - 0.5, rng.next_f64() as f32 - 0.5) * JAG;
            let next = at + Vec3::new(away.x + wobble.x, -0.8 / STEPS as f32, away.y + wobble.y);
            if next.y <= 0.0 {
                break;
            }
            pieces.push(Segment {
                from: at,
                to: next,
                width: FORK_WIDTH,
            });
            at = next;
        }
    }
    pieces
}

#[cfg(test)]
mod tests;
