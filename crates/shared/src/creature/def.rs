//! What a kind of creature is like: its body parts, size and attributes.

use super::{Layer, Location, Size};
use crate::sheet::{Ability, Attributes, Skill, Trait};

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
    /// What being this kind adds on top of what is chosen or rolled: some, all
    /// or none of the nine.
    pub bonus: Attributes,
    /// Skills it starts with, and how good at them.
    pub skills: &'static [(Skill, u8)],
    /// What it can do because of what it is.
    pub abilities: &'static [Ability],
    /// What can differ between creatures of this kind, to look at.
    pub looks: &'static [Trait],
}

impl CreatureDef {
    pub fn part(&self, location: Location) -> Option<&'static BodyPart> {
        self.body.iter().find(|part| part.location == location)
    }

    /// How many fingers it has: none if it has no `Fingers`.
    pub fn fingers(&self) -> Option<u8> {
        self.part(Location::Fingers).map(|part| part.places)
    }
}
