//! Developer tools: a mob made to order and put on a cell (Create Mob).

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
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
