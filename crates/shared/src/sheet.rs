//! What a creature is like: attributes, skills, abilities and how it looks. See
//! `docs/context/shared/creature.md` and `docs/context/abilities-skills.md`.
//!
//! Attributes and skills are for gameplay that doesn't exist yet, apart from what
//! strength says about carrying. Appearance never affects the rules.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::creature::{CreatureKind, Size};
use crate::energy::NORMAL_SPEED;
use crate::gender::Gender;
use crate::item::Inventory;
use crate::movement::{FULL, MOVEMENT_SKILLS};
use crate::skill::Skills;
use crate::vitals::Vitals;

mod attributes;
mod build;
mod foes;
mod from_build;
mod hair;
mod kind_bonus;
mod looks;
mod shapeshift;

pub use crate::ability::Ability;
pub use attributes::{Attribute, Attributes, Row};
pub use build::{Build, Dot, POINTS_TOTAL, TURN_STEPS, WEIGHT_TOTAL};
pub use foes::{Foe, Foes};
pub use from_build::{POINT_WEIGHT, SPREAD};
pub use kind_bonus::{BONUS_POINTS, bonus_from};
pub use looks::{Choice, Pattern};
pub use shapeshift::TrueForm;

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
    /// Its gender (`shared::gender`), whose bonus is in its attributes. Neutral in
    /// a save from before genders, whose attributes are kept with no bonus added
    /// (owner, 2026-11-12).
    #[serde(default)]
    pub gender: Gender,
    /// What it has learned (`skill::Skills`): saves from before the tree are read
    /// by the old names.
    pub skills: Skills,
    pub appearance: Appearance,
    /// Health and Mana: full in a save from before them.
    #[serde(default)]
    pub vitals: Vitals,
    /// What it really is while shapeshifted into another kind's body (`shapeshift`).
    #[serde(default)]
    pub true_form: Option<TrueForm>,
    /// What it remembers of each kind of creature it has fought (`Foes`).
    #[serde(default)]
    pub foes: Foes,
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new(CreatureKind::default())
    }
}

impl Sheet {
    /// A plain one of this kind: Neutral, its dots in the middle, Body at the top.
    pub fn new(kind: CreatureKind) -> Self {
        Self::built(kind, Gender::default(), Build::CENTRED)
    }

    /// One of this kind and gender, its attributes chosen with these triangles,
    /// which must be valid (`Build::is_valid`).
    pub fn built(kind: CreatureKind, gender: Gender, build: Build) -> Self {
        Self {
            attributes: Attributes::for_kind(kind, gender, &build),
            build,
            gender,
            skills: Skills::starting(kind),
            appearance: Appearance::default(),
            vitals: Vitals::default(),
            true_form: None,
            foes: Foes::default(),
        }
    }

    /// As loaded, its body of kind `kind`: a save from before the nine attributes has
    /// none, so they are worked out again from its build (the centre, for such a save).
    pub fn settled(mut self, kind: CreatureKind) -> Self {
        if self.attributes == Attributes::default() {
            if !self.build.is_valid() {
                self.build = Build::CENTRED;
            }
            // Its own kind's, shapeshifted again if it was.
            let own = self.own_kind(kind);
            self.true_form = None;
            self.attributes = Attributes::for_kind(own, self.gender, &self.build);
            if own != kind {
                self.shapeshift(own, kind);
            }
        }
        self
    }

    /// How much it can carry before it is overloaded, in grams: by strength, and
    /// more the bigger it is. A person of median strength (100) carries 30 kg,
    /// and Weight Training adds its value as a share of that: twice as much at
    /// 100% (owner).
    pub fn capacity(&self, size: Size) -> u32 {
        const GRAMS_PER_STRENGTH_AND_SIZE: u32 = 100;
        let base =
            self.attributes.strength as u32 * (size as u32 + 1) * GRAMS_PER_STRENGTH_AND_SIZE;
        let trained = self.skills.value(MOVEMENT_SKILLS.weight_training).min(FULL);
        (u64::from(base) * u64::from(FULL + trained) / u64::from(FULL)) as u32
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
