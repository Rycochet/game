//! What a creature is like: stats, skills, abilities and how it looks. See
//! `docs/context/shared/creature.md`.
//!
//! Stats and skills are for gameplay that doesn't exist yet, apart from what
//! strength says about carrying. Appearance never affects the rules.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::creature::{CreatureKind, Size};
use crate::energy::NORMAL_SPEED;
use crate::item::Inventory;

/// A creature's base stats.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub strength: u8,
    pub dexterity: u8,
    pub intelligence: u8,
    pub constitution: u8,
}

/// Something that can be learned, and be better or worse at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Skill {
    Climbing,
    Foraging,
    Running,
    Stealth,
    Swimming,
}

/// Something a creature can do because of what it is. No more than a name so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ability {
    Flutter,
    Kick,
    NightVision,
    Trample,
}

/// Something that differs between creatures of a kind, only to look at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Trait {
    CoatColour,
    FeatherColour,
    HairColour,
    Height,
    SkinShade,
}

impl Trait {
    /// The choices, in order.
    pub const fn choices(self) -> &'static [&'static str] {
        match self {
            Self::CoatColour => &["Bay", "Black", "Chestnut", "Grey", "White"],
            Self::FeatherColour => &["Brown", "Black", "Speckled", "White"],
            Self::HairColour => &["Black", "Brown", "Blonde", "Red", "Grey", "White"],
            Self::Height => &["Average", "Very short", "Short", "Tall", "Very tall"],
            Self::SkinShade => &["Medium", "Pale", "Light", "Tan", "Brown", "Dark"],
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::CoatColour => "Coat",
            Self::FeatherColour => "Feathers",
            Self::HairColour => "Hair",
            Self::Height => "Height",
            Self::SkinShade => "Skin",
        }
    }
}

/// How one creature looks: a choice for each trait, the first if not given.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appearance(pub BTreeMap<Trait, u8>);

impl Appearance {
    /// Whether every choice is for a trait this kind has, and is one of its choices.
    pub fn suits(&self, kind: CreatureKind) -> bool {
        self.0.iter().all(|(&which, &choice)| {
            kind.def().looks.contains(&which) && (choice as usize) < which.choices().len()
        })
    }

    /// What this trait is, in words.
    pub fn says(&self, which: Trait) -> &'static str {
        let choice = self.0.get(&which).copied().unwrap_or(0);
        which.choices().get(choice as usize).copied().unwrap_or("")
    }
}

/// How one creature feels about another: what its token's colour says to them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feeling {
    /// It doesn't feel anything: another player.
    #[default]
    None,
    Friendly,
    Neutral,
    Unfriendly,
}

impl Feeling {
    /// Where this feeling starts on the scale memories shift.
    pub fn warmth(self) -> i32 {
        match self {
            Self::None | Self::Neutral => 0,
            Self::Friendly => 50,
            Self::Unfriendly => -50,
        }
    }

    /// The feeling this far along the scale.
    pub fn from_warmth(warmth: i32) -> Self {
        match warmth {
            25.. => Self::Friendly,
            ..=-25 => Self::Unfriendly,
            _ => Self::Neutral,
        }
    }
}

/// What a creature does on noticing another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reaction {
    Ignore,
    /// Goes up to it, and stays nearby.
    Approach,
    /// Runs at it. Nothing happens on arriving until there is combat.
    Attack,
    Flee,
}

/// How weighed down someone is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Load {
    Light,
    Burdened,
    Overloaded,
}

/// What one creature is like: starts from its kind, and can grow from there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sheet {
    pub stats: Stats,
    pub skills: BTreeMap<Skill, u8>,
    pub appearance: Appearance,
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new(CreatureKind::default())
    }
}

impl Sheet {
    /// A plain one of this kind.
    pub fn new(kind: CreatureKind) -> Self {
        let def = kind.def();
        Self {
            stats: def.stats,
            skills: def.skills.iter().copied().collect(),
            appearance: Appearance::default(),
        }
    }

    /// How much it can carry before it is overloaded, in grams: by strength, and
    /// more the bigger it is.
    pub fn capacity(&self, size: Size) -> u32 {
        const GRAMS_PER_STRENGTH_AND_SIZE: u32 = 1000;
        self.stats.strength as u32 * (size as u32 + 1) * GRAMS_PER_STRENGTH_AND_SIZE
    }

    /// How weighed down this makes it: burdened past half what it can carry.
    pub fn load(&self, carrying: &Inventory) -> Load {
        let capacity = self.capacity(carrying.creature.def().size);
        let weight = carrying.total_weight();
        if weight > capacity {
            Load::Overloaded
        } else if weight * 2 > capacity {
            Load::Burdened
        } else {
            Load::Light
        }
    }
}

impl Load {
    /// How much of its speed something keeps, as a percentage.
    pub fn speed_percent(self) -> u32 {
        match self {
            Self::Light => 100,
            Self::Burdened => 75,
            Self::Overloaded => 40,
        }
    }
}

impl Sheet {
    /// How much energy it gains per tick: its kind's speed, slowed by what it carries.
    pub fn speed(&self, carrying: &Inventory) -> u32 {
        let kind = carrying.creature.def().speed;
        (NORMAL_SPEED * kind / 100 * self.load(carrying).speed_percent() / 100).max(1)
    }
}

#[cfg(test)]
mod tests;
