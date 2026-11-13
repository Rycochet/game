//! A quest's steps: a graph, each step coming after others (in series) or
//! alongside them (in parallel). Turning in, "Return to the giver", always
//! comes after every step, so it isn't one of them.

use serde::{Deserialize, Serialize};

use super::Hook;
use crate::poi::ChunkPos;
use crate::skill::SkillId;
use crate::story::Goal;

/// Where a step is done.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    /// At a poi, by one of its hooks.
    Poi { chunk: ChunkPos, hook: Hook },
    /// In the wilderness round a chunk (the giver's own town's, for now).
    Wilds { chunk: ChunkPos },
    /// In another town: something taken there.
    Town { chunk: ChunkPos },
}

impl Target {
    /// The chunk the step sends the player to.
    pub const fn chunk(self) -> ChunkPos {
        match self {
            Self::Poi { chunk, .. } | Self::Wilds { chunk } | Self::Town { chunk } => chunk,
        }
    }
}

/// Another way of doing a step, so later abilities never lock anyone out
/// (owner): carrying a lost dog home rather than leading it, say.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Way {
    /// Carry what is to be escorted (and maybe get bitten).
    Carry,
    /// Calm an animal so it follows.
    Tame,
    /// Drive a boss off rather than kill it.
    Intimidate,
    // New ways go at the end: messages carry one by its place.
}

/// A way, and the skill it needs, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Alternative {
    pub way: Way,
    pub skill: Option<SkillId>,
}

/// One thing a quest asks.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Step {
    pub goal: Goal,
    pub target: Target,
    /// How many, when counted ("Slay kobolds 2/6").
    pub count: Option<u32>,
    /// The steps (by index) that must be done first; none means it can be
    /// done from the start.
    pub after: Vec<u8>,
    /// Other ways of doing it.
    pub alternatives: Vec<Alternative>,
}

impl Step {
    /// Whether it can be worked on, given which steps are done.
    pub fn is_open(&self, done: &[bool]) -> bool {
        (self.after.iter()).all(|&i| done.get(i as usize).copied().unwrap_or(false))
    }
}
