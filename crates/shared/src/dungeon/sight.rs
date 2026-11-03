//! What can be seen from a cell, and how far light reaches.

use super::{CELL_UNITS, Level, clear_line};

/// The cells that can be seen (or lit) from one or more points in a level.
/// A wall is seen when an open cell touching it (corners too) is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sight {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    seen: Vec<bool>,
}

impl Sight {
    /// Nothing seen yet, sized for `level`.
    pub fn new(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            seen: vec![false; (width * height) as usize],
        }
    }

    /// Whether `(x, y)` is seen. Always false outside the level.
    pub fn is_seen(&self, x: i64, y: i64) -> bool {
        let (col, row) = (x - self.min_x, y - self.min_y);
        (0..self.width).contains(&col)
            && (0..self.height).contains(&row)
            && self.seen[(row * self.width + col) as usize]
    }

    /// Counts `(x, y)` as seen, e.g. a cell remembered from earlier.
    /// Nothing happens outside the level.
    pub fn insert(&mut self, x: i64, y: i64) {
        self.mark(x, y);
    }

    fn mark(&mut self, x: i64, y: i64) {
        let (col, row) = (x - self.min_x, y - self.min_y);
        if (0..self.width).contains(&col) && (0..self.height).contains(&row) {
            self.seen[(row * self.width + col) as usize] = true;
        }
    }

    /// Adds everything seen from `(x, y)`, out to `radius` cells (straight-line
    /// distance between cell centres; `None` for no limit). Several calls
    /// combine, e.g. for allies sharing their sight. Open cells count as seen
    /// when a straight line between the centres misses every wall, so seeing
    /// is symmetric (if A sees B, B sees A): the rule for the game.
    pub fn add_view(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>) {
        self.add(level, x, y, radius, false);
    }

    /// As `add_view`, but open cells count as seen when a line reaches near
    /// any of their corners too, e.g. a side opening glimpsed from a long
    /// corridor. Not symmetric, so only for drawing what a viewer can make
    /// out, never for game rules.
    pub fn add_glimpses(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>) {
        self.add(level, x, y, radius, true);
    }

    fn add(&mut self, level: &Level, x: i64, y: i64, radius: Option<u32>, partly: bool) {
        // Nothing can be seen from inside a wall or outside the level.
        if !level.is_open(x, y) {
            return;
        }
        let from = (x * CELL_UNITS, y * CELL_UNITS);
        for cy in self.min_y..self.min_y + self.height {
            for cx in self.min_x..self.min_x + self.width {
                let (dx, dy) = (cx - x, cy - y);
                let in_reach = radius.is_none_or(|r| dx * dx + dy * dy <= i64::from(r).pow(2));
                if !in_reach || !level.is_open(cx, cy) || self.is_seen(cx, cy) {
                    continue;
                }
                let centre = (cx * CELL_UNITS, cy * CELL_UNITS);
                let seen = clear_line(level, from, centre)
                    || (partly
                        && GLIMPSE_POINTS.iter().any(|(ox, oy)| {
                            clear_line(level, from, (centre.0 + ox, centre.1 + oy))
                        }));
                if seen {
                    self.reveal(level, cx, cy);
                }
            }
        }
    }

    /// Counts an open cell as seen, and the walls touching it.
    pub fn reveal(&mut self, level: &Level, x: i64, y: i64) {
        self.mark(x, y);
        for ny in y - 1..=y + 1 {
            for nx in x - 1..=x + 1 {
                if !level.is_open(nx, ny) {
                    self.mark(nx, ny);
                }
            }
        }
    }
}

/// The points near an open cell's corners that count for glimpses, in units
/// from its centre.
pub(super) const GLIMPSE_POINTS: [(i64, i64); 4] = [(-16, -16), (16, -16), (-16, 16), (16, 16)];
