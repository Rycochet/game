//! What dev tools can do to a character's skills.

use serde::{Deserialize, Serialize};

/// What `ClientMessage::DevSkills` does to the character's skills.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevSkills {
    /// Practises `DEV_SKILLS_LEARNT` nodes at the ends of branches, picked at random,
    /// by random amounts, so there is a tree to look at.
    LearnSome,
    /// Back to what its kind starts with.
    Forget,
}

/// How many nodes `DevSkills::LearnSome` practises.
pub const DEV_SKILLS_LEARNT: usize = 6;
