//! Caves (and dug dungeons such as crypts): irregular caves joined by
//! winding passages, one or more levels deep.
//!
//! Each cave is dug by a random walk inside an ellipse ("Random Walk Cave
//! Generation" on RogueBasin), smoothed once with the cellular automata rule
//! ("Cellular Automata Method for Generating Random Cave-Like Levels"), so
//! no cave is a rectangle. Passages join each cave to the nearest one
//! already joined (a spanning tree), plus a few more for loops; they run in
//! short straight stretches that turn as they go. Pools of water or lava are
//! only kept where everything can still be reached around them.
//!
//! The way in is a hole with stairs down from the world map, or a cave mouth:
//! a strip of outside ground (as wide as the mouth, not the cave) on the
//! south edge, sometimes another on the east or west; since `CAVE_VERSION`
//! that level is then a hill of rock round its caves (`outcrop`), its mouths'
//! passages running out through it. Every level's size is its own, unrelated
//! to the levels above and below.

// Split into files (`cargo xtask lint` keeps files small).
mod dig;
mod doors;
mod join;
mod level;
mod mouths;
mod options;
mod outcrop;
mod place;
mod plan;
mod pool;

use dig::dig_cave;
use doors::add_doors;
use join::{join_caves, nearest_reached, wind};
use level::open_skylights;
use mouths::{Mouth, dig_mouths, furthest_cave};
pub use options::{CaveEntrance, CaveOptions};
use place::{Cave, place_caves};
pub use plan::Plan;
use pool::add_pool;

/// Depth of the outside ground at a cave mouth, in cells.
const BORDER: i64 = 3;

/// Caves per this many cells of a level (at least 3, at most 24).
const CELLS_PER_CAVE: i64 = 220;

/// Cave radii, each way, in cells.
const MIN_RADIUS: i64 = 3;
const MAX_RADIUS: i64 = 8;

/// Percentage of a cave's ellipse dug before smoothing, and how many times
/// it is smoothed.
const FILL_PERCENT: u32 = 60;
const SMOOTHING_PASSES: u32 = 3;

/// Gap kept between caves' bounding boxes.
const CAVE_GAP: i64 = 3;

/// Passage stretches run this many cells before they may turn.
const MIN_RUN: i64 = 2;
const MAX_RUN: i64 = 6;

/// Percentage of passage stretches that wander sideways.
const WANDER_PERCENT: u32 = 30;

/// Percentage chance of a door where a passage enters a cave (with doors).
const DOOR_PERCENT: u32 = 40;

/// Percentage of doors that become secret, where nothing depends on them.
const SECRET_DOOR_PERCENT: u32 = 10;

/// Percentage chance of a second mouth on the east or west side.
const SECOND_MOUTH_PERCENT: u32 = 35;

/// One floor cell in this many of a natural cave's top level is open to the
/// sky, and how far (counting diagonals as 1) one stays from liquid and stairs.
const CELLS_PER_SKYLIGHT: u64 = 1500;
const SKYLIGHT_CLEARANCE: i64 = 2;
