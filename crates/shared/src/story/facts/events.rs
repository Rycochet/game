//! Facts about what happened: why someone went, how long ago, and what is at
//! stake now.

use super::super::fact::fact;
use crate::story::PlaceKind;

fact! {
    /// Why someone went to the place. Each place kind allows some (`allowed_at`).
    Errand as "errand" {
        Treasure = "treasure",
        Herbs = "herbs",
        Hunting = "hunting",
        Graveside = "graveside",
        Trade = "trade",
        Shelter = "shelter",
        Work = "work",
        Pilgrimage = "pilgrimage",
        Curiosity = "curiosity",
        Rescue = "rescue",
        Eloping = "eloping",
    }
}

impl Errand {
    /// Whether someone could have gone to a place of this kind for this.
    pub fn allowed_at(self, place: PlaceKind) -> bool {
        use PlaceKind::*;
        let places: &[PlaceKind] = match self {
            Self::Treasure => &[Ruin, Pyramid, Crypt, Cave, Tower, Cellar],
            Self::Herbs => &[Wilds, Cave, Oasis, HerbalistHut],
            Self::Hunting => &[Wilds, Camp, Cave],
            Self::Graveside => &[Grave, Crypt],
            Self::Trade => &[Settlement, Store, Camp, Farm],
            Self::Shelter => &[Cave, Tower, Ruin, Dwelling, Cellar],
            Self::Work => &[Farm, Stronghold, Tower, Store],
            Self::Pilgrimage => &[Pyramid, Tower, Grave, Oasis],
            Self::Eloping => &[Settlement, Dwelling],
            Self::Curiosity | Self::Rescue => return true,
        };
        places.contains(&place)
    }
}

fact! {
    /// How long ago something happened.
    TimeAgo as "time-ago" {
        Days = "days",
        Weeks = "weeks",
        LastSeason = "last-season",
        Years = "years",
        /// A generation: nobody who went then is still alive there.
        LongAgo = "long-ago",
    }
}

fact! {
    /// An occasion something is wanted for.
    Occasion as "occasion" {
        Wedding = "wedding",
        Birth = "birth",
        Funeral = "funeral",
        Festival = "festival",
        ComingOfAge = "coming-of-age",
        NameDay = "name-day",
        Harvest = "harvest",
        Winter = "winter",
    }
}

fact! {
    /// What the giver stands to keep or lose.
    Stake as "stake" {
        Memory = "memory",
        Safety = "safety",
        Livelihood = "livelihood",
        Honour = "honour",
        Promise = "promise",
        Occasion = "occasion",
        Child = "child",
        Love = "love",
    }
}
