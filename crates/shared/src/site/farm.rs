//! Farms: a farmhouse (a building, see `building`) in the middle of up to eight
//! fields, one in each of the eight places round it, one of which may be a pond
//! instead.
//!
//! The house's floors keep their cells: the ground floor's level just reaches
//! further, to north and west into negative coordinates, so stairs still line
//! up. Every field is a rectangle with a fence round it, and one gate as near
//! the front door as a side of the fence gets. Dirt tracks join the front door,
//! the gates and the edge of the level.

// Split into files (`cargo xtask lint` keeps files small).
mod fences;
mod lay_out;
mod options;
mod plan;
mod tracks;

use fences::fence_field;
pub(super) use fences::fence_line;
use lay_out::lay_out;
pub(super) use lay_out::{Plot, surroundings};
pub use options::{FarmOptions, FieldLook};
pub use plan::Plan;
use tracks::lay_tracks;

/// Ground left between the house's yard, each field and the next. At least 1
/// must be walkable, or a fence jumped could land on nothing.
const LANE: i64 = 2;

/// Ground left round the outside of the fields.
const BORDER: i64 = 3;

/// The smallest a field can be across: 8 cells sown and the fence each side.
const MIN_FIELD: i64 = 10;

/// How much room a place north, east, south or west of the house may have.
const PLACE_SIZE: (i64, i64) = (12, 22);

/// Percentage chance that a farm with at least 2 places has a pond in one.
const POND_PERCENT: u32 = 33;

/// How many blobs a pond is made of, and how far each reaches from its middle.
const POND_BLOBS: (i64, i64) = (3, 6);
const POND_REACH: (i64, i64) = (2, 6);

/// Percentage chance a farm is all crops or all pasture; otherwise each field
/// chooses.
const ALL_CROPS_PERCENT: u64 = 50;
const ALL_PASTURE_PERCENT: u64 = 25;

/// Of each 100 cells of a ruined farm's fence, how many start a gap, per level
/// of ruin; and how long a gap may be.
const FENCE_GAP_PERCENT: u32 = 4;
const FENCE_GAP: (i64, i64) = (1, 3);

#[cfg(test)]
mod tests;
