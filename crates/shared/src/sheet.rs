//! What a creature is like: attributes, skills, abilities and how it looks. See
//! `docs/context/shared/creature.md` and `docs/context/abilities-skills.md`.
//!
//! Attributes and skills are for gameplay that doesn't exist yet, apart from what
//! strength says about carrying. Appearance never affects the rules.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::creature::{CreatureKind, Size};
use crate::energy::NORMAL_SPEED;
use crate::item::Inventory;
use crate::vitals::Vitals;

mod attributes;
mod build;
mod from_build;
mod hair;
mod kind_bonus;
mod looks;

pub use attributes::{Attribute, Attributes, Row};
pub use build::{Build, Dot, POINTS_TOTAL, TURN_STEPS, WEIGHT_TOTAL};
pub use from_build::{POINT_WEIGHT, SPREAD};
pub use kind_bonus::{BONUS_POINTS, bonus_from};
pub use looks::{Choice, Pattern};

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
    // New traits go at the end: messages carry a trait by its place in this list.
    HairStyle,
    Beard,
}

impl Trait {
    pub const fn name(self) -> &'static str {
        match self {
            Self::CoatColour => "Coat",
            Self::FeatherColour => "Feathers",
            Self::HairColour => "Hair",
            Self::Height => "Height",
            Self::SkinShade => "Skin",
            Self::HairStyle => "Hair style",
            Self::Beard => "Beard",
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
            kind.def().looks.contains(&which) && (choice as usize) < which.choices_for(kind).len()
        })
    }

    /// What is chosen for this trait, if this kind has it: the first if not given.
    pub fn choice(&self, kind: CreatureKind, which: Trait) -> Option<Choice> {
        if !kind.def().looks.contains(&which) {
            return None;
        }
        let chosen = self.0.get(&which).copied().unwrap_or(0);
        which.choices_for(kind).get(chosen as usize).copied()
    }

    /// What this trait is, in words.
    pub fn says(&self, kind: CreatureKind, which: Trait) -> &'static str {
        self.choice(kind, which).map_or("", |choice| choice.name)
    }

    /// The colour its body is drawn in: its skin, or its coat for one with fur.
    pub fn skin(&self, kind: CreatureKind) -> Option<[u8; 3]> {
        [Trait::SkinShade, Trait::CoatColour]
            .into_iter()
            .find_map(|which| self.choice(kind, which)?.colour)
    }

    /// The colour of its coat, if it has one: a taur's lower body's.
    pub fn coat(&self, kind: CreatureKind) -> Option<[u8; 3]> {
        self.choice(kind, Trait::CoatColour)?.colour
    }

    /// The colour of its feathers, if it has any.
    pub fn feathers(&self, kind: CreatureKind) -> Option<[u8; 3]> {
        self.choice(kind, Trait::FeatherColour)?.colour
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
    /// What it is like now. Missing from saves made before there were nine
    /// (`settled` fills it in).
    #[serde(default)]
    pub attributes: Attributes,
    /// How its attributes were chosen, kept to show and to work them out again.
    #[serde(default)]
    pub build: Build,
    pub skills: BTreeMap<Skill, u8>,
    pub appearance: Appearance,
    /// Health and Mana: full in a save from before them.
    #[serde(default)]
    pub vitals: Vitals,
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new(CreatureKind::default())
    }
}

impl Sheet {
    /// A plain one of this kind: its dots in the middle, Body at the top.
    pub fn new(kind: CreatureKind) -> Self {
        Self::built(kind, Build::CENTRED)
    }

    /// One of this kind, its attributes chosen with these triangles, which must be
    /// valid (`Build::is_valid`).
    pub fn built(kind: CreatureKind, build: Build) -> Self {
        Self {
            attributes: Attributes::for_kind(kind, &build),
            build,
            skills: kind.def().skills.iter().copied().collect(),
            appearance: Appearance::default(),
            vitals: Vitals::default(),
        }
    }

    /// As loaded: a save from before the nine attributes has none, so they are
    /// worked out again from its build (the centre, for such a save).
    pub fn settled(mut self, kind: CreatureKind) -> Self {
        if self.attributes == Attributes::default() {
            if !self.build.is_valid() {
                self.build = Build::CENTRED;
            }
            self.attributes = Attributes::for_kind(kind, &self.build);
        }
        self
    }

    /// How much it can carry before it is overloaded, in grams: by strength, and
    /// more the bigger it is. A person of median strength (100) carries 30 kg.
    pub fn capacity(&self, size: Size) -> u32 {
        const GRAMS_PER_STRENGTH_AND_SIZE: u32 = 100;
        self.attributes.strength as u32 * (size as u32 + 1) * GRAMS_PER_STRENGTH_AND_SIZE
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
