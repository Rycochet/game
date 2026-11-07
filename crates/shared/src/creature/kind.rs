//! The kinds of creature.

use serde::{Deserialize, Serialize};

use super::Temper;
use crate::sheet::{Feeling, Reaction};

/// A kind of creature. Saved by name, so new ones can go anywhere in the list.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum CreatureKind {
    Beggar,
    Birdfolk,
    Cattle,
    Centaur,
    Chicken,
    Drider,
    Dwarf,
    Elf,
    Giant,
    Goblin,
    Halfling,
    Horse,
    #[default]
    Human,
    Kobold,
    Lizardfolk,
    Minotaur,
    Orc,
    ScorpionMan,
    Wolf,
}

impl CreatureKind {
    /// How this kind feels about another, to begin with: it likes its own, and a
    /// hostile kind dislikes everyone else.
    pub fn feeling_about(self, other: Self) -> Feeling {
        use CreatureKind::{Beggar, Giant, Kobold};
        match (self, other) {
            // Has nothing against anyone, and hopes.
            (Beggar, _) => Feeling::Neutral,
            (one, two) if one == two => Feeling::Friendly,
            (Giant, Kobold) => Feeling::Unfriendly,
            _ if self.temper() == Temper::Hostile => Feeling::Unfriendly,
            _ => Feeling::Neutral,
        }
    }

    /// What this kind does about something it feels this way about, by its temper:
    /// a timid kind keeps away from everyone, friends least (how far is the
    /// server's); a friendly
    /// one comes over to anyone it doesn't dislike.
    pub fn reaction(self, feeling: Feeling) -> Reaction {
        match (self.temper(), feeling) {
            (_, Feeling::None) => Reaction::Ignore,
            // Even friends are kept a little way off (`server::entities::timid`).
            (Temper::Timid, _) => Reaction::Flee,
            (_, Feeling::Friendly) => Reaction::Approach,
            (Temper::Friendly, Feeling::Neutral) => Reaction::Approach,
            (_, Feeling::Neutral) => Reaction::Ignore,
            (_, Feeling::Unfriendly) => Reaction::Attack,
        }
    }
}
