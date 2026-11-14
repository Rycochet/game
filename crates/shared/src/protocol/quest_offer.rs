//! What a giver says of their quests: each one offered and where it stands,
//! hints for what a character still needs (never numbers), and who they
//! heard good words of the character from.

use serde::{Deserialize, Serialize};

use super::quest::Ahead;
use crate::celestial::ColourName;
use crate::poi::ChunkPos;
use crate::quest::{QuestId, QuestKind, Standing};
use crate::site::BuildingKind;
use crate::skill::SkillId;
use crate::story::{Person, RewardPart, Story};

/// One quest a giver talks about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub id: QuestId,
    pub kind: QuestKind,
    /// Who gives it: the building's household's member for its slot.
    pub giver: Person,
    pub state: OfferState,
}

/// Where a giver's quest stands for the player.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferState {
    /// On offer: its story and what it pays.
    Available {
        story: Box<Story>,
        reward: Vec<RewardPart>,
        /// The character has already killed its boss or found its item: they
        /// say so, and that step is done on taking it.
        ahead: Option<Ahead>,
        /// Who the giver heard of the character from, when the quest needed
        /// another one done ("I heard you helped Mara Tull").
        heard: Option<Heard>,
    },
    /// Not yet: what the giver says of why ("!" in grey).
    Locked(Vec<Hint>),
    /// Taken, and still under way.
    Taken,
    /// Taken, every step done: to hand in ("?").
    Ready,
}

/// Why a giver won't give a quest yet, for the client to put in the giver's
/// words: never how far off the character is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hint {
    /// Folk round here don't know the character well enough yet.
    Standing { least: Standing },
    /// The giver has more to ask once their last quest is done.
    Earlier,
    /// Someone in another town (or another giver here) should be helped first:
    /// its name, when it has one.
    Elsewhere {
        town: ChunkPos,
        name: Option<String>,
    },
    /// Another giver in the same town should be helped first: who they are.
    Neighbour { giver: Person, kind: BuildingKind },
    /// The giver wants someone good at this.
    Skill(SkillId),
    /// The giver waits for a moon in the sky: its colour, if it has a name.
    Moon { colour: Option<ColourName> },
    /// The giver has heard the character killed the quest's boss, or found
    /// its item.
    Ahead(Ahead),
}

/// Who a giver heard good words of the character from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heard {
    /// Another giver in their town.
    Neighbour { giver: Person, kind: BuildingKind },
    /// Someone in a smaller town nearby: its name, when it has one.
    Town { name: Option<String> },
}
