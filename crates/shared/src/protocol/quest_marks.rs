//! The signs over quest bosses and quest items in sight (`QuestNews::Bosses`,
//! `QuestNews::Finds`).

use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::quest::QuestKind;
use crate::spot::Spot;

/// The sign over a quest item lying where it was left, for one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindMark {
    pub at: Spot,
    pub sign: BossSign,
}

/// The sign over a quest boss for one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossMark {
    /// The mob, as in `EntityLook::id`.
    pub id: Id,
    pub sign: BossSign,
}

/// Which sign a quest boss (or a quest item lying in a site) has over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossSign {
    /// A red "!": someone's quest, which the player doesn't have and hasn't
    /// dealt with (owner).
    Unknown,
    /// The player's own quest step: the quest's colour.
    Quest(QuestKind),
}
