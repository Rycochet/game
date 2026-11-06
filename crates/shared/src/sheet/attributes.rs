//! The nine attributes: what a body, mind and spirit are. See
//! `docs/context/abilities-skills.md`.

use serde::{Deserialize, Serialize};

/// One side of a creature: a row of three attributes, and one triangle when
/// choosing them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Row {
    Body,
    Mind,
    Spirit,
}

impl Row {
    pub const ALL: [Self; 3] = [Self::Body, Self::Mind, Self::Spirit];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Body => "Body",
            Self::Mind => "Mind",
            Self::Spirit => "Spirit",
        }
    }

    /// Its three attributes, in column order: power, finesse, resistance.
    pub const fn attributes(self) -> [Attribute; 3] {
        use Attribute::*;
        match self {
            Self::Body => [Strength, Agility, Toughness],
            Self::Mind => [Intellect, Perception, Will],
            Self::Spirit => [Presence, Empathy, Aura],
        }
    }
}

/// One of the nine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Attribute {
    Strength,
    Agility,
    Toughness,
    Intellect,
    Perception,
    Will,
    Presence,
    Empathy,
    Aura,
}

impl Attribute {
    /// Row by row, in column order.
    pub const ALL: [Self; 9] = [
        Self::Strength,
        Self::Agility,
        Self::Toughness,
        Self::Intellect,
        Self::Perception,
        Self::Will,
        Self::Presence,
        Self::Empathy,
        Self::Aura,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Strength => "Strength",
            Self::Agility => "Agility",
            Self::Toughness => "Toughness",
            Self::Intellect => "Intellect",
            Self::Perception => "Perception",
            Self::Will => "Will",
            Self::Presence => "Presence",
            Self::Empathy => "Empathy",
            Self::Aura => "Aura",
        }
    }
}

/// A value for each of the nine. A kind's medians are near 100 (a person's are
/// 100), so there is room for small differences and slow growth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: u16,
    pub agility: u16,
    pub toughness: u16,
    pub intellect: u16,
    pub perception: u16,
    pub will: u16,
    pub presence: u16,
    pub empathy: u16,
    pub aura: u16,
}

impl Attributes {
    /// Every one the same.
    pub const fn all(value: u16) -> Self {
        Self {
            strength: value,
            agility: value,
            toughness: value,
            intellect: value,
            perception: value,
            will: value,
            presence: value,
            empathy: value,
            aura: value,
        }
    }

    pub const fn get(&self, which: Attribute) -> u16 {
        match which {
            Attribute::Strength => self.strength,
            Attribute::Agility => self.agility,
            Attribute::Toughness => self.toughness,
            Attribute::Intellect => self.intellect,
            Attribute::Perception => self.perception,
            Attribute::Will => self.will,
            Attribute::Presence => self.presence,
            Attribute::Empathy => self.empathy,
            Attribute::Aura => self.aura,
        }
    }

    pub fn set(&mut self, which: Attribute, value: u16) {
        *match which {
            Attribute::Strength => &mut self.strength,
            Attribute::Agility => &mut self.agility,
            Attribute::Toughness => &mut self.toughness,
            Attribute::Intellect => &mut self.intellect,
            Attribute::Perception => &mut self.perception,
            Attribute::Will => &mut self.will,
            Attribute::Presence => &mut self.presence,
            Attribute::Empathy => &mut self.empathy,
            Attribute::Aura => &mut self.aura,
        } = value;
    }
}
