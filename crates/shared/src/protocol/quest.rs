//! Quest messages: what a client asks of givers and its quests
//! (`ClientMessage::Quest`), and what the server tells it
//! (`ServerMessage::Quest`). The server works quests out (`crate::quest`);
//! a client is only told what its player may know: a quest's story with the
//! offer, its steps once accepted, its ending at turn-in, and hints, never
//! numbers, for what a character still needs. Alignment is never told.

use serde::{Deserialize, Serialize};

use crate::hall::GateCode;
use crate::id::Id;
use crate::poi::ChunkPos;
use crate::quest::{QuestId, QuestKind, Standing, Step};
use crate::skill::SkillId;
use crate::spot::Spot;
use crate::story::{Ending, Person, RewardPart, Story};
use crate::world::WorldId;

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
    /// The quests the character has taken on another planet, for the Quest
    /// Log: answered with `QuestNews::PlanetLog`, or nothing if they have
    /// none there.
    PlanetLog(WorldId),
    /// Every planet the character has quest history on: answered with
    /// `QuestNews::Planets`.
    Planets,
    /// Follow this quest of this planet under the minimap, or none (the
    /// newest taken then): kept on the character, per planet. Answered with
    /// `QuestNews::Tracked`.
    Track(Option<QuestId>),
    /// Go into a quest's own dungeon ("Enter Cellar"), from beside its
    /// giver's door, while the quest is taken: answered by entering it
    /// (`EnterLevel`), or nothing.
    Enter(QuestId),
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
    /// The character's quests on this planet, their daily givers, and the
    /// quest they chose to follow.
    Log {
        quests: Vec<LogEntry>,
        dailies: Vec<DailyDone>,
        tracked: Option<QuestId>,
    },
    /// The character's quests on another planet, for the Quest Log.
    PlanetLog {
        world: WorldId,
        quests: Vec<LogEntry>,
        dailies: Vec<DailyDone>,
    },
    /// The quest followed on this planet now.
    Tracked(Option<QuestId>),
    /// One quest has changed: taken, a step moved on, finished, failed or
    /// given up.
    Changed(LogEntry),
    /// A quest handed in: what the giver says now. Its reward is in the
    /// inventory sent before this.
    TurnedIn { id: QuestId, ending: Ending },
    /// Something asked couldn't be done, and why.
    Refused { id: QuestId, why: QuestRefusal },
    /// The planets where the character has quest history, for the Hall's
    /// gates (those with quests taken) and the Quest Log: sent on coming into
    /// the Hall, and when asked.
    Planets(Vec<PlanetQuests>),
    /// Where on the level the player is on each quest taken has an objective
    /// that can be done now, for the maps' dots: only places the character
    /// may know (stairs they remember, a giver's door, a known poi). Sent
    /// whenever it changes; none for a quest means no dot on this level.
    Dots(Vec<QuestDots>),
    /// The quest bosses on the level the player is on that they can see, and
    /// the sign over each: sent whenever it changes, none for no sign.
    Bosses(Vec<BossMark>),
    /// The quest item lying on the level the player is on for them alone (their
    /// own copy: owner), while in sight, with the sign over it. Sent whenever
    /// it changes.
    Finds(Vec<FindMark>),
    /// Quests whose boss the character killed, or whose item they found,
    /// before taking them: each giver's town has a marker on the maps until
    /// the quest is taken. Sent with the log and whenever it changes.
    Ahead(Vec<DoneAhead>),
}

/// The sign over a quest item lying where it was left, for one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindMark {
    pub at: Spot,
    pub sign: BossSign,
}

/// The sign over a quest boss for one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossMark {
    /// The mob, as in `EntityLook::id`.
    pub id: Id,
    pub sign: BossSign,
}

/// Which sign a quest boss (or a quest item lying in a site) has over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossSign {
    /// A red "!": someone's quest, which the player doesn't have and hasn't
    /// dealt with (owner).
    Unknown,
    /// The player's own quest step: the quest's colour.
    Quest(QuestKind),
}

/// A quest a step of which the character has already done, not taken yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoneAhead {
    pub id: QuestId,
    pub kind: QuestKind,
    pub ahead: Ahead,
}

/// What a character did before taking a quest, so that step is done on
/// taking it (owner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ahead {
    /// Killed its boss: "I've already dealt with it".
    Slain,
    /// Picked up its item: "I've already found it".
    Found,
}

/// The cells of the level the player is on that one quest's dots go on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDots {
    pub id: QuestId,
    pub cells: Vec<(i64, i64)>,
}

/// A planet where the character has quest history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanetQuests {
    pub world: WorldId,
    /// Its name in runes (its first World Gate's code), if it is one of the
    /// Hall's and the server has learned it.
    pub code: Option<GateCode>,
    /// The planet the character is on (or came into the Hall from).
    pub here: bool,
    /// Some quests are taken there, not yet handed in ("!" over its gates).
    pub taken: bool,
    /// Some of them wait to be handed in ("?" over its gates, not "!").
    pub ready: bool,
}

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
    /// The giver has heard the character killed the quest's boss, or found
    /// its item.
    Ahead(Ahead),
}

/// A door's marker. A giver with quests in several states shows the first of
/// `Ready`, `Taken`, `Available`, `Locked` they have (owner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Marker {
    /// "!" in the kind's colour: a quest on offer.
    Available(QuestKind),
    /// "?" in the kind's colour: one to hand in.
    Ready(QuestKind),
    /// "!" in grey: one the character can't take yet.
    Locked,
    /// "?" in grey: one taken from them and not finished yet (owner).
    Taken,
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyDone {
    pub town: ChunkPos,
    pub building: u16,
    /// Who gives them, if their town's quests can still be made.
    pub giver: Option<Person>,
    /// The giver's town's name.
    pub home: String,
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
