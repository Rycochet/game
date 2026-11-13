//! Buildings: an outline (rectangle, cut or rounded corners, circle) filled
//! with rooms and corridors by binary space partitioning (after RogueBasin's
//! "Basic BSP Dungeon generation"), one or more floors high.
//!
//! The space inside the outer wall is split again and again by one-cell
//! walls (or, near the top of the tree, by corridors with a wall each side);
//! each final part is a room that fills it completely, so rooms reach out to
//! their walls and no wall inside is more than a few cells thick. Rooms in a
//! corner cut by the outline follow the cut, keeping a wall round them.
//! Doors are then added until everything can be reached from the way in.
//!
//! Every floor has the same outline, the same size or smaller (shrunk the
//! same amount on every side, so its walls stand on the floor below). Bigger
//! buildings may have several staircases between two floors. Each is on the
//! same cell of both floors, chosen before either is
//! generated, and the splits avoid them, so each floor can be generated on
//! its own. A ruined building's floors depend on each other (see `ruin`), so
//! they are all made together at the start and kept.

// Split into files (`cargo xtask lint` keeps files small).
mod castle;
mod doors;
mod entrances;
mod fill;
mod floors;
mod options;
mod plan;
mod shape;
mod splitter;
mod walled;

use castle::{Castle, CastleSize};
pub use castle::{CastleOptions, GATE_TOWER_SIZE, PoiCastle, TOWER_SIZE, TowerSpot, tower_spots};
use doors::{add_doors, add_extra_doors, join_corridors};
use entrances::{add_entrances, between};
use fill::{add_windows, pillars};
pub use options::{BuildingOptions, Outline};
pub use plan::Plan;
pub(super) use shape::Shape;
use splitter::{Corridor, Rect, Splitter};
use walled::Walled;

/// Outside ground round the ground floor, in cells.
const BORDER: i64 = 3;

/// The smallest a room can be across, and in cells.
const MIN_ROOM: i64 = 3;
const MIN_ROOM_CELLS: i64 = 6;

/// Parts no bigger than this both ways may stop being split.
const MAX_ROOM: i64 = 9;
/// Percentage chance that a part small enough to be a room stops there.
const STOP_PERCENT: u32 = 40;

/// Splits along at least this many cells may be corridors, near the top of
/// the tree (depth below `CORRIDOR_DEPTH`), with this percentage chance.
const CORRIDOR_MIN_SPAN: i64 = 10;
const CORRIDOR_DEPTH: u32 = 3;
const CORRIDOR_PERCENT: u32 = 60;
/// Corridors along at least this many cells may be two wide.
const WIDE_CORRIDOR_SPAN: i64 = 24;

/// A floor narrower than this isn't built.
const MIN_FLOOR_SIZE: i64 = 9;

/// The smallest floor a tapering building has: a smallest room and its walls.
const MIN_TAPERED_FLOOR: i64 = MIN_ROOM + 2;

/// Up to one extra staircase between two floors per this many cells of the
/// upper floor (at most `MOST_EXTRA_STAIRS`), each with this percentage chance,
/// at least `MIN_STAIRS_GAP` cells from the others between those floors.
const CELLS_PER_EXTRA_STAIRS: i64 = 300;
const MOST_EXTRA_STAIRS: i64 = 3;
const EXTRA_STAIRS_PERCENT: u32 = 50;
const MIN_STAIRS_GAP: i64 = 8;

/// Percentage chance that a building of `same_size` floors has a small top.
const SMALL_TOP_PERCENT: u32 = 50;

/// Percentage chance that a floor is the same size as the one below.
const SAME_SIZE_PERCENT: u64 = 50;

/// Up to one extra outside door per this many cells of width plus height
/// (at most 3), each at least `MIN_ENTRANCE_GAP` cells (walking along the
/// grid) from the others.
const CELLS_PER_EXTRA_ENTRANCE: i64 = 25;
const MIN_ENTRANCE_GAP: i64 = 6;

/// How far apart pillars under a building on stilts are, at least: the larger
/// of the distances across and along.
const PILLAR_SPACING: i64 = 4;

/// Extra doors (making loops) per this many rooms.
const ROOMS_PER_EXTRA_DOOR: u16 = 6;

/// Percentage of doors that become secret, where nothing depends on them.
const SECRET_DOOR_PERCENT: u32 = 8;

/// A cell's position.
type Spot = (i64, i64);
