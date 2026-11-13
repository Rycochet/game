//! Towns: roads with solid buildings along them, on the land outside.
//!
//! The middle of a town is a straight road, a T junction or a crossroads, with
//! a square over it in bigger towns. It grows a plot at a time: each new
//! building or garden goes beside one already there, more often an early one,
//! lined up with it, across the road from it, or round a corner on a new side
//! road, which gets likelier the longer a road has gone without one. A plot
//! with no room left beside it is taken off the list. Every building is solid,
//! with its door in a straight stretch of wall facing something walkable: a
//! road, except that houses may open onto any ground.
//!
//! What a building is, how it looks and its numbers come from hashes of the
//! seed and the building's number rather than the layout's random sequence, so
//! adding to them later doesn't move anything.

use super::SiteLevel;
use super::wall::WallMaterial;
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

// Split into files (`cargo xtask lint` keeps files small).
mod arms;
mod corner;
mod finish;
mod fits;
mod furnish;
mod grow;
mod kind;
mod looks;
mod middle;
mod odds;
mod place;
mod place_building;
mod plan;
mod plot;
mod road;
mod town_building;
mod wall;

use finish::rects_of;
pub use kind::BuildingKind;
pub use looks::{RoofKind, WallKind};
use odds::{ROADS, blend, pick};
pub use plan::{Plan, TownOptions};
use plot::{Beside, Frame, Placed, Plot};
use road::{Block, Road};
pub use town_building::{BuildingLook, Rect, TownBuilding};
pub(super) use wall::{CITY_WALLS, HAMLET_WALLS, TOWN_WALLS, VILLAGE_WALLS};

/// How wide the ground round the whole town is.
const BORDER: i64 = 3;

/// A town's smallest and largest number of buildings on which the size-based
/// weights blend from their small to their large values.
const SMALLEST: u64 = 3;
const LARGEST: u64 = 150;

/// Towns with fewer buildings than this have no side roads.
const SIDE_ROADS_FROM: i64 = 8;

/// Tiles of frontage along a road before a side road gets likelier, and how
/// much likelier for each one after.
const SIDE_ROAD_AFTER: i64 = 8;
const SIDE_ROAD_WEIGHT: u64 = 6;

/// The longest line of building that can be built on: anything joining it
/// must leave a gap first, so nobody has further than this to walk round.
const LONGEST_WITHOUT_GAP: i64 = 10;

/// How far apart parallel roads must be, in cells: room for a building.
const BETWEEN_ROADS: i64 = 3;

/// How far along a road to look for a free plot, past whatever is in the way.
const LOOK_ALONG: i64 = 8;

/// Houses behind other buildings start at towns this big, and make up at most
/// this share of the town, in percent.
const BEHIND_FROM: i64 = 15;
const BEHIND_PERCENT: i64 = 20;

/// The share of buildings that are houses, in percent, at least.
const HOUSE_PERCENT: i64 = 30;

/// One garden per this many plots, on average.
const PLOTS_PER_GARDEN: u64 = 25;

/// A garden's smallest side is the larger of 3 and buildings divided by this.
const BUILDINGS_PER_GARDEN_SIDE: i64 = 20;

/// Squares are possible from this many buildings, and certain from the next.
const SQUARE_POSSIBLE: i64 = 10;
const SQUARE_CERTAIN: i64 = 50;

/// A town being generated.
struct Town {
    seed: Seed,
    target: i64,
    rng: Rng,
    level: Level,
    placed: Vec<Placed>,
    roads: Vec<Road>,
    /// What can still have something put beside it, oldest first.
    plots: Vec<Plot>,
    /// The middle of the town, where growth starts.
    middle: (i64, i64),
    /// How many houses stand behind others.
    behind: i64,
    /// The bare ground in front of their doors, kept clear.
    door_fronts: Vec<(i64, i64)>,
    /// Which arms the middle has: north, east, south, west.
    arms: [bool; 4],
    /// What its wall is made of, if it has one.
    wall: Option<WallMaterial>,
}

impl Town {
    fn new(seed: Seed, target: i64, arms: [bool; 4], wall: Option<WallMaterial>) -> Self {
        // A generous area, cut down to what is used at the end.
        let span = (target * 90).isqrt().max(24) + 2 * BORDER;
        let mut level = Level::new(0, 0, span, span);
        for y in 0..span {
            for x in 0..span {
                level.set(x, y, Cell::Ground);
            }
        }
        Self {
            seed,
            target,
            rng: Seed::new(seed.derive("town").at(&[0])).rng(),
            level,
            placed: Vec::new(),
            roads: Vec::new(),
            plots: Vec::new(),
            middle: (span / 2, span / 2),
            behind: 0,
            door_fronts: Vec::new(),
            arms,
            wall,
        }
    }

    /// A road surface, by the size of the town, no better than `best` (an
    /// index into `ROADS`).
    fn road_kind(&mut self, best: usize) -> usize {
        let choices: Vec<(usize, u64)> = ROADS[..=best]
            .iter()
            .enumerate()
            .map(|(i, &(_, small, large))| (i, blend(small, large, self.target)))
            .collect();
        pick(&mut self.rng, &choices)
    }

    fn paint(&mut self, area: Block, cell: Cell) {
        for y in area.y0..area.y1 {
            for x in area.x0..area.x1 {
                self.level.set(x, y, cell);
            }
        }
    }

    fn generate(mut self) -> SiteLevel {
        let (arms, middle, main) = self.lay_out();
        self.finish(arms, middle, main)
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wall_tests;
