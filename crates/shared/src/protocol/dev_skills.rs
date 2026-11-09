//! What dev tools can do to a character's skills (the debug menu's Learn Skills).

use serde::{Deserialize, Serialize};

use crate::skill::SkillId;

/// What `ClientMessage::DevSkills` does to the skills.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevSkills {
    /// Only asks what they are.
    Ask,
    /// Practises `DEV_SKILLS_LEARNT` nodes at the ends of branches, picked at random,
    /// by random amounts, so there is a tree to look at.
    LearnSome,
    /// Forgets every skill.
    Clear,
    /// Learns a node at `skill::DEV_LEARN_PERCENT`, the nodes above it their share.
    Learn(SkillId),
    /// Forgets a node and everything under it.
    Forget(SkillId),
    /// Sets one node to a whole percentage (0 forgets it).
    Set { skill: SkillId, percent: u32 },
    /// Sets every node of the tree to a whole percentage.
    SetAll(u32),
}

/// How many nodes `DevSkills::LearnSome` practises.
pub const DEV_SKILLS_LEARNT: usize = 6;
