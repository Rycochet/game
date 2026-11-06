//! The terrain at the world map points round some cells, worked out once and
//! blended across the cells between them.

use super::{SCALE, Wilderness};
use crate::climate::SNOW_TEMPERATURE;

/// What the wilderness needs to know of one world map point.
#[derive(Clone, Copy, Default)]
pub(super) struct PointSample {
    pub height: f64,
    /// Sea frozen hard enough to walk on.
    pub frozen: bool,
    /// Trees and low growth (`vegetation::Cover`), none on water or snow.
    pub trees: f64,
    pub growth: f64,
}

/// Samples of the points covering a rectangle of cells, by point.
pub(super) struct Samples {
    min_px: i64,
    min_py: i64,
    columns: i64,
    samples: Vec<PointSample>,
}

impl Samples {
    /// The points needed to blend every cell of `(min_x, min_y, width, height)`
    /// (unwrapped: they may run past the planet's seam).
    pub fn new(wild: &Wilderness, (min_x, min_y, width, height): (i64, i64, i64, i64)) -> Self {
        let min_px = min_x.div_euclid(SCALE);
        let min_py = min_y.div_euclid(SCALE);
        let columns = (min_x + width - 1).div_euclid(SCALE) + 2 - min_px;
        let rows = (min_y + height - 1).div_euclid(SCALE) + 2 - min_py;
        let mut samples = Vec::with_capacity((columns * rows) as usize);
        for py in min_py..min_py + rows {
            for px in min_px..min_px + columns {
                samples.push(wild.sample_point(px.rem_euclid(wild.points), py));
            }
        }
        Self {
            min_px,
            min_py,
            columns,
            samples,
        }
    }

    fn at(&self, px: i64, py: i64) -> &PointSample {
        let index = (py - self.min_py) * self.columns + px - self.min_px;
        &self.samples[index as usize]
    }

    /// A value blended between the four points round a cell's middle (points
    /// stand on the corners of their `SCALE` cells).
    fn blend(&self, x: i64, y: i64, value: impl Fn(&PointSample) -> f64) -> f64 {
        let (px, py) = (x.div_euclid(SCALE), y.div_euclid(SCALE));
        // Exact in binary: (0.5..15.5) / 16.
        let tx = (x.rem_euclid(SCALE) as f64 + 0.5) / SCALE as f64;
        let ty = (y.rem_euclid(SCALE) as f64 + 0.5) / SCALE as f64;
        let north = value(self.at(px, py)) * (1.0 - tx) + value(self.at(px + 1, py)) * tx;
        let south = value(self.at(px, py + 1)) * (1.0 - tx) + value(self.at(px + 1, py + 1)) * tx;
        north * (1.0 - ty) + south * ty
    }

    /// The ground's height at a cell.
    pub fn height(&self, x: i64, y: i64) -> f64 {
        self.blend(x, y, |sample| sample.height)
    }

    /// Whether the sea at a cell is frozen: the nearest point's.
    pub fn frozen(&self, x: i64, y: i64) -> bool {
        let nearest = |c: i64| (c + SCALE / 2).div_euclid(SCALE);
        self.at(nearest(x), nearest(y)).frozen
    }

    /// How wooded a cell is and how much low growth it has, `0..=1`.
    pub fn cover(&self, x: i64, y: i64) -> (f64, f64) {
        (
            self.blend(x, y, |sample| sample.trees),
            self.blend(x, y, |sample| sample.growth),
        )
    }
}

impl Wilderness {
    /// One point's sample, `px` in `0..points`.
    pub(super) fn sample_point(&self, px: i64, py: i64) -> PointSample {
        let surface = self.surface.sample(px, py);
        let climate = self.climate.sample(px, py, surface.height);
        let land = surface.height >= self.low_water;
        let mut sample = PointSample {
            height: surface.height,
            frozen: !land && surface.is_frozen(self.low_water, climate),
            ..PointSample::default()
        };
        if land && climate.temperature >= SNOW_TEMPERATURE {
            let cover = self.vegetation.cover(px, py, surface.height, climate);
            (sample.trees, sample.growth) = (cover.trees as f64, cover.growth as f64);
        }
        sample
    }
}
