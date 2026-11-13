//! What a character needs before a giver offers a quest. A need the giver
//! can't see past shows as a grey "!" and a hint in their words; some are
//! hidden (only the hint tells of them), and an alignment need hides the
//! quest entirely from anyone without it (owner).

use serde::{Deserialize, Serialize};

use super::QuestId;
use crate::alignment::Band;
use crate::poi::ChunkPos;
use crate::skill::SkillId;

/// A character's standing in a town, lowest first (tiers: Claude's, kept by
/// the server from quests and helpful acts; Quest system PR 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Standing {
    Stranger,
    Known,
    Trusted,
    Honoured,
    Champion,
}

impl Standing {
    pub const ALL: [Self; 5] = [
        Self::Stranger,
        Self::Known,
        Self::Trusted,
        Self::Honoured,
        Self::Champion,
    ];
}

/// One thing needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Need {
    /// At least this standing in a town.
    Standing { town: ChunkPos, least: Standing },
    /// Another quest turned in: the one before in a chain, or a smaller
    /// town's feeder.
    Done(QuestId),
    /// A skill known to at least this value (`Skills::value`, hundredths).
    Skill { skill: SkillId, least: u32 },
    /// An alignment band. The quest is hidden entirely from anyone outside
    /// it: no grey "!", no hint.
    Alignment(Band),
}

/// A need, and whether only a hint tells of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Requirement {
    pub need: Need,
    /// Shown only as the giver's hint, never as what is missing.
    pub hidden: bool,
}

impl Requirement {
    pub const fn shown(need: Need) -> Self {
        Self {
            need,
            hidden: false,
        }
    }

    pub const fn hidden(need: Need) -> Self {
        Self { need, hidden: true }
    }

    /// Whether a character without it doesn't see the quest at all.
    pub const fn hides_quest(&self) -> bool {
        matches!(self.need, Need::Alignment(_))
    }
}
