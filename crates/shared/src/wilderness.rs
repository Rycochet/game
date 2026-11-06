//! The wilderness: the land between pois at walking scale, one endless map per
//! world, `SCALE` cells to a world map point and wrapping east–west like it.
//! Made on demand in `BLOCK_SIZE` blocks from the public terrain (height,
//! climate, vegetation), with each chunk's poi set into it as a cutout of its
//! site's first level. A block nobody has changed can always be made again, so
//! only changes need keeping.

mod block;
mod course;
mod cutout;
mod gate;
mod plan;
mod samples;
mod scale;
mod water;

#[cfg(test)]
mod course_tests;
#[cfg(test)]
mod tests;

pub use cutout::Cutout;
pub use gate::{GATE_PAD, GateSpot};
pub use plan::{ChunkPlan, gates_of};
pub use scale::{BLOCK_SIZE, BlockPos, SCALE};

use crate::climate::Climate;
use crate::river::Rivers;
use crate::rng::Seed;
use crate::surface::Surface;
use crate::vegetation::Vegetation;
use crate::world::WorldParams;

/// The wilderness generator's version (see "Generator versions" in
/// `architecture.md`): bump it when what a block holds changes in a way that
/// matters (not for looks).
pub const WILDERNESS_VERSION: u32 = 1 + crate::site::FORGET_BEFORE;

/// How far from land water stays shallow enough to wade, in cells (owner): a
/// river up to twice this wide can be crossed.
pub const SHALLOWS_REACH: i64 = 8;

/// The share of cells with a tree in the thickest woods, in percent: low
/// enough that trees never wall off more than the odd cell.
pub const MOST_TREES_PERCENT: u64 = 35;

/// No tree grows within this many cells of a cutout (owner).
pub const CLEARED: i64 = 8;

/// Trees are back to the land's own cover this many cells from a cutout,
/// thinning in between (owner): a clearing no bigger than woods' own gaps.
pub const CLEARED_TO: i64 = 24;

/// No tree grows within this many cells of a river's edge, and trees are
/// back to the land's own this far from it.
pub const RIVER_CLEARED: f64 = 1.0;
pub const RIVER_CLEARED_TO: f64 = 4.0;

/// The same for roads.
pub const ROAD_CLEARED: f64 = 1.0;
pub const ROAD_CLEARED_TO: f64 = 6.0;

/// How far round a way down into a site (a cave entered by a hole) the ground
/// is clear, in cells.
pub const ENTRANCE_CLEARING: i64 = 2;

/// Works out the wilderness of one world.
pub struct Wilderness {
    surface: Surface,
    climate: Climate,
    vegetation: Vegetation,
    rivers: Rivers,
    seed: Seed,
    /// The lowest the tide goes: below it is sea in the wilderness, for now.
    low_water: f64,
    /// World map points round the planet.
    points: i64,
}

impl Wilderness {
    pub fn new(world: &WorldParams) -> Self {
        let surface = Surface::new(world);
        Self {
            points: surface.width(),
            surface,
            climate: Climate::new(world),
            vegetation: Vegetation::new(world),
            rivers: Rivers::new(world),
            seed: world.seed.derive("wilderness"),
            low_water: world.sea_level - world.tide_reach(),
        }
    }

    /// Cells round the planet: `x` wraps at this.
    pub fn width(&self) -> i64 {
        self.points * SCALE
    }

    /// World map points round the planet.
    pub fn points(&self) -> i64 {
        self.points
    }

    /// The biome at the middle of a block, for who lives there.
    pub fn biome_of(&self, block: BlockPos) -> crate::climate::Biome {
        let (x, y, width, height) = self.block_bounds(block);
        let (px, py) = self.point_of(x + width / 2, y + height / 2);
        let height = self.surface.sample(px, py).height;
        self.climate.sample(px, py, height).biome()
    }
}
