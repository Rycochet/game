//! One thing in a creature's way, and how close a move passes it.

use crate::dungeon::line_hits_box;
use crate::spot::{Spot, root};

/// The shape of a blocker, in units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// A box from its smallest corner to its largest, edges included.
    Box { low: Spot, high: Spot },
    /// A round post (a tree's trunk) at `at`, `radius` units across from it.
    Post { at: Spot, radius: i64 },
}

/// Something that stops creatures, from a level's cells (`cell_blockers`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blocker {
    pub shape: Shape,
    /// The cell whose fence this is: a move crossing it is a jump over that
    /// cell. `None` for what can't be jumped.
    pub fence: Option<(i64, i64)>,
}

impl Shape {
    /// How far a move from `a` to `b` passes from it, rounded down to a whole
    /// unit: 0 if it touches or crosses it.
    pub fn clearance(&self, a: Spot, b: Spot) -> i64 {
        match *self {
            Shape::Box { low, high } => {
                if line_hits_box((a.x, a.y), (b.x, b.y), (low.x, low.y), (high.x, high.y)) {
                    return 0;
                }
                let corners = [
                    low,
                    Spot::new(high.x, low.y),
                    Spot::new(low.x, high.y),
                    high,
                ];
                let nearest = corners
                    .iter()
                    .map(|&corner| point_to_segment(corner, a, b))
                    .chain([point_to_box(a, low, high), point_to_box(b, low, high)])
                    .min()
                    .expect("there are corners");
                root(nearest) as i64
            }
            Shape::Post { at, radius } => (root(point_to_segment(at, a, b)) as i64 - radius).max(0),
        }
    }
}

/// The square of the distance from `p` to the nearest point of the box,
/// exactly.
fn point_to_box(p: Spot, low: Spot, high: Spot) -> u128 {
    let dx = (low.x - p.x).max(p.x - high.x).max(0) as i128;
    let dy = (low.y - p.y).max(p.y - high.y).max(0) as i128;
    (dx * dx + dy * dy) as u128
}

/// The square of the distance from `p` to the nearest point of the segment
/// from `a` to `b`, rounded down.
pub(super) fn point_to_segment(p: Spot, a: Spot, b: Spot) -> u128 {
    let (dx, dy) = ((b.x - a.x) as i128, (b.y - a.y) as i128);
    let (px, py) = ((p.x - a.x) as i128, (p.y - a.y) as i128);
    let length = dx * dx + dy * dy;
    let along = px * dx + py * dy;
    if length == 0 || along <= 0 {
        return p.distance_squared(a);
    }
    if along >= length {
        return p.distance_squared(b);
    }
    let across = px * dy - py * dx;
    (across * across / length) as u128
}
