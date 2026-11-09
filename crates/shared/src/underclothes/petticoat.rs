//! Petticoats and underskirts, gathered or tied at the waist. On the Waist alone, as
//! they hang from it, so stockings go under them (owner, 2026-11-09).

use serde::{Deserialize, Serialize};

use super::table::{has, when};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkirtLength {
    Knee,
    Calf,
    Ankle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkirtFullness {
    /// Gored to fit.
    Fitted,
    /// Gathered.
    Full,
    /// Wrapped or layered.
    VeryFull,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkirtMake {
    Plain,
    /// Quilted for warmth (late medieval on).
    Quilted,
    /// Horsehair, standing out (mamigun).
    Horsehair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkirtWaist {
    Drawstring,
    /// A waistband with ties.
    Ties,
    /// Wrapped round and tied (koshimaki, sokchima).
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Petticoat {
    pub length: SkirtLength,
    pub fullness: SkirtFullness,
    pub make: SkirtMake,
    pub waist: SkirtWaist,
}

const LENGTHS: [(SkirtLength, u64); 3] = [
    (SkirtLength::Knee, 15),
    (SkirtLength::Calf, 35),
    (SkirtLength::Ankle, 50),
];

const FULLNESS: [(SkirtFullness, u64); 3] = [
    (SkirtFullness::Fitted, 30),
    (SkirtFullness::Full, 50),
    (SkirtFullness::VeryFull, 20),
];

fn makes(fullness: SkirtFullness) -> [(SkirtMake, u64); 3] {
    [
        (SkirtMake::Plain, 80),
        (
            SkirtMake::Quilted,
            when(fullness != SkirtFullness::Fitted, 12),
        ),
        (
            SkirtMake::Horsehair,
            when(fullness == SkirtFullness::VeryFull, 8),
        ),
    ]
}

fn waists(fullness: SkirtFullness) -> [(SkirtWaist, u64); 3] {
    [
        (SkirtWaist::Drawstring, 50),
        (SkirtWaist::Ties, 35),
        (SkirtWaist::Wrap, when(fullness != SkirtFullness::Full, 15)),
    ]
}

impl Petticoat {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        let length = rng.pick_weighted(&LENGTHS);
        let fullness = rng.pick_weighted(&FULLNESS);
        Self {
            length,
            fullness,
            make: rng.pick_weighted(&makes(fullness)),
            waist: rng.pick_weighted(&waists(fullness)),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&LENGTHS, self.length)
            && has(&FULLNESS, self.fullness)
            && has(&makes(self.fullness), self.make)
            && has(&waists(self.fullness), self.waist)
    }

    pub(super) fn name(&self) -> &'static str {
        match (self.make, self.waist, self.fullness) {
            (SkirtMake::Horsehair, ..) => "Mamigun",
            (SkirtMake::Quilted, ..) => "Quilted Petticoat",
            (_, SkirtWaist::Wrap, SkirtFullness::Fitted) => "Koshimaki",
            (_, SkirtWaist::Wrap, _) => "Sokchima",
            (_, _, SkirtFullness::Fitted) => "Underskirt",
            _ => "Petticoat",
        }
    }

    pub(super) fn base_grams(&self) -> u32 {
        let skirt = match self.length {
            SkirtLength::Knee => 280,
            SkirtLength::Calf => 420,
            SkirtLength::Ankle => 560,
        };
        let fullness = match self.fullness {
            SkirtFullness::Fitted => 0,
            SkirtFullness::Full => 120,
            SkirtFullness::VeryFull => 300,
        };
        let total = skirt + fullness;
        match self.make {
            // Two layers with wadding between.
            SkirtMake::Quilted => total * 2,
            _ => total,
        }
    }
}
