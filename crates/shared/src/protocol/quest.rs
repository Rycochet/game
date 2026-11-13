//! Quest messages: what a client asks of givers and its quests
//! (`ClientMessage::Quest`), and what the server tells it
//! (`ServerMessage::Quest`). The server works quests out (`crate::quest`);
//! a client is only told what its player may know: a quest's story with the
//! offer, its steps once accepted, its ending at turn-in, and hints, never
//! numbers, for what a character still needs. Alignment is never told.

use serde::{Deserialize, Serialize};

use crate::poi::ChunkPos;
use crate::quest::{QuestId, QuestKind, Standing, Step};
use crate::skill::SkillId;
use crate::story::{Ending, RewardPart, Story};

/// What a client asks about quests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestAsk {
    /// Talk to a town's giver (a building, by `BuildingLook::id`) from beside
    /// its door, in turn with the player's steps: answered with
    /// `QuestNews::Talk`, or nothing if they aren't there.
    Talk { town: ChunkPos, building: u16 },
    /// The markers over a town's doors, for the town the player is in:
    /// answered with `QuestNews::Markers` (also sent unasked on arriving in a
    /// town and after a change).
    Markers { town: ChunkPos },
    /// Take a quest on offer, from beside its giver's door.
    Accept(QuestId),
    /// Give up a quest taken, wherever the player is: progress is lost, and it
    /// can't be taken again for an hour.
    Abandon(QuestId),
    /// Hand in a quest whose steps are all done, from beside its giver's door.
    TurnIn(QuestId),
    /// Every quest of this planet the character has taken: answered with
    /// `QuestNews::Log` (also sent unasked on joining a world).
    Log,
}

/// What the server tells a client about quests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestNews {
    /// What a giver has for the player: quests on offer, or that they can't
    /// take yet (with hints), and quests of theirs taken or ready to hand in.
    Talk {
        town: ChunkPos,
        building: u16,
        offers: Vec<Offer>,
    },
    /// The marker over each giver's door that has one, in a town.
    Markers {
        town: ChunkPos,
        markers: Vec<(u16, Marker)>,
    },
    /// The character's quests on this planet, and their daily givers.
    Log {
        quests: Vec<LogEntry>,
        dailies: Vec<DailyDone>,
    },
    /// One quest has changed: taken, a step moved on, finished, failed or
    /// given up.
    Changed(LogEntry),
    /// A quest handed in: what the giver says now. Its reward is in the
    /// inventory sent before this.
    TurnedIn { id: QuestId, ending: Ending },
    /// Something asked couldn't be done, and why.
    Refused { id: QuestId, why: QuestRefusal },
}

/// One quest a giver talks about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub id: QuestId,
    pub kind: QuestKind,
    pub state: OfferState,
}

/// Where a giver's quest stands for the player.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfferState {
    /// On offer: its story and what it pays.
    Available {
        story: Box<Story>,
        reward: Vec<RewardPart>,
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
    /// The giver wants someone good at this.
    Skill(SkillId),
}

/// A door's marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Marker {
    /// "!" in the kind's colour: a quest on offer.
    Available(QuestKind),
    /// "?" in the kind's colour: one to hand in.
    Ready(QuestKind),
    /// "!" in grey: one the character can't take yet.
    Locked,
}

/// One of the character's quests, as their Quest Log shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: QuestId,
    pub kind: QuestKind,
    /// The giver's building in `id.town`.
    pub building: u16,
    pub state: QuestState,
    pub story: Box<Story>,
    pub steps: Vec<Step>,
    /// How far along each step is (its count, or 1 for done when uncounted).
    pub counts: Vec<u32>,
    pub reward: Vec<RewardPart>,
    /// When it last changed state: Unix seconds.
    pub at: i64,
}

/// Where one of the character's quests stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestState {
    Taken,
    /// Every step done: to hand in.
    Ready,
    Done,
    Failed,
    Abandoned,
}

/// A daily giver the character has done dailies for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyDone {
    pub town: ChunkPos,
    pub building: u16,
    pub count: u32,
    /// When the last was handed in: Unix seconds.
    pub last: i64,
}

/// Why something asked of a quest wasn't done.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestRefusal {
    /// The character has as many quests on this planet as they may.
    TooMany,
    /// Not all of the reward fits in what they carry: the giver keeps it,
    /// and the quest, until it does.
    NoRoom,
    /// It isn't on offer to them now.
    NotOffered,
    /// It isn't one of theirs, or its steps aren't all done.
    NotReady,
}
