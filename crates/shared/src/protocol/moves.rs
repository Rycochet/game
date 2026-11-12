//! Moving on levels in units (free positions, `docs/context/free-positions.md`).
//!
//! **The map's measure.** Positions in `Position`, `Teleported`, `DevTeleport`,
//! `EntityLook` and `Stretch` are counted as the map the player is on counts
//! them: world map points on the world map; on a site level, in the wilderness
//! and in the Hall, units of `spot::CELL_SPAN` (240) a cell, a cell's middle at
//! its `(x, y) × 240` (`Spot::middle_of`). In the Hall they count from
//! `EnterHall`'s `origin` (`origin × 240` units).
//!
//! On levels the client sends `ClientMessage::Move { to }`: a straight move
//! from where the server has the player to `to`, at most `MOST_MOVE` long. The
//! server takes it if it is a step the cell rule allows (`movement`, from one
//! cell's middle to another's), or else a legal segment (`clearance`).

use serde::{Deserialize, Serialize};

use crate::clearance::JUMP_REACH;
use crate::id::Id;
use crate::spot::{CELL_SPAN, Spot};

/// The longest move the server takes, in units: the longest jump.
pub const MOST_MOVE: u64 = JUMP_REACH;

/// Someone in sight moving: from `from` to `to` (the map's measure), starting
/// when it is received and taking `ticks` action ticks (none: at once). Sent
/// in `ServerMessage::Moves` only when someone's stretch changes; the way they
/// face comes from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stretch {
    pub id: Id,
    pub from: (i64, i64),
    pub to: (i64, i64),
    pub ticks: u32,
}

/// Units from a level's cell: its middle.
pub fn units_of_cell(cell: (i64, i64)) -> (i64, i64) {
    let Spot { x, y } = Spot::middle_of(cell);
    (x, y)
}

/// The cell a position in units is in (`Spot::cell`).
pub fn cell_of_units((x, y): (i64, i64)) -> (i64, i64) {
    Spot::new(x, y).cell()
}

/// Whether `spot` is a cell's middle exactly.
pub fn is_middle(spot: Spot) -> bool {
    spot.x.rem_euclid(CELL_SPAN) == 0 && spot.y.rem_euclid(CELL_SPAN) == 0
}
