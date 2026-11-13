//! A character's quests on one planet, and their standing in its towns: kept
//! in `CharacterWorldSave`, as quest lists are per planet (owner). Quests are
//! kept by id alone: the server makes them again from their town's seed
//! (`shared::quest`). Times are Unix seconds, as dailies go by real days.

use serde::{Deserialize, Serialize};
use shared::poi::ChunkPos;
use shared::quest::QuestId;
use std::collections::BTreeMap;

/// Everything a character has done with quests on a planet.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestBook {
    /// Quests taken and not yet handed in, failed or given up.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub taken: BTreeMap<QuestId, Progress>,
    /// Quests handed in (not dailies: `dailies` counts those), and when.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub done: BTreeMap<QuestId, i64>,
    /// Quests failed or given up, and when, the latest only: each can be
    /// taken again an hour later.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ended: BTreeMap<QuestId, Ended>,
    /// Dailies handed in, by giver: town and building.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dailies: BTreeMap<(ChunkPos, u16), Dailies>,
    /// Standing points by town (`quests::standing` turns them into tiers).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub standing: BTreeMap<ChunkPos, u32>,
}

/// How far along a quest taken is.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    /// When it was taken.
    pub at: i64,
    /// How far along each step is: its count, or 1 once done if uncounted.
    pub counts: Vec<u32>,
    /// Every step is done: it waits to be handed in. Kept so the Hall's gates
    /// can show it without making the quest again on another planet.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ready: bool,
}

/// A quest failed or given up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ended {
    pub at: i64,
    /// Failed, rather than given up.
    pub failed: bool,
}

/// A daily giver's quests a character has handed in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dailies {
    pub count: u32,
    /// When the last was handed in.
    pub last: i64,
}

/// How long after failing or giving up a quest it can be taken again (owner:
/// at least an hour).
pub const RETAKE_AFTER: i64 = 60 * 60;

impl QuestBook {
    /// Whether `id` may be taken again at `now`, as far as failing or giving
    /// it up goes.
    pub fn may_retake(&self, id: &QuestId, now: i64) -> bool {
        (self.ended.get(id)).is_none_or(|ended| now - ended.at >= RETAKE_AFTER)
    }
}

#[cfg(test)]
mod tests;
