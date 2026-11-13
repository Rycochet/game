//! The three kinds of quest, each with its own colour and rules (owner).

use serde::{Deserialize, Serialize};

/// What kind of quest it is: how often it can be done and its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum QuestKind {
    /// Yellow: once per character.
    Normal,
    /// Blue: a new one from the giver a day after the last was turned in,
    /// the same for everyone on the same UTC day.
    Daily,
    /// Orange: cities only, a long chain ending in an artifact or the
    /// planet's set item (Quest system PR 11; not generated yet).
    Artifact,
    // New kinds go at the end: messages carry one by its place.
}
