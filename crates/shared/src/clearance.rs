//! Room to move on a level for creatures of each size: what blocks them
//! (`Blocker`), whether a move from one `Spot` to another is allowed and how
//! squeezed it is (`check_segment`), sliding along what is in the way (`slide`)
//! and where each size has room (`RoomMap`). See
//! `docs/context/shared/clearance.md`.
//!
//! A creature is a circle (`Size::radius`). It may move where its middle has
//! footing and the circle, swept along the move, clears every blocker. It may
//! squeeze through a gap down to half its own width, more slowly (`squeeze`).
//! A jump is one move over one jumpable cell. Integers only: both sides agree.
//!
//! Nothing in play uses these yet (free positions, PR 2 of 9).

// Split into files (`cargo xtask lint` keeps files small).
mod blocker;
mod cell_blockers;
mod footing;
mod room_map;
mod segment;
mod slide;
mod squeeze;

pub use blocker::{Blocker, Shape};
pub use cell_blockers::{
    CORNER_CUT, DOOR_HALF_GAP, FENCE_HALF, TRUNK_RADIUS, blockers_near, cell_blockers,
};
pub use footing::has_footing;
pub use room_map::RoomMap;
pub use segment::{JUMP_REACH, Passage, check_segment, segment_cost, segment_is_legal};
pub use slide::slide;
pub use squeeze::{MOST_SQUEEZE_PER_MILLE, squeeze_per_mille};

#[cfg(test)]
mod tests;
