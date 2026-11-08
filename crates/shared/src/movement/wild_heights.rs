//! How high wilderness cells count as for walking: the world map's elevations
//! (`WorldGround::elevation`, 25 m each) at the points on the corners of each
//! point's 16 × 16 cells, blended across them, in 16ths of an elevation. A
//! climb across 16 cells then costs and teaches what one world map step of the
//! same height does. The sea is flat at the lowest tide, where the wilderness
//! keeps it.

use super::{ELEVATIONS_PER_HEIGHT, FlatGround, Gait, SLOPE_COST, WorldGround, step_cost};

/// Cells to a world map point (`wilderness::SCALE`, which depends on this
/// module, so it is repeated here; a test checks they agree).
pub(crate) const CELLS_PER_POINT: i64 = 16;

/// The heights of a rectangle of wilderness cells, from the points round it,
/// each sampled once (sampling is the slow part).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WildHeights {
    min_px: i64,
    min_py: i64,
    columns: i64,
    rows: i64,
    /// Each point's elevation, in 16ths, row by row.
    points: Vec<i32>,
    /// The sea's height, in 16ths: nothing is lower.
    floor: i32,
    /// Ground kept level (cutouts), which wins over the slopes.
    flats: Vec<FlatGround>,
}

impl WildHeights {
    /// The heights of every cell in `(min_x, min_y, width, height)` (which may
    /// run past the planet's seam: the points wrap).
    pub fn new(ground: &WorldGround, (min_x, min_y, width, height): (i64, i64, i64, i64)) -> Self {
        let min_px = min_x.div_euclid(CELLS_PER_POINT);
        let min_py = min_y.div_euclid(CELLS_PER_POINT);
        let columns = (min_x + width.max(1) - 1).div_euclid(CELLS_PER_POINT) + 2 - min_px;
        let rows = (min_y + height.max(1) - 1).div_euclid(CELLS_PER_POINT) + 2 - min_py;
        let mut points = Vec::with_capacity((columns * rows) as usize);
        for py in min_py..min_py + rows {
            for px in min_px..min_px + columns {
                points.push(ground.broad_elevation(px, py) * CELLS_PER_POINT as i32);
            }
        }
        Self {
            min_px,
            min_py,
            columns,
            rows,
            points,
            floor: ground.low_water_elevation() * CELLS_PER_POINT as i32,
            flats: Vec::new(),
        }
    }

    /// The same, with `flats` kept level (the server's cutouts in its window,
    /// or those the client has been told of).
    pub fn with_flats(mut self, flats: Vec<FlatGround>) -> Self {
        self.flats = flats;
        self
    }

    /// Whether a cell's height is known here.
    pub fn covers(&self, x: i64, y: i64) -> bool {
        let (px, py) = (x.div_euclid(CELLS_PER_POINT), y.div_euclid(CELLS_PER_POINT));
        (self.min_px..self.min_px + self.columns - 1).contains(&px)
            && (self.min_py..self.min_py + self.rows - 1).contains(&py)
    }

    fn point(&self, px: i64, py: i64) -> i64 {
        let index = (py - self.min_py) * self.columns + px - self.min_px;
        i64::from(self.points[index as usize])
    }

    /// A cell's height, in 16ths of an elevation: blended between the four
    /// points round its middle, as the wilderness blends its ground, and never
    /// below the sea; a flat's own height in one. 0 for a cell outside the
    /// rectangle.
    pub fn at(&self, x: i64, y: i64) -> i32 {
        if !self.covers(x, y) {
            return 0;
        }
        if let Some(flat) = self.flats.iter().find(|flat| flat.contains(x, y)) {
            return flat.height;
        }
        let (px, py) = (x.div_euclid(CELLS_PER_POINT), y.div_euclid(CELLS_PER_POINT));
        // The middle is (0.5..15.5) / 16 of the way across: in 32nds, odd.
        let east = 2 * x.rem_euclid(CELLS_PER_POINT) + 1;
        let south = 2 * y.rem_euclid(CELLS_PER_POINT) + 1;
        let across = 2 * CELLS_PER_POINT;
        let north_row = self.point(px, py) * (across - east) + self.point(px + 1, py) * east;
        let south_row =
            self.point(px, py + 1) * (across - east) + self.point(px + 1, py + 1) * east;
        let blended =
            (north_row * (across - south) + south_row * south).div_euclid(across * across);
        (blended as i32).max(self.floor)
    }
}

impl WorldGround {
    /// A point's elevation whatever covers it, water or not: what the
    /// wilderness's slopes are blended from.
    pub(crate) fn broad_elevation(&self, px: i64, py: i64) -> i32 {
        let px = px.rem_euclid(self.width.max(1));
        (self.surface.broad_height(px, py) * ELEVATIONS_PER_HEIGHT).floor() as i32
    }

    /// The lowest tide's elevation, where the wilderness's sea lies.
    pub(crate) fn low_water_elevation(&self) -> i32 {
        (self.low_water * ELEVATIONS_PER_HEIGHT).floor() as i32
    }
}

/// What a wilderness step costs, in `step_cost` units (a site's energy is
/// `energy::SITE_STEP_SCALE` times it), given the heights (`WildHeights::at`)
/// at each end: its distance plus `SLOPE_COST` per 16th of an elevation
/// climbed or descended, less by `gait`'s Climbing. Across 16 cells that comes
/// to the world map's slope for the same height, in proportion to the
/// distance.
pub fn wild_step_cost(from: (i64, i64), to: (i64, i64), heights: (i32, i32), gait: Gait) -> u32 {
    step_cost(from, to) + gait.climb(heights.0.abs_diff(heights.1) * SLOPE_COST)
}

#[cfg(test)]
mod tests;
