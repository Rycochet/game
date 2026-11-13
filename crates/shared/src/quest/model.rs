//! A whole quest: who gives it, what it needs, its steps, its reward, how it
//! moves alignment, and its story.

use super::{QuestId, QuestKind, Requirement, Step};
use crate::alignment::Deed;
use crate::story::{Ending, RewardPart, Story};

/// One quest, as the generator makes it. The server keeps all of it; a
/// client is sent its story and what it needs to show the steps, never the
/// ending (until turn-in), hidden requirements or alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quest {
    pub id: QuestId,
    pub kind: QuestKind,
    /// The giver's building (its `RoomId`) in the town. The giver is its
    /// household's member `member` (the story's cast names them).
    pub building: u16,
    pub member: u32,
    pub requirements: Vec<Requirement>,
    pub steps: Vec<Step>,
    /// The real reward, which the story names (owner).
    pub reward: Vec<RewardPart>,
    /// What turning it in does to the character's alignment.
    pub deeds: Vec<Deed>,
    /// Sent with the offer.
    pub story: Story,
    /// Kept by the server until turn-in.
    pub ending: Ending,
}

impl Quest {
    /// The steps that can be worked on now, given which are done.
    pub fn open_steps(&self, done: &[bool]) -> Vec<usize> {
        (0..self.steps.len())
            .filter(|&i| !done.get(i).copied().unwrap_or(false) && self.steps[i].is_open(done))
            .collect()
    }

    /// Whether every step is done, so only turning in is left ("?").
    pub fn ready_to_turn_in(&self, done: &[bool]) -> bool {
        (0..self.steps.len()).all(|i| done.get(i).copied().unwrap_or(false))
    }

    /// Whether anyone outside its alignment band is never shown it.
    pub fn is_hidden_by_alignment(&self) -> bool {
        self.requirements.iter().any(Requirement::hides_quest)
    }
}
