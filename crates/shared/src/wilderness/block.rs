//! Making a block of the wilderness, cell by cell.

use super::course::Segment;
use super::samples::Samples;
use super::water::Water;
use super::{
    BlockPos, CLEARED, CLEARED_TO, ChunkPlan, MOST_TREES_PERCENT, RIVER_CLEARED, RIVER_CLEARED_TO,
    ROAD_CLEARED, ROAD_CLEARED_TO, SHALLOWS_REACH, Wilderness,
};
use crate::dungeon::{Cell, Level};
use crate::movement::past_the_ice;
use crate::rng::Seed;
use crate::road::RoadKind;

/// Rolls are out of this many.
const ROLL_OUT_OF: u64 = 1_000_000;

impl Wilderness {
    /// A block's cells. `plan` is the plan of the chunk the block is in
    /// (`chunk_of` any of its cells).
    pub fn block(&self, block: BlockPos, plan: &ChunkPlan) -> Level {
        self.cells(self.block_bounds(block), plan)
    }

    /// The cells of a rectangle inside one chunk, whose plan `plan` is.
    pub(super) fn cells(&self, bounds: (i64, i64, i64, i64), plan: &ChunkPlan) -> Level {
        let (min_x, min_y, width, height) = bounds;
        debug_assert_eq!(plan.chunk(), self.chunk_of(min_x, min_y));
        let around = (
            min_x - SHALLOWS_REACH,
            min_y - SHALLOWS_REACH,
            width + 2 * SHALLOWS_REACH,
            height + 2 * SHALLOWS_REACH,
        );
        let samples = Samples::new(self, around);
        let margin = (SHALLOWS_REACH as f64)
            .max(ROAD_CLEARED_TO)
            .max(RIVER_CLEARED_TO);
        let rivers: Vec<Segment> = plan
            .rivers
            .iter()
            .filter(|river| river.near(bounds, margin))
            .copied()
            .collect();
        let roads: Vec<(Segment, RoadKind)> = plan
            .roads
            .iter()
            .filter(|(road, _)| road.near(bounds, margin))
            .copied()
            .collect();
        let water = Water::new(&samples, self.low_water, &rivers, bounds);
        let rolls = Rolls {
            trees: self.seed.derive("trees"),
            growth: self.seed.derive("growth"),
        };
        let mut level = Level::new(min_x, min_y, width, height);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if let Some(cutout) = plan.cutout().filter(|cutout| cutout.contains(x, y)) {
                    level.set(x, y, cutout.level().cell(x, y));
                    level.set_room(x, y, cutout.level().room(x, y));
                    continue;
                }
                let road = roads
                    .iter()
                    .filter(|(road, _)| road.outside(x, y) <= 0.0)
                    .map(|&(_, kind)| kind)
                    .max();
                let cell = if past_the_ice(self.points, y.div_euclid(super::SCALE)) {
                    Cell::Wall
                } else if let Some(kind) = road {
                    if water.is_wet(x, y) {
                        Cell::Bridge
                    } else {
                        road_surface(kind)
                    }
                } else if water.is_wet(x, y) {
                    if water.is_shallow(x, y) {
                        Cell::Shallows
                    } else {
                        Cell::Water
                    }
                } else {
                    let (trees, growth) = samples.cover(x, y);
                    let open = clearing(plan, x, y)
                        * courses_clear(&rivers, x, y, RIVER_CLEARED, RIVER_CLEARED_TO)
                        * courses_clear(
                            roads.iter().map(|(road, _)| road),
                            x,
                            y,
                            ROAD_CLEARED,
                            ROAD_CLEARED_TO,
                        );
                    let trees = trees * open * MOST_TREES_PERCENT as f64 / 100.0;
                    rolls.ground(self.wrap(x), y, trees, growth)
                };
                level.set(x, y, cell);
            }
        }
        level
    }
}

/// How much of the land's own tree cover a cell keeps near its chunk's
/// cutout: none within `CLEARED`, all from `CLEARED_TO`, evenly between.
fn clearing(plan: &ChunkPlan, x: i64, y: i64) -> f64 {
    let Some(cutout) = plan.cutout() else {
        return 1.0;
    };
    let squared = cutout.distance_squared(x, y);
    if squared < CLEARED * CLEARED {
        0.0
    } else if squared >= CLEARED_TO * CLEARED_TO {
        1.0
    } else {
        ((squared as f64).sqrt() - CLEARED as f64) / (CLEARED_TO - CLEARED) as f64
    }
}

/// How much of the land's own tree cover a cell keeps near rivers or roads:
/// none within `cleared` of an edge, all from `cleared_to`, evenly between.
fn courses_clear<'a>(
    courses: impl IntoIterator<Item = &'a Segment>,
    x: i64,
    y: i64,
    cleared: f64,
    cleared_to: f64,
) -> f64 {
    let nearest = courses
        .into_iter()
        .map(|course| course.outside(x, y))
        .fold(f64::INFINITY, f64::min);
    ((nearest - cleared) / (cleared_to - cleared)).clamp(0.0, 1.0)
}

/// What a road is surfaced with in the wilderness, by its kind.
fn road_surface(kind: RoadKind) -> Cell {
    match kind {
        RoadKind::Path => Cell::DirtTrack,
        RoadKind::Track => Cell::GravelTrack,
        RoadKind::Road => Cell::Cobblestone,
        RoadKind::Highway => Cell::Flagstones,
    }
}

/// The seeds a cell's ground is rolled from, by its position alone.
struct Rolls {
    trees: Seed,
    growth: Seed,
}

impl Rolls {
    /// Dry land: a tree with chance `trees`, else grass with chance `growth`.
    fn ground(&self, x: i64, y: i64, trees: f64, growth: f64) -> Cell {
        let roll = |seed: Seed| (seed.at(&[x, y]) % ROLL_OUT_OF) as f64 / ROLL_OUT_OF as f64;
        if roll(self.trees) < trees {
            Cell::Tree
        } else if roll(self.growth) < growth {
            Cell::Grass
        } else {
            Cell::Ground
        }
    }
}
