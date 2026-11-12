//! A story's cast: the people in it by role, and its pet and boss. Sent to
//! clients resolved (names and all); a later step of a chain carries roles
//! over from an earlier story instead of rolling new people (`Carried`).

use serde::{Deserialize, Serialize};

use super::facts::{Errand, Occasion, Passerby, Relation, State, TimeAgo, Trait};
use super::{Boss, Person, Pet};

/// A person's part in a story. The giver is always there; the others only
/// when a beat needs them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Role {
    Giver,
    /// Kin or a friend of the giver: the one who went, was lost or died.
    Relative,
    /// A neighbour or another family the giver has a quarrel with.
    Rival,
    /// Who a delivery is for, in another town.
    Recipient,
    /// Who saw something or told the rumour, when named (else a `Passerby`).
    Witness,
    // New roles go at the end: messages carry one by its place.
}

impl Role {
    pub const ALL: [Self; 5] = [
        Self::Giver,
        Self::Relative,
        Self::Rival,
        Self::Recipient,
        Self::Witness,
    ];

    /// The start of its values' names in language files (`$relative-name`).
    pub const fn key(self) -> &'static str {
        match self {
            Self::Giver => "giver",
            Self::Relative => "relative",
            Self::Rival => "rival",
            Self::Recipient => "recipient",
            Self::Witness => "witness",
        }
    }
}

/// Someone in a story.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Member {
    pub role: Role,
    pub person: Person,
    /// What they are to the giver (none for the giver).
    pub relation: Option<Relation>,
    pub nature: Trait,
    pub state: State,
}

impl Member {
    /// The same person, now `state`, if that is a step forward
    /// (`State::can_become`): for carrying someone into a later step.
    pub fn moved_to(&self, state: State) -> Option<Self> {
        self.state.can_become(state).then(|| Self {
            state,
            ..self.clone()
        })
    }
}

/// The giver's household's animal in a story.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CastPet {
    pub pet: Pet,
    pub nature: Trait,
    pub state: State,
}

/// Everyone in one story.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cast {
    /// The giver first, then others in role order.
    pub members: Vec<Member>,
    pub pet: Option<CastPet>,
    pub boss: Option<Boss>,
    /// An unnamed witness, when the story has one rather than a named one.
    pub passerby: Option<Passerby>,
}

impl Cast {
    pub fn member(&self, role: Role) -> Option<&Member> {
        self.members.iter().find(|m| m.role == role)
    }

    pub fn giver(&self) -> &Member {
        self.member(Role::Giver).expect("every cast has a giver")
    }

    /// These roles (and the pet and boss, if asked) to carry into a later
    /// step's story. Roles the cast hasn't got are left out. `Story::carry`
    /// also carries the backstory's facts.
    pub fn carry(&self, roles: &[Role], pet: bool, boss: bool) -> Carried {
        Carried {
            members: (self.members.iter())
                .filter(|m| roles.contains(&m.role))
                .cloned()
                .collect(),
            pet: self.pet.clone().filter(|_| pet),
            boss: self.boss.clone().filter(|_| boss),
            ..Carried::default()
        }
    }
}

/// What a later step takes from an earlier story. The server rebuilds the
/// earlier story from its request (so its seed) and applies the states its
/// finished steps set (`Member::moved_to`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Carried {
    pub members: Vec<Member>,
    pub pet: Option<CastPet>,
    pub boss: Option<Boss>,
    /// The backstory as the earlier step told it, so it is told the same.
    pub errand: Option<Errand>,
    pub time_ago: Option<TimeAgo>,
    pub occasion: Option<Occasion>,
}

impl Carried {
    pub fn member(&self, role: Role) -> Option<&Member> {
        self.members.iter().find(|m| m.role == role)
    }
}
