//! The ground's shape on a level: a height at every grid corner, in whole
//! units (`UNITS_PER_CELL` across a cell, as positions will be), with the
//! surface between them four flat triangles per cell meeting at its middle
//! (the average of its corners). Both sides work it out the same way: the
//! server tells which kind of ground a level has (`LevelGround`), and
//! `Relief::of` turns that into heights.
//!
//! - The wilderness, and a site's first level when it is set into the
//!   wilderness whole, follow the world map's heights (`movement::WildHeights`),
//!   with pois' covered ground kept level (`FlatGround`).
//! - Caves undulate gently by a hash of position, nearly flat inside their
//!   caves (cells with a `RoomId`), level round pools and stairs.
//! - Everything else (buildings, the Hall) is flat.
//!
//! Stairs cells are always level.

mod cave;
mod level_ground;

use crate::movement::WildHeights;

pub use cave::{CAVE_SWELL, ROOM_SWELL};
pub use level_ground::LevelGround;

/// Height units across a cell (positions use the same: 240 a cell).
pub const UNITS_PER_CELL: i32 = 240;

/// Height units in a 16th of an elevation (`WildHeights::at`): an eighth of a
/// cell (owner, 2026-11-08, after playing with raised cells).
pub const UNITS_PER_16TH: i32 = 30;

/// Half a cell, in units: how far a cell's corners are from its middle.
const HALF: i64 = UNITS_PER_CELL as i64 / 2;

/// The heights of the corners of a rectangle of cells. Corner `(x, y)` is the
/// north-west corner of cell `(x, y)`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Relief {
    /// The cells it covers: `(min_x, min_y, width, height)`.
    bounds: (i64, i64, i64, i64),
    /// Each corner's height, row by row, `width + 1` to a row.
    corners: Vec<i32>,
}

impl Relief {
    /// Level ground at height 0 over `bounds`.
    pub fn flat(bounds: (i64, i64, i64, i64)) -> Self {
        Self::from_fn(bounds, |_, _| 0)
    }

    /// Each corner's height from `height(x, y)`.
    pub fn from_fn(bounds: (i64, i64, i64, i64), height: impl Fn(i64, i64) -> i32) -> Self {
        let (min_x, min_y, width, rows) = bounds;
        let mut corners = Vec::with_capacity(((width + 1) * (rows + 1)).max(0) as usize);
        for y in min_y..=min_y + rows {
            for x in min_x..=min_x + width {
                corners.push(height(x, y));
            }
        }
        Self { bounds, corners }
    }

    /// The land's heights over `bounds` (cells of the level), read from
    /// `heights` at each cell plus `offset` (a site's first level set into
    /// the wilderness; `(0, 0)` for the wilderness itself), with each of
    /// `stairs`' cells kept level.
    pub fn of_land(
        heights: &WildHeights,
        bounds: (i64, i64, i64, i64),
        (ox, oy): (i64, i64),
        stairs: &[(i64, i64)],
    ) -> Self {
        let mut relief = Self::from_fn(bounds, |x, y| heights.corner(x + ox, y + oy));
        relief.level_stairs(stairs);
        relief
    }

    /// The cells it covers.
    pub fn bounds(&self) -> (i64, i64, i64, i64) {
        self.bounds
    }

    /// Whether a cell is in it.
    pub fn covers(&self, x: i64, y: i64) -> bool {
        let (min_x, min_y, width, height) = self.bounds;
        (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y)
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (min_x, min_y, width, height) = self.bounds;
        let inside = (min_x..=min_x + width).contains(&x) && (min_y..=min_y + height).contains(&y);
        inside.then(|| ((y - min_y) * (width + 1) + x - min_x) as usize)
    }

    /// The height of corner `(x, y)` (cell `(x, y)`'s north-west corner): 0
    /// beyond the rectangle.
    pub fn corner(&self, x: i64, y: i64) -> i32 {
        self.index(x, y).map_or(0, |i| self.corners[i])
    }

    fn set_corner(&mut self, x: i64, y: i64, height: i32) {
        if let Some(i) = self.index(x, y) {
            self.corners[i] = height;
        }
    }

    /// Cell `(x, y)`'s four corners: north-west, north-east, south-west,
    /// south-east.
    pub fn corners_of(&self, x: i64, y: i64) -> [i32; 4] {
        [
            self.corner(x, y),
            self.corner(x + 1, y),
            self.corner(x, y + 1),
            self.corner(x + 1, y + 1),
        ]
    }

    /// The height at a cell's middle: the average of its corners, rounded.
    pub fn centre(&self, x: i64, y: i64) -> i32 {
        let sum: i64 = self.corners_of(x, y).iter().map(|&h| i64::from(h)).sum();
        (sum + 2).div_euclid(4) as i32
    }

    /// The height at a point, in units from cell `(0, 0)`'s middle (cell
    /// `(x, y)`'s middle is at `(x, y) × UNITS_PER_CELL`), on the triangle
    /// of its cell it lies on.
    pub fn at(&self, ux: i64, uy: i64) -> i32 {
        let cell = |u: i64| (u + HALF).div_euclid(2 * HALF);
        let (x, y) = (cell(ux), cell(uy));
        let (dx, dy) = (ux - x * 2 * HALF, uy - y * 2 * HALF);
        let [nw, ne, sw, se] = self.corners_of(x, y).map(i64::from);
        let c = i64::from(self.centre(x, y));
        // Twice the height times `HALF`, on the triangle towards the side the
        // point is nearest: along towards that side, and across it.
        let twice = if dx.abs() >= dy.abs() {
            let (a, b) = if dx >= 0 { (ne, se) } else { (nw, sw) };
            2 * HALF * c + dx.abs() * (a + b - 2 * c) + dy * (b - a)
        } else {
            let (a, b) = if dy >= 0 { (sw, se) } else { (nw, ne) };
            2 * HALF * c + dy.abs() * (a + b - 2 * c) + dx * (b - a)
        };
        (twice + HALF).div_euclid(2 * HALF) as i32
    }

    /// Makes each cell level at the average of its corners (stairs).
    pub fn level_stairs(&mut self, cells: &[(i64, i64)]) {
        for &(x, y) in cells {
            let height = self.centre(x, y);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                self.set_corner(x + dx, y + dy, height);
            }
        }
    }
}

#[cfg(test)]
mod tests;
