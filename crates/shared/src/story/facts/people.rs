//! Facts about the people in a story: how they are related to the giver, what
//! they are like, and what has become of them.

use super::super::fact::fact;
use crate::gender::Gender;

fact! {
    /// How someone is related to the giver. The kin ones share the giver's
    /// family name; the gendered ones only go to someone of that gender, and
    /// the plain ones (Parent, Sibling...) suit anyone.
    Relation as "relation" {
        Grandmother = "grandmother",
        Grandfather = "grandfather",
        Mother = "mother",
        Father = "father",
        Sister = "sister",
        Brother = "brother",
        Daughter = "daughter",
        Son = "son",
        Aunt = "aunt",
        Uncle = "uncle",
        Cousin = "cousin",
        Spouse = "spouse",
        Betrothed = "betrothed",
        Friend = "friend",
        Neighbour = "neighbour",
        Apprentice = "apprentice",
        Master = "master",
        Partner = "partner",
        Grandparent = "grandparent",
        Parent = "parent",
        Sibling = "sibling",
        Child = "child",
    }
}

impl Relation {
    /// The gender someone must be to be called this, if any.
    pub const fn gender(self) -> Option<Gender> {
        use Relation::*;
        match self {
            Grandmother | Mother | Sister | Daughter | Aunt => Some(Gender::Female),
            Grandfather | Father | Brother | Son | Uncle => Some(Gender::Male),
            _ => None,
        }
    }

    /// Whether someone of `gender` may be called this.
    pub fn suits(self, gender: Gender) -> bool {
        self.gender().is_none_or(|needed| needed == gender)
    }

    /// Family, sharing the giver's household and family name.
    pub const fn is_kin(self) -> bool {
        !matches!(
            self,
            Self::Betrothed
                | Self::Friend
                | Self::Neighbour
                | Self::Apprentice
                | Self::Master
                | Self::Partner
        )
    }

    /// An older generation: these can have gone somewhere long ago.
    pub const fn is_elder(self) -> bool {
        matches!(
            self,
            Self::Grandmother | Self::Grandfather | Self::Grandparent | Self::Master
        )
    }
}

fact! {
    /// What a person (or a pet) is like.
    Trait as "trait" {
        Adventurous = "adventurous",
        Curious = "curious",
        Stubborn = "stubborn",
        Proud = "proud",
        Greedy = "greedy",
        Kind = "kind",
        Fearful = "fearful",
        Careless = "careless",
        Pious = "pious",
        Brave = "brave",
        HotTempered = "hot-tempered",
        Forgetful = "forgetful",
        Sickly = "sickly",
        Old = "old",
    }
}

fact! {
    /// What has become of someone. It only moves forward across a questline
    /// (`can_become`).
    State as "state" {
        Alive = "alive",
        Missing = "missing",
        Dead = "dead",
        Captive = "captive",
        Sick = "sick",
        Hurt = "hurt",
        /// Left for good, by choice.
        Gone = "gone",
    }
}

impl State {
    /// Whether a later step may say this of someone who is `self` now. The
    /// dead stay dead; anyone else may be found alive, hurt or dead.
    pub fn can_become(self, next: State) -> bool {
        self == next || self != State::Dead
    }
}

fact! {
    /// Someone unnamed who saw something or passed on a rumour ("a hunter").
    Passerby as "passerby" {
        Hunter = "hunter",
        Carter = "carter",
        Traveller = "traveller",
        Child = "child",
        Shepherd = "shepherd",
        Pedlar = "pedlar",
    }
}
