//! What can be told of something on the ground from afar: how full its bag looks.
//! What is in it is only told to someone within reach of it. See
//! `docs/context/shared/ground.md`.

use serde::{Deserialize, Serialize};

use crate::item::Item;

mod reach;

pub use reach::{ARM_REACH, in_reach, nearest_in_reach, reach};

/// How full a bag on the ground looks, by the weight of all in it. Saved in what
/// characters remember.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Load {
    /// Almost flat.
    Flat,
    Slack,
    Half,
    Full,
    Bulging,
}

impl Load {
    pub const ALL: [Self; 5] = [
        Self::Flat,
        Self::Slack,
        Self::Half,
        Self::Full,
        Self::Bulging,
    ];

    /// How full a bag this heavy looks, in grams. First guesses.
    pub fn of_weight(grams: u32) -> Self {
        match grams {
            0..200 => Self::Flat,
            200..1_000 => Self::Slack,
            1_000..5_000 => Self::Half,
            5_000..15_000 => Self::Full,
            _ => Self::Bulging,
        }
    }

    /// How full a bag holding these looks.
    pub fn of(items: &[Item]) -> Self {
        let grams = items.iter().map(Item::total_weight).sum();
        Self::of_weight(grams)
    }
}

#[cfg(test)]
mod tests;
