//! The kinds of creature.

use serde::{Deserialize, Serialize};

use crate::sheet::{Feeling, Reaction};

/// A kind of creature. Saved by name, so new ones can go anywhere in the list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatureKind {
    Beggar,
    Birdfolk,
    Centaur,
    Chicken,
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
}

impl CreatureKind {
    /// How this kind feels about another, to begin with. A kind likes its own.
    pub fn feeling_about(self, other: Self) -> Feeling {
        use CreatureKind::{Beggar, Giant, Goblin, Kobold};
        match (self, other) {
            // Has nothing against anyone, and hopes.
            (Beggar, _) => Feeling::Neutral,
            (one, two) if one == two => Feeling::Friendly,
            (Giant, Kobold) | (Kobold | Goblin, _) => Feeling::Unfriendly,
            _ => Feeling::Neutral,
        }
    }

    /// What this kind does about something it feels this way about.
    pub fn reaction(self, feeling: Feeling) -> Reaction {
        use CreatureKind::{Chicken, Horse};
        match (self, feeling) {
            (_, Feeling::None) => Reaction::Ignore,
            (Chicken | Horse, Feeling::Friendly) => Reaction::Ignore,
            (_, Feeling::Friendly) => Reaction::Approach,
            (Chicken, _) => Reaction::Flee,
            (Horse, Feeling::Unfriendly) => Reaction::Flee,
            (_, Feeling::Unfriendly) => Reaction::Attack,
            (_, Feeling::Neutral) => Reaction::Ignore,
        }
    }
}
