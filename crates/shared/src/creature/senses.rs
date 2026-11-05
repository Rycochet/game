//! How a kind of creature takes to others, and how well it senses them, from the
//! creature research (`docs/context/research-creatures.md`).

use super::CreatureKind;

/// How a kind takes to those it doesn't know, before anything has happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Temper {
    /// Comes over to say hello.
    Friendly,
    /// Leaves others alone. The research's "varies" (humans, giants) is this too,
    /// until creatures of one kind can differ.
    Neutral,
    /// Keeps away from anyone it doesn't know.
    Timid,
    /// Minds its own patch: guarding it comes later, and until then it is `Neutral`.
    Territorial,
    /// Dislikes everyone not its own kind.
    Hostile,
}

/// How good one sense is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Acuity {
    None,
    Poor,
    Normal,
    Keen,
}

/// How well a kind sees without light: not used until light is a shared rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DarkSight {
    /// Needs light.
    None,
    /// Gets by in dim light.
    Low,
    /// Sees in the dark.
    Dark,
}

/// How well a kind senses what is round it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Senses {
    pub sight: Acuity,
    pub dark: DarkSight,
    pub hearing: Acuity,
    pub smell: Acuity,
}

impl Senses {
    /// How far it notices someone it has a clear line to, in cells (a diagonal
    /// counting as one). Without sight, only someone beside it.
    pub const fn sight_range(self) -> i64 {
        match self.sight {
            Acuity::None => 1,
            Acuity::Poor => 6,
            Acuity::Normal => 12,
            Acuity::Keen => 20,
        }
    }
}

impl CreatureKind {
    /// How it takes to those it doesn't know.
    pub const fn temper(self) -> Temper {
        use CreatureKind::*;
        match self {
            Halfling => Temper::Friendly,
            Chicken | Horse => Temper::Timid,
            Lizardfolk => Temper::Territorial,
            Goblin | Kobold | Minotaur | Orc => Temper::Hostile,
            Beggar | Birdfolk | Centaur | Dwarf | Elf | Giant | Human => Temper::Neutral,
        }
    }

    /// How well it senses what is round it.
    pub const fn senses(self) -> Senses {
        use Acuity::{Keen, Normal, Poor};
        use CreatureKind::*;
        const fn senses(sight: Acuity, dark: DarkSight, hearing: Acuity, smell: Acuity) -> Senses {
            Senses {
                sight,
                dark,
                hearing,
                smell,
            }
        }
        match self {
            Beggar | Human => senses(Normal, DarkSight::None, Normal, Poor),
            Birdfolk => senses(Keen, DarkSight::None, Normal, Poor),
            Centaur => senses(Keen, DarkSight::None, Normal, Normal),
            Chicken => senses(Normal, DarkSight::None, Normal, Poor),
            Dwarf => senses(Normal, DarkSight::Dark, Normal, Normal),
            Elf => senses(Keen, DarkSight::Low, Keen, Normal),
            Giant | Lizardfolk => senses(Normal, DarkSight::None, Normal, Normal),
            Goblin => senses(Normal, DarkSight::Dark, Keen, Normal),
            Halfling => senses(Normal, DarkSight::None, Keen, Normal),
            Horse => senses(Normal, DarkSight::Low, Keen, Keen),
            Kobold | Orc => senses(Normal, DarkSight::Dark, Normal, Keen),
            Minotaur => senses(Normal, DarkSight::Low, Normal, Keen),
        }
    }
}
