//! A creature's gender: Male, Female or Neutral, for every creature, animals too.
//! Each gives a small bonus to one row of the attribute grid (owner, 2026-11-12: "a
//! subtle nudge"), and later text takes its pronouns from it. See
//! `docs/context/shared/gender.md`.

use serde::{Deserialize, Serialize};

use crate::sheet::{Attributes, Row};

/// How much a gender adds to each of the three attributes of its row. The same
/// for every gender, so each adds the same total. *To balance.*
pub const GENDER_BONUS: u16 = 2;

/// A creature's gender. Neutral is the default, and what saves from before
/// genders load as.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Gender {
    Male,
    Female,
    #[default]
    Neutral,
    // New genders go at the end: messages and saves carry one by its place.
}

impl Gender {
    pub const ALL: [Self; 3] = [Self::Male, Self::Female, Self::Neutral];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Male => "Male",
            Self::Female => "Female",
            Self::Neutral => "Neutral",
        }
    }

    /// The row of attributes its bonus goes to.
    pub const fn row(self) -> Row {
        match self {
            Self::Male => Row::Body,
            Self::Female => Row::Mind,
            Self::Neutral => Row::Spirit,
        }
    }

    /// What it adds to each attribute: `GENDER_BONUS` on its row's three.
    pub fn bonus(self) -> Attributes {
        let mut bonus = Attributes::default();
        for which in self.row().attributes() {
            bonus.set(which, GENDER_BONUS);
        }
        bonus
    }

    /// One rolled for a mob from a hash: 45% Male, 45% Female, 10% Neutral.
    pub fn rolled(hash: u64) -> Self {
        match hash % 100 {
            0..45 => Self::Male,
            45..90 => Self::Female,
            _ => Self::Neutral,
        }
    }
}

#[cfg(test)]
mod tests;
