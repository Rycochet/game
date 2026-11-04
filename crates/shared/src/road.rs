//! Roads on the world map, between towns. Like rivers they come from the
//! world alone, so the order anything is looked at in doesn't matter; unlike
//! rivers they run between pois, which come from the secret seed, so only the
//! server works them out and tells clients of them. See
//! `docs/context/shared/road.md`.
//!
//! Two towns within `RANGE` of each other are joined unless a third town is
//! nearer to both (a "relative neighbourhood graph"), and the road between
//! them follows the walking rules (`route`), with larger roads minding slopes
//! less.

use serde::{Deserialize, Serialize};

use crate::movement::{WorldConditions, WorldGround};
use crate::poi::{CHUNK_SIZE, ChunkPos, PoiMap};
use crate::world::WorldParams;

// Split into files (`cargo xtask lint` keeps files small).
mod kind;
mod network;
mod route;
mod steps;

pub use kind::{FASTEST_PERCENT, RoadKind};
pub use steps::RoadSteps;

/// The furthest apart two towns may be and still be joined, in grid points:
/// 6 chunks (owner's choice).
pub const RANGE: i64 = 6 * CHUNK_SIZE;

/// How far a road may stray outside the box its two towns make, in grid
/// points: the most a detour round a hill or a bay can take.
pub const MARGIN: i64 = CHUNK_SIZE;

/// Bump whenever the roads made from a world change, so saved roads from
/// before are worked out again.
pub const ROAD_VERSION: u32 = 3;

/// Which two towns a road joins, by their chunks, the smaller first, so each
/// pair has one name however it was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoadLink {
    pub a: ChunkPos,
    pub b: ChunkPos,
}

impl RoadLink {
    pub fn new(one: ChunkPos, other: ChunkPos) -> Self {
        Self {
            a: one.min(other),
            b: one.max(other),
        }
    }
}

/// A road from town `a` to town `b` of its link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Road {
    pub link: RoadLink,
    pub kind: RoadKind,
    /// World grid points one step apart, from one town's poi to the other's.
    /// `x` starts at town `a`'s, within the first planet width, and carries on
    /// past the planet's edges without wrapping (as a river's does).
    pub points: Vec<(i64, i64)>,
}

impl Road {
    /// The smallest box holding every point, `(min, max)`, in the points' frame.
    pub fn bounds(&self) -> ((i64, i64), (i64, i64)) {
        let mut min = (i64::MAX, i64::MAX);
        let mut max = (i64::MIN, i64::MIN);
        for &(x, y) in &self.points {
            min = (min.0.min(x), min.1.min(y));
            max = (max.0.max(x), max.1.max(y));
        }
        (min, max)
    }
}

/// Works out one world's roads.
pub struct Roads {
    ground: WorldGround,
    /// Roads are laid as if at high tide and in no frost, so the tide and the
    /// weather never cut one.
    conditions: WorldConditions,
}

impl Roads {
    pub fn new(world: &WorldParams) -> Self {
        Self {
            ground: WorldGround::new(world),
            conditions: WorldConditions {
                water: world.high_water(),
                snow_line: f64::INFINITY,
            },
        }
    }

    /// Every pair of towns joined by a road that may pass through the area
    /// `min..=max` (world grid points, in any planet-width frame), sorted, with
    /// what each road will be.
    pub fn links_in_area(
        &self,
        pois: &PoiMap,
        min: (i64, i64),
        max: (i64, i64),
    ) -> Vec<(RoadLink, RoadKind)> {
        network::links_in_area(pois, min, max)
    }

    /// The road between a link's two towns (from `links_in_area`), if they can
    /// be joined over land: `None` with water between them.
    pub fn road(&self, pois: &PoiMap, link: RoadLink) -> Option<Road> {
        let (a, b) = (pois.in_chunk(link.a)?, pois.in_chunk(link.b)?);
        let kind = RoadKind::between(a.kind, b.kind)?;
        // Town `b` as near `a` as it can be, round the planet either way.
        let width = pois.width();
        let mut east = (b.x - a.x).rem_euclid(width);
        if east > width / 2 {
            east -= width;
        }
        let to = (a.x + east, b.y);
        let points = route::route(&self.ground, self.conditions, (a.x, a.y), to, kind)?;
        Some(Road { link, kind, points })
    }
}

#[cfg(test)]
mod tests;
