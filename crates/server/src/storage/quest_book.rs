//! A character's quests on one planet, and their standing in its towns: kept
//! in `CharacterWorldSave`, as quest lists are per planet (owner). Quests are
//! kept by id alone: the server makes them again from their town's seed
//! (`shared::quest`). Times are Unix seconds, as dailies go by real days.

use serde::{Deserialize, Serialize};
use shared::hall::GateCode;
use shared::poi::ChunkPos;
use shared::quest::QuestId;
use std::collections::{BTreeMap, BTreeSet};

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
    /// Quests whose boss the character killed before taking them (owner):
    /// the giver offers them with that step done. Kept until taken.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub slain: BTreeSet<QuestId>,
    /// Quests whose unique item the character picked up before taking them
    /// (owner): offered with that step done. Kept until taken.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub found: BTreeSet<QuestId>,
    /// The quest followed under the minimap, chosen in the Quest Log (none:
    /// the newest taken).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracked: Option<QuestId>,
    /// The planet's name in runes, learned while there, so the Quest Log can
    /// name it from anywhere (`None` until then, or if it isn't a Hall
    /// planet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<GateCode>,
    /// Quests' own dungeons the character has a copy of (made on entering,
    /// gone when it resets or the quest ends).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dungeons: BTreeMap<QuestId, super::DungeonStay>,
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
    /// Where the one its Escort step sees home is, once it has been come
    /// across (`quests::escort`); `None` before, and after it is home.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escort: Option<EscortAt>,
}

/// Where someone escorted is (`Progress::escort`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscortAt {
    /// With the character, wherever they go: on the world map too.
    With,
    /// Waiting on a site's level where the character lost them (or first came
    /// across them), at this spot, until they come back for them.
    Waiting {
        site: super::SiteId,
        level: u32,
        spot: shared::spot::Spot,
    },
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
    /// Whether anything has been done with quests here: the Quest Log lists
    /// the planet.
    pub fn has_history(&self) -> bool {
        !(self.taken.is_empty()
            && self.done.is_empty()
            && self.ended.is_empty()
            && self.dailies.is_empty())
    }

    /// Whether `id` may be taken again at `now`, as far as failing or giving
    /// it up goes.
    pub fn may_retake(&self, id: &QuestId, now: i64) -> bool {
        (self.ended.get(id)).is_none_or(|ended| now - ended.at >= RETAKE_AFTER)
    }
}

#[cfg(test)]
mod tests;
