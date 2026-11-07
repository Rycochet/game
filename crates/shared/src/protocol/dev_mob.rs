//! Developer tools for mobs: making one to order (Create Mob), looking into one,
//! deleting one, and stopping them all.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::id::Id;
use crate::sheet::Feeling;

/// A mob for the server to put on `(x, y)` of the player's level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevMob {
    pub kind: CreatureKind,
    pub x: i64,
    pub y: i64,
    /// How it stands with the character who made it; `None` as its kind feels.
    pub attitude: Option<Feeling>,
    /// Whether it has the gear its kind may start with (rolled from `seed`).
    pub gear: bool,
    /// It stands still and does nothing, even when hit: something to practise on.
    pub still: bool,
    /// Its looks, build and gear, rolled as a level's own mob's.
    pub seed: u64,
}

/// What a developer does to mobs (`ClientMessage::DevMobs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevMobTool {
    /// Put a mob made to order on a cell of the player's site level. Not kept:
    /// it goes when the level is let go.
    Create(DevMob),
    /// Tell me everything the server knows of this mob, in sight:
    /// answered with `ServerMessage::DevMobInfo`.
    Inspect(Id),
    /// Take this mob, in sight, away for good: no body, nothing it had left.
    Delete(Id),
    /// Stop every mob in this world thinking and moving (they can still be hit),
    /// or let them go again.
    Freeze(bool),
}
