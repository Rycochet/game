//! What a kind of creature is like: its body parts, size and attributes.

use super::{Layer, Location, Size};
use crate::sheet::{Ability, Attribute, Attributes, Trait, bonus_from};

/// One location on a body: which layers go there, and how many places it has for
/// things that take only one (fingers for rings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyPart {
    pub location: Location,
    pub layers: &'static [Layer],
    pub places: u8,
}

/// What a kind of creature is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureDef {
    pub name: &'static str,
    /// What its body weighs, in grams, with nothing on it.
    pub weight: u32,
    pub size: Size,
    /// Its base grid: what it can carry with no bag.
    pub inventory: (u8, u8),
    pub body: &'static [BodyPart],
    /// How fast it goes for its kind, as a percentage of ordinary.
    pub speed: u32,
    /// Whether it can go up and down stairs, so as to follow someone.
    pub uses_stairs: bool,
    /// Whether someone can choose to play one.
    pub playable: bool,
    /// The middle of its range for each attribute, before its bonus.
    pub attributes: Attributes,
    /// The bonuses this kind sets; the rest of `BONUS_POINTS` is shared among
    /// the others (`bonus`). Setting one to 0 keeps it from any share.
    pub bonus_set: &'static [(Attribute, u16)],
    /// Skills it starts with: paths in the skill tree, and how much practice
    /// (`skill::Skills::starting`): a level's cube times 1,000
    /// (`skill::level_practice`), so 27,000 is 30%.
    pub skills: &'static [(&'static str, u32)],
    /// What it can do because of what it is.
    pub abilities: &'static [Ability],
    /// What can differ between creatures of this kind, to look at.
    pub looks: &'static [Trait],
}

impl CreatureDef {
    /// What being this kind adds on top of what is chosen or rolled: the same
    /// points in all for every kind (`sheet::bonus_from`).
    pub fn bonus(&self) -> Attributes {
        bonus_from(self.bonus_set)
    }

    pub fn part(&self, location: Location) -> Option<&'static BodyPart> {
        self.body.iter().find(|part| part.location == location)
    }

    /// How many fingers it has: none if it has no `Fingers`.
    pub fn fingers(&self) -> Option<u8> {
        self.part(Location::Fingers).map(|part| part.places)
    }
}
