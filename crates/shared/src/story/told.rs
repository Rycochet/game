//! A story as told: what the server sends a client and the client words in
//! its language. It holds nothing secret: its own text seed, the cast, the
//! facts as the giver tells them, and the beats in paragraphs. The truth
//! behind any lie and the ending stay with the server until turn-in.

use serde::{Deserialize, Serialize};

use super::Beat;
use super::cast::{Carried, Cast, Role};
use super::facts::{
    Errand, ItemCategory, Occasion, Reason, RewardPart, Season, Stake, TimeAgo, Weather,
};
use super::request::{Ask, Creatures, Destination, Place};
use crate::rng::Seed;

/// The facts a story is told with. Checkable ones come from the quest and
/// are true; colour ones are rolled and kept only when a beat uses them.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Facts {
    /// The giver's town's name.
    pub home: String,
    pub asks: Vec<Ask>,
    pub place: Option<Place>,
    pub creatures: Option<Creatures>,
    pub item: Option<ItemCategory>,
    pub destination: Option<Destination>,
    pub reward: Vec<RewardPart>,
    pub errand: Option<Errand>,
    pub time_ago: Option<TimeAgo>,
    pub occasion: Option<Occasion>,
    pub stake: Option<Stake>,
    pub season: Option<Season>,
    pub weather: Option<Weather>,
}

/// A quest's story, for its offer and while it is under way.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Story {
    /// Made only for text: picks each beat's wording.
    pub text_seed: Seed,
    pub reason: Reason,
    pub cast: Cast,
    pub facts: Facts,
    /// Up to three paragraphs of beats; the last holds the ask.
    pub paragraphs: Vec<Vec<Beat>>,
}

/// The real value of a fact someone lied about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Truth {
    Errand(Errand),
    TimeAgo(TimeAgo),
}

/// A lie in a story: who told it (the witness role also stands for an
/// unnamed passerby) and the truth. Fixed when the story is made, so nothing
/// later contradicts it (owner).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lie {
    pub liar: Role,
    pub truth: Truth,
}

/// What the giver says at turn-in: beats, and the lies coming out. Sent then,
/// not before.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ending {
    pub beats: Vec<Beat>,
    pub lies: Vec<Lie>,
}

/// Everything telling a story makes: the story to send, and the ending kept
/// by the server until the quest is turned in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Telling {
    pub story: Story,
    pub ending: Ending,
}

impl Story {
    /// These roles (and the pet and boss, if asked) and the backstory's
    /// facts, to carry into a later step's story (`Request::carried`).
    pub fn carry(&self, roles: &[Role], pet: bool, boss: bool) -> Carried {
        Carried {
            errand: self.facts.errand,
            time_ago: self.facts.time_ago,
            occasion: self.facts.occasion,
            ..self.cast.carry(roles, pet, boss)
        }
    }

    /// Every beat, paragraph by paragraph.
    pub fn beats(&self) -> impl Iterator<Item = Beat> + '_ {
        self.paragraphs.iter().flatten().copied()
    }
}
