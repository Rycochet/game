//! A level's walls ready for casting rays, and what a point sees.

use bevy::prelude::*;
use shared::dungeon::{CELL_UNITS, Level};
use std::f64::consts::TAU;

use super::{BESIDE_CORNER, EVEN_RAYS};

/// A point in cells, whose centres are at whole numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

/// A bar of wall in cells: its smallest and largest corners.
pub(super) type Bar = (Vec2d, Vec2d);

/// The walls of a level, ready for casting rays, with the corners worth casting
/// them at.
#[derive(Default)]
pub struct WallMap {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    bars: Vec<Option<[Bar; 2]>>,
    corners: Vec<Vec2d>,
}

impl WallMap {
    pub fn of(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        let to_cells = |units: i64| units as f64 / CELL_UNITS as f64;
        let mut bars = Vec::with_capacity((width * height) as usize);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                bars.push(level.wall_bars(x, y).map(|found| {
                    found.map(|(low, high)| {
                        (
                            Vec2d {
                                x: to_cells(low.0),
                                y: to_cells(low.1),
                            },
                            Vec2d {
                                x: to_cells(high.0),
                                y: to_cells(high.1),
                            },
                        )
                    })
                }));
            }
        }
        let mut map = Self {
            min_x,
            min_y,
            width,
            height,
            bars,
            corners: Vec::new(),
        };
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let Some(found) = map.bars_at(x, y).copied() else {
                    continue;
                };
                for (low, high) in found {
                    for corner in [
                        low,
                        high,
                        Vec2d {
                            x: low.x,
                            y: high.y,
                        },
                        Vec2d {
                            x: high.x,
                            y: low.y,
                        },
                    ] {
                        if !map.corner_is_hidden(corner, (x, y)) {
                            map.corners.push(corner);
                        }
                    }
                }
            }
        }
        map
    }

    fn bars_at(&self, x: i64, y: i64) -> Option<&[Bar; 2]> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        if !(0..self.width).contains(&col) || !(0..self.height).contains(&row) {
            return None;
        }
        self.bars[(row * self.width + col) as usize].as_ref()
    }

    /// Whether a bar's corner is no corner of the wall as a whole: inside its
    /// own cell's other bar, or where the wall carries on into the next cell.
    fn corner_is_hidden(&self, corner: Vec2d, cell: (i64, i64)) -> bool {
        for y in cell.1 - 1..=cell.1 + 1 {
            for x in cell.0 - 1..=cell.0 + 1 {
                let Some(found) = self.bars_at(x, y) else {
                    continue;
                };
                let own = (x, y) == cell;
                for (low, high) in found {
                    let inside = if own {
                        low.x < corner.x
                            && corner.x < high.x
                            && low.y < corner.y
                            && corner.y < high.y
                    } else {
                        low.x <= corner.x
                            && corner.x <= high.x
                            && low.y <= corner.y
                            && corner.y <= high.y
                    };
                    if inside {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// What can be seen from a point: the ends of its rays, going round.
    pub fn outline(&self, from: Vec2d) -> Vec<Vec2d> {
        let mut angles = Vec::with_capacity(self.corners.len() * 2 + EVEN_RAYS);
        for i in 0..EVEN_RAYS {
            angles.push(i as f64 / EVEN_RAYS as f64 * TAU - TAU / 2.0);
        }
        for corner in &self.corners {
            let angle = (corner.y - from.y).atan2(corner.x - from.x);
            angles.push(angle - BESIDE_CORNER);
            angles.push(angle + BESIDE_CORNER);
        }
        angles.sort_by(f64::total_cmp);
        angles
            .into_iter()
            .map(|angle| self.ray_end(from, angle))
            .collect()
    }

    /// Where a ray stops: the first wall, or just past the level's edge. Goes
    /// through the cells it crosses in turn.
    pub(super) fn ray_end(&self, from: Vec2d, angle: f64) -> Vec2d {
        let (dx, dy) = (angle.cos(), angle.sin());
        let at = |along: f64| Vec2d {
            x: from.x + dx * along,
            y: from.y + dy * along,
        };
        let (mut x, mut y) = ((from.x + 0.5).floor() as i64, (from.y + 0.5).floor() as i64);
        let (step_x, step_y) = (if dx > 0.0 { 1 } else { -1 }, if dy > 0.0 { 1 } else { -1 });
        // How far along the ray the next cell edge is, each way.
        let edge = |start: f64, cell: i64, step: i64, by: f64| {
            if by == 0.0 {
                f64::INFINITY
            } else {
                (cell as f64 + step as f64 * 0.5 - start) / by
            }
        };
        let mut next_x = edge(from.x, x, step_x, dx);
        let mut next_y = edge(from.y, y, step_y, dy);
        let (cross_x, cross_y) = (1.0 / dx.abs(), 1.0 / dy.abs());
        let (right, bottom) = (self.min_x + self.width, self.min_y + self.height);
        let mut along = 0.0;
        loop {
            if x < self.min_x || y < self.min_y || x >= right || y >= bottom {
                return at(along);
            }
            if let Some(found) = self.bars_at(x, y) {
                let hit = found
                    .iter()
                    .filter_map(|bar| ray_hits_bar(from, (dx, dy), bar))
                    .min_by(f64::total_cmp);
                if let Some(hit) = hit {
                    return at(hit);
                }
            }
            if next_x < next_y {
                along = next_x;
                next_x += cross_x;
                x += step_x;
            } else {
                along = next_y;
                next_y += cross_y;
                y += step_y;
            }
        }
    }
}

/// How far along a ray it first meets a bar, if it does. A ray starting inside
/// a bar passes through it: a viewer can't be blinded by the wall it is cutting
/// the corner of.
pub(super) fn ray_hits_bar(from: Vec2d, (dx, dy): (f64, f64), (low, high): &Bar) -> Option<f64> {
    let slab = |start: f64, by: f64, low: f64, high: f64| {
        if by == 0.0 {
            (low..=high)
                .contains(&start)
                .then_some((f64::NEG_INFINITY, f64::INFINITY))
        } else {
            let (a, b) = ((low - start) / by, (high - start) / by);
            Some((a.min(b), a.max(b)))
        }
    };
    let (x_in, x_out) = slab(from.x, dx, low.x, high.x)?;
    let (y_in, y_out) = slab(from.y, dy, low.y, high.y)?;
    let (enter, leave) = (x_in.max(y_in), x_out.min(y_out));
    (enter <= leave && enter > 0.0).then_some(enter)
}
