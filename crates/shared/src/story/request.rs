//! What the quest generator hands the story: the quest's checkable facts (its
//! goals, place, creatures, item, reward), who the giver is, and what carries
//! over from earlier steps. The story explains these; it never changes them.

use super::cast::Carried;
use super::facts::{CountBand, Goal, ItemCategory, Reason, Remains, RewardPart, Season, Weather};
use super::{Boss, Direction, DistanceBand, Household, Pet, PlaceDescription};
use crate::creature::CreatureKind;
use crate::rng::Seed;

/// One thing a quest asks, in order: "Gather 8 herbs", then "Deliver".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Ask {
    pub goal: Goal,
    /// How many, when counted ("Slay kobolds 2/6").
    pub count: Option<u32>,
}

/// The place a quest sends the player, as the giver's town sees it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Place {
    pub description: PlaceDescription,
    /// A proper name, for places that have one (castles, towns).
    pub name: Option<String>,
    pub direction: Direction,
    pub distance: DistanceBand,
}

/// The creatures at the place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Creatures {
    pub kind: CreatureKind,
    pub count: CountBand,
}

/// Another town something is taken to.
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Destination {
    pub name: String,
    /// Its people, for the recipient's name.
    pub kind: CreatureKind,
    pub direction: Direction,
    pub distance: DistanceBand,
}

/// How much a story says: Short (dailies) is one paragraph of a hook and the
/// ask; Full is up to three.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Length {
    Short,
    #[default]
    Full,
}

/// Everything a story is told from. Make one with `new`, then fill in what
/// the quest has.
#[derive(Clone, Debug)]
pub struct Request {
    /// The story's own seed: never a poi's or anything secret, as the client
    /// is sent it. Every colour fact and wording comes from it.
    pub text_seed: Seed,
    /// The giver's household (its kind is the giver's people), and which
    /// member the giver is. Kin in the story come from the same household.
    pub household: Household,
    pub giver_index: u32,
    /// The giver's town's name.
    pub home: String,
    /// What the quest asks, one or two things.
    pub asks: Vec<Ask>,
    pub place: Option<Place>,
    pub creatures: Option<Creatures>,
    pub boss: Option<Boss>,
    pub item: Option<ItemCategory>,
    /// What a "Collect" step brings back, worded as the step's items.
    pub remains: Option<Remains>,
    pub destination: Option<Destination>,
    /// The pet the quest is about, if it is one in the world (an escort).
    /// A story about a lost pet makes one up otherwise.
    pub pet: Option<Pet>,
    pub reward: Vec<RewardPart>,
    /// Seasons and weather are only told in quests with a bigger reward.
    pub higher_reward: bool,
    pub season: Option<Season>,
    pub weather: Option<Weather>,
    /// A chain keeps its reason: set it from the first step's story.
    pub reason: Option<Reason>,
    /// A later step of a chain: twists and quandaries may be told.
    pub later_step: bool,
    /// The quest needs a bad reputation (decided by the quest, hidden from
    /// everyone else): its story has a dark beat, and only then.
    pub dark: bool,
    /// The chain has a next step: the ending gives a lead to it.
    pub continues: bool,
    pub length: Length,
    /// People, a pet or a boss from an earlier step, instead of new ones.
    pub carried: Carried,
}

impl Request {
    /// A request asking `goal`, with nothing else known yet.
    pub fn new(
        text_seed: Seed,
        household: Household,
        giver_index: u32,
        home: impl Into<String>,
        goal: Goal,
    ) -> Self {
        Self {
            text_seed,
            household,
            giver_index,
            home: home.into(),
            asks: vec![Ask { goal, count: None }],
            place: None,
            creatures: None,
            boss: None,
            item: None,
            remains: None,
            destination: None,
            pet: None,
            reward: Vec::new(),
            higher_reward: false,
            season: None,
            weather: None,
            reason: None,
            later_step: false,
            dark: false,
            continues: false,
            length: Length::Full,
            carried: Carried::default(),
        }
    }

    /// The first thing asked: what the plot is chosen for.
    pub fn main_goal(&self) -> Goal {
        self.asks.first().map_or(Goal::Visit, |ask| ask.goal)
    }
}
