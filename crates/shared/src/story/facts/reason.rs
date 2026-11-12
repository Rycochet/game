//! What a quest asks (its goals) and why the giver asks it (the reason), with
//! how well each reason suits each goal and place, from the research
//! catalogue (`docs/context/research-story-plots.md`).

use super::super::fact::fact;
use crate::story::PlaceKind;

fact! {
    /// What a step of a quest asks the player to do.
    Goal as "goal" {
        Slay = "slay",
        Explore = "explore",
        Find = "find",
        Deliver = "deliver",
        Visit = "visit",
        Gather = "gather",
        Escort = "escort",
    }
}

fact! {
    /// Why a giver asks: the plot a story follows. A chain keeps its reason.
    Reason as "reason" {
        Rumour = "rumour",
        Family = "family",
        NeighbourFeud = "neighbour-feud",
        LostPet = "lost-pet",
        DebtOrTrade = "debt-or-trade",
        Threat = "threat",
        Grief = "grief",
        /// Owner: added to the catalogue's seven.
        Learning = "learning",
    }
}

/// How well a reason suits something: never, possible, or suits well.
const NEVER: u64 = 0;
const POSSIBLE: u64 = 1;
const WELL: u64 = 4;

impl Reason {
    /// The catalogue's table of reasons by goal (★ well, · possible).
    pub fn goal_weight(self, goal: Goal) -> u64 {
        use Goal::*;
        let (well, possible): (&[Goal], &[Goal]) = match self {
            Self::Rumour => (&[Explore, Find, Visit], &[Slay, Gather]),
            Self::Family => (&[Find, Escort], &[Slay, Explore, Deliver, Visit, Gather]),
            Self::NeighbourFeud => (&[Find], &[Slay, Deliver, Visit, Gather]),
            Self::LostPet => (&[Escort], &[Slay, Explore, Visit]),
            Self::DebtOrTrade => (&[Deliver, Gather], &[Slay, Find, Visit, Escort]),
            Self::Threat => (&[Slay, Visit], &[Explore, Deliver, Gather, Escort]),
            Self::Grief => (&[Find, Visit], &[Slay, Explore, Deliver, Gather]),
            Self::Learning => (&[Find, Visit, Gather], &[Explore, Deliver, Escort]),
        };
        if well.contains(&goal) {
            WELL
        } else if possible.contains(&goal) {
            POSSIBLE
        } else {
            NEVER
        }
    }

    /// Places the reason uses well count double; any other place still works.
    pub fn place_weight(self, place: PlaceKind) -> u64 {
        use PlaceKind::*;
        let good: &[PlaceKind] = match self {
            Self::Rumour => &[Ruin, Pyramid, Tower, Cave, Crypt, Oasis],
            Self::Family => &[Dwelling, Farm, Settlement, Cave, Ruin],
            Self::NeighbourFeud => &[Farm, Dwelling, Store, Settlement, Wilds],
            Self::LostPet => &[Wilds, Cave, Farm, Ruin, Dwelling],
            Self::DebtOrTrade => &[Settlement, Store, Farm, Camp, HerbalistHut],
            Self::Threat => &[Cave, Camp, Ruin, Stronghold, Wilds, Farm],
            Self::Grief => &[Grave, Crypt, Ruin, Cave, Stronghold, Wilds],
            Self::Learning => &[Tower, Pyramid, Ruin, Crypt, HerbalistHut],
        };
        if good.contains(&place) { 2 } else { 1 }
    }
}
