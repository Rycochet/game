//! A cave mouth's level as a hill: the rock round its caves at least a few
//! cells deep, with no inward bend anywhere (owner: it looks like a hill and
//! gives nothing of the caves' shape away), outside ground round it, and each
//! mouth's passage carried straight out through it.
//!
//! The rock is every cell within the depth of the convex hull of the caves'
//! open cells (a convex shape, so each row and column of it is one run),
//! worked out in whole numbers so every machine agrees.

use super::super::SiteLevel;
use super::{BORDER, Mouth};
use crate::dungeon::{Cell, Level};

/// The least depth of rock round a cave mouth's caves, rolled per cave
/// (owner's range, 3 to 8): deeper wherever the hull needs it.
pub(super) const LEAST_ROCK: (i64, i64) = (3, 8);

/// Makes a cave mouth's level (its caves dug, starting at `(0, 0)`) a hill
/// of rock `depth` deep round its caves, with `BORDER` of outside ground
/// round that; the mouths' ground goes, their passages run out through the
/// rock, and the first one's end is the new entrance.
pub(super) fn raise_outcrop(made: &mut SiteLevel, mouths: &[Mouth], depth: i64) {
    let old = &made.level;
    let (_, _, width, height) = old.bounds();
    // The mouths' ground: the strips along the south, east and west edges.
    let mouth_ground = |x: i64, y: i64| y >= height - BORDER || x < BORDER || x >= width - BORDER;
    let mut open = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if old.cell(x, y) != Cell::Wall && !mouth_ground(x, y) {
                open.push((x, y));
            }
        }
    }
    let rock = Rock {
        hull: convex_hull(open),
        depth,
    };
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    for y in -depth - 1..=height + depth {
        for x in -depth - 1..=width + depth {
            if rock.covers(x, y) {
                (min_x, min_y) = (min_x.min(x), min_y.min(y));
                (max_x, max_y) = (max_x.max(x), max_y.max(y));
            }
        }
    }
    // Added to an old cell to give the new one.
    let (sx, sy) = (BORDER - min_x, BORDER - min_y);
    let new_width = max_x - min_x + 1 + 2 * BORDER;
    let new_height = max_y - min_y + 1 + 2 * BORDER;
    let mut level = Level::new(0, 0, new_width, new_height);
    for y in 0..new_height {
        for x in 0..new_width {
            let (ox, oy) = (x - sx, y - sy);
            let kept = rock.covers(ox, oy) && old.contains(ox, oy) && !mouth_ground(ox, oy);
            let cell = match (rock.covers(ox, oy), kept) {
                (false, _) => Cell::Ground,
                (true, false) => Cell::Wall,
                (true, true) => old.cell(ox, oy),
            };
            level.set(x, y, cell);
            level.set_room(x, y, if kept { old.room(ox, oy) } else { None });
        }
    }
    let mut entrance = None;
    for mouth in mouths {
        for &(px, py) in &mouth.passage {
            let (mut x, mut y) = (px + mouth.out.0, py + mouth.out.1);
            while rock.covers(x, y) {
                level.set(x + sx, y + sy, Cell::Floor);
                level.set_room(x + sx, y + sy, None);
                (x, y) = (x + mouth.out.0, y + mouth.out.1);
            }
            entrance.get_or_insert((x + sx, y + sy));
        }
    }
    for stairs in &mut made.stairs {
        (stairs.x, stairs.y) = (stairs.x + sx, stairs.y + sy);
    }
    made.entrance = entrance.or(made.entrance.map(|(x, y)| (x + sx, y + sy)));
    made.level = level;
}

/// The rock: within `depth` of the convex hull.
struct Rock {
    /// Corners, anticlockwise (with y down: each turn's cross product is
    /// positive), without repeats; one or two when the caves are a dot or a
    /// line.
    hull: Vec<(i64, i64)>,
    depth: i64,
}

impl Rock {
    fn covers(&self, x: i64, y: i64) -> bool {
        let count = self.hull.len();
        let edges = (0..count).map(|i| (self.hull[i], self.hull[(i + 1) % count]));
        let inside = count >= 3 && edges.clone().all(|(a, b)| cross(a, b, (x, y)) >= 0);
        inside
            || edges
                .into_iter()
                .any(|(a, b)| near(a, b, (x, y), self.depth))
    }
}

/// Twice the signed area of `a`, `b`, `p`: positive when `p` is to the left of
/// `a` → `b` (counting as the hull's corners go).
fn cross(a: (i64, i64), b: (i64, i64), p: (i64, i64)) -> i64 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Whether `p` is within `depth` of the segment from `a` to `b` (squared
/// distances, so whole numbers throughout).
fn near(a: (i64, i64), b: (i64, i64), p: (i64, i64), depth: i64) -> bool {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let (apx, apy) = (p.0 - a.0, p.1 - a.1);
    let length = abx * abx + aby * aby;
    let along = apx * abx + apy * aby;
    if length == 0 || along <= 0 {
        return apx * apx + apy * apy <= depth * depth;
    }
    if along >= length {
        let (bpx, bpy) = (p.0 - b.0, p.1 - b.1);
        return bpx * bpx + bpy * bpy <= depth * depth;
    }
    let side = abx * apy - aby * apx;
    side * side <= depth * depth * length
}

/// The convex hull of `points` (Andrew's monotone chain), corners in order.
fn convex_hull(mut points: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    points.sort_unstable();
    points.dedup();
    if points.len() < 3 {
        return points;
    }
    let mut hull: Vec<(i64, i64)> = Vec::with_capacity(points.len() + 1);
    for pass in [false, true] {
        let start = hull.len();
        let ordered: Box<dyn Iterator<Item = &(i64, i64)>> = if pass {
            Box::new(points.iter().rev())
        } else {
            Box::new(points.iter())
        };
        for &p in ordered {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
    }
    hull
}

#[cfg(test)]
mod tests;
