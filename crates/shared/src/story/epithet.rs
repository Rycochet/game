//! Epithets for named bosses: values, so "the Grey" is worded by each language.

use serde::{Deserialize, Serialize};

use crate::lang::Selector;

/// What a boss is called after its name. Suits people and beasts alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Epithet {
    Grey,
    Black,
    White,
    Red,
    OneEyed,
    Scarred,
    Old,
    Cruel,
    Hungry,
    Silent,
    Mad,
    Great,
    Swift,
    Cunning,
    Pale,
    Bloody,
    // New epithets go at the end: messages carry one by its place.
}

impl Epithet {
    pub const ALL: [Self; 16] = [
        Self::Grey,
        Self::Black,
        Self::White,
        Self::Red,
        Self::OneEyed,
        Self::Scarred,
        Self::Old,
        Self::Cruel,
        Self::Hungry,
        Self::Silent,
        Self::Mad,
        Self::Great,
        Self::Swift,
        Self::Cunning,
        Self::Pale,
        Self::Bloody,
    ];
}

impl Selector for Epithet {
    fn fact(&self) -> &'static str {
        "epithet"
    }

    fn key(&self) -> &'static str {
        match self {
            Self::Grey => "grey",
            Self::Black => "black",
            Self::White => "white",
            Self::Red => "red",
            Self::OneEyed => "one-eyed",
            Self::Scarred => "scarred",
            Self::Old => "old",
            Self::Cruel => "cruel",
            Self::Hungry => "hungry",
            Self::Silent => "silent",
            Self::Mad => "mad",
            Self::Great => "great",
            Self::Swift => "swift",
            Self::Cunning => "cunning",
            Self::Pale => "pale",
            Self::Bloody => "bloody",
        }
    }
}
