//! Rivers and roads at wilderness scale: their world map points as cells
//! (each moved a little by a hash of the point, so courses wander rather than
//! run in 16-cell straights), joined into segments with a half width at each
//! end.

use super::{SCALE, Wilderness};
use crate::river::River;
use crate::rng::Seed;
use crate::road::{Road, RoadKind};

/// How far a river's points may be moved, each way, in cells.
const RIVER_BEND: i64 = 5;

/// How far a road's points may be moved, each way, in cells.
const ROAD_BEND: i64 = 3;

/// A river's half width at its source, in cells.
const RIVER_SOURCE_HALF: f64 = 1.0;

/// It widens by a cell each side over this many world map steps.
const RIVER_WIDENS_OVER: f64 = 16.0;

/// Its widest, half across: 17 cells, so where it is widest the middle is
/// out of `SHALLOWS_REACH` of either bank (owner: up to 16 wide can be waded).
const RIVER_MOST_HALF: f64 = 8.5;

/// A stretch of a river or road between two of its points, in cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Segment {
    pub(super) from: (f64, f64),
    pub(super) to: (f64, f64),
    /// Half its width at each end.
    pub(super) half: (f64, f64),
}

impl Segment {
    /// How far a cell's middle is outside its edge, in cells (negative inside).
    pub fn outside(&self, x: i64, y: i64) -> f64 {
        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
        let (ax, ay) = self.from;
        let (dx, dy) = (self.to.0 - ax, self.to.1 - ay);
        let length = dx * dx + dy * dy;
        let along = if length > 0.0 {
            (((px - ax) * dx + (py - ay) * dy) / length).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let (cx, cy) = (ax + dx * along - px, ay + dy * along - py);
        (cx * cx + cy * cy).sqrt() - (self.half.0 + (self.half.1 - self.half.0) * along)
    }

    /// Whether it comes within `margin` of a rectangle of cells.
    pub fn near(&self, (min_x, min_y, width, height): (i64, i64, i64, i64), margin: f64) -> bool {
        let reach = self.half.0.max(self.half.1) + margin;
        let low = (
            self.from.0.min(self.to.0) - reach,
            self.from.1.min(self.to.1) - reach,
        );
        let high = (
            self.from.0.max(self.to.0) + reach,
            self.from.1.max(self.to.1) + reach,
        );
        low.0 < (min_x + width) as f64
            && high.0 > min_x as f64
            && low.1 < (min_y + height) as f64
            && high.1 > min_y as f64
    }
}

/// How wide a road is, half across, in cells.
pub(super) fn road_half(kind: RoadKind) -> f64 {
    match kind {
        RoadKind::Path => 0.6,
        RoadKind::Track => 1.1,
        RoadKind::Road => 1.6,
        RoadKind::Highway => 2.1,
    }
}

impl Wilderness {
    /// A river's segments, its points moved `shift` world map points east.
    pub(super) fn river_segments(&self, river: &River, shift: i64) -> Vec<Segment> {
        let half = |step: usize| {
            (RIVER_SOURCE_HALF + step as f64 / RIVER_WIDENS_OVER).min(RIVER_MOST_HALF)
        };
        let seed = self.seed.derive("river bend");
        self.segments(&river.points, shift, seed, RIVER_BEND, half)
    }

    /// A road's segments, as `river_segments`.
    pub(super) fn road_segments(&self, road: &Road, shift: i64) -> Vec<Segment> {
        let half = road_half(road.kind);
        let seed = self.seed.derive("road bend");
        self.segments(&road.points, shift, seed, ROAD_BEND, |_| half)
    }

    fn segments(
        &self,
        points: &[(i64, i64)],
        shift: i64,
        seed: Seed,
        bend: i64,
        half: impl Fn(usize) -> f64,
    ) -> Vec<Segment> {
        let cell = |&(px, py): &(i64, i64)| {
            // By where the point is round the planet, so every copy agrees.
            let hash = seed.at(&[px.rem_euclid(self.points), py]);
            let span = (2 * bend + 1) as u64;
            let dx = (hash % span) as i64 - bend;
            let dy = ((hash / span) % span) as i64 - bend;
            let (x, y) = (
                (px + shift) * SCALE + SCALE / 2 + dx,
                py * SCALE + SCALE / 2 + dy,
            );
            (x as f64, y as f64)
        };
        points
            .windows(2)
            .enumerate()
            .map(|(step, pair)| Segment {
                from: cell(&pair[0]),
                to: cell(&pair[1]),
                half: (half(step), half(step + 1)),
            })
            .collect()
    }
}
