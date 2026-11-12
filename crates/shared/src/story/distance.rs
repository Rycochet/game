//! How far a place lies from a town, in bands a story can speak of.

use serde::{Deserialize, Serialize};

use super::offset;
use crate::lang::Selector;
use crate::poi::CHUNK_SIZE;

/// Within this many world map points (straight, the near way round) a place is
/// Near: about a chunk, in sight of town. *To balance.*
pub const NEAR_WITHIN: i64 = CHUNK_SIZE;
/// Within this many, a Short walk. *To balance.*
pub const SHORT_WITHIN: i64 = 3 * CHUNK_SIZE;
/// Within this many, most of a Day; beyond it, Far. *To balance.*
pub const DAY_WITHIN: i64 = 10 * CHUNK_SIZE;

/// How far away a place is, roughly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DistanceBand {
    /// In sight of town.
    Near,
    /// A short walk.
    Short,
    /// Most of a day on foot.
    Day,
    /// Days away.
    Far,
    // New values go at the end: messages carry one by its place.
}

impl DistanceBand {
    pub const ALL: [Self; 4] = [Self::Near, Self::Short, Self::Day, Self::Far];

    /// The band from world map position `from` to `to`, on a planet `width`
    /// points round. Whole numbers only (squared distances).
    pub fn between(from: (i64, i64), to: (i64, i64), width: i64) -> Self {
        let (dx, dy) = offset(from, to, width);
        let squared = i128::from(dx).pow(2) + i128::from(dy).pow(2);
        let within = |limit: i64| squared <= i128::from(limit).pow(2);
        if within(NEAR_WITHIN) {
            Self::Near
        } else if within(SHORT_WITHIN) {
            Self::Short
        } else if within(DAY_WITHIN) {
            Self::Day
        } else {
            Self::Far
        }
    }
}

impl Selector for DistanceBand {
    fn fact(&self) -> &'static str {
        "distance"
    }

    fn key(&self) -> &'static str {
        match self {
            Self::Near => "near",
            Self::Short => "short",
            Self::Day => "day",
            Self::Far => "far",
        }
    }
}
