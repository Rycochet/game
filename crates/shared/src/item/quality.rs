//! `Quality`: how good an item is, and artifacts' names.

use serde::{Deserialize, Serialize};

use crate::item::{ItemKind, ODDS_OUT_OF};
use crate::rng::Rng;

/// How good an item is, worst first. The better, the rarer. Being part of a set is
/// apart from this: see `Item::set` and `decisions.md`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Quality {
    Poor,
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Artifact,
}

impl Quality {
    pub const ALL: [Quality; 7] = [
        Self::Poor,
        Self::Common,
        Self::Uncommon,
        Self::Rare,
        Self::Epic,
        Self::Legendary,
        Self::Artifact,
    ];

    /// How likely a roll is to give it, out of `ODDS_OUT_OF`. First guesses.
    pub const fn odds(self) -> u64 {
        match self {
            Self::Poor => 2500,
            Self::Common => 5000,
            Self::Uncommon => 1700,
            Self::Rare => 600,
            Self::Epic => 150,
            Self::Legendary => 45,
            Self::Artifact => 5,
        }
    }

    /// How many sizes from its own something magical of this quality fits perfectly.
    /// Nothing below uncommon can be magical.
    pub const fn magic_reach(self) -> u8 {
        match self {
            Self::Poor | Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
            Self::Legendary => 4,
            Self::Artifact => 5,
        }
    }

    /// The first of a drop's two rolls: how good it will be.
    pub fn roll(rng: &mut Rng) -> Self {
        let mut left = rng.range_u64(0, ODDS_OUT_OF);
        for quality in Self::ALL {
            if left < quality.odds() {
                return quality;
            }
            left -= quality.odds();
        }
        Self::Common
    }
}

/// A placeholder name for an artifact of this kind: artifacts are always named.
pub fn artifact_name(rng: &mut Rng, kind: ItemKind) -> String {
    const WORDS: [&str; 12] = [
        "Ashen",
        "Dawning",
        "Endless",
        "Forgotten",
        "Gilded",
        "Hollow",
        "Last",
        "Quiet",
        "Sunken",
        "Thousandth",
        "Wandering",
        "Whispering",
    ];
    let word = WORDS[rng.range_u64(0, WORDS.len() as u64) as usize];
    format!("The {word} {}", kind.def().name)
}
