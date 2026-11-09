//! Braies and drawers: loose linen on a belt through a turned-down waistband (or a
//! drawstring), the legs tucked into the hose. On the Waist alone, so hose go with
//! them (owner, 2026-11-09).

use serde::{Deserialize, Serialize};

use super::table::{has, when};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BraiesLength {
    /// Halfway to the knee (late medieval).
    Thigh,
    Knee,
    /// Halfway from the knee to the ankle.
    Calf,
    /// The earliest, to the ankle.
    Ankle,
}

/// How full the legs are cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Fit {
    Fitted,
    Loose,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Waistband {
    /// A belt (the braies girdle) through the turned-down top, which hose tie to.
    Belt,
    Drawstring,
}

/// How the legs end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ends {
    Open,
    /// Laced at the calf: long ones only.
    TiedAtCalf,
    /// Tucked into the hose (drawn so when hose are worn).
    Tucked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Braies {
    pub length: BraiesLength,
    pub fit: Fit,
    pub waist: Waistband,
    pub ends: Ends,
}

const LENGTHS: [(BraiesLength, u64); 4] = [
    (BraiesLength::Thigh, 10),
    (BraiesLength::Knee, 40),
    (BraiesLength::Calf, 30),
    (BraiesLength::Ankle, 20),
];

const FITS: [(Fit, u64); 3] = [(Fit::Fitted, 25), (Fit::Loose, 55), (Fit::Full, 20)];

const WAISTS: [(Waistband, u64); 2] = [(Waistband::Belt, 70), (Waistband::Drawstring, 30)];

fn ends(length: BraiesLength) -> [(Ends, u64); 3] {
    [
        (Ends::Open, 30),
        (Ends::TiedAtCalf, when(length >= BraiesLength::Calf, 20)),
        (Ends::Tucked, 50),
    ]
}

impl Braies {
    pub(super) fn roll(rng: &mut Rng) -> Self {
        let length = rng.pick_weighted(&LENGTHS);
        Self {
            length,
            fit: rng.pick_weighted(&FITS),
            waist: rng.pick_weighted(&WAISTS),
            ends: rng.pick_weighted(&ends(length)),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        has(&LENGTHS, self.length)
            && has(&FITS, self.fit)
            && has(&WAISTS, self.waist)
            && has(&ends(self.length), self.ends)
    }

    pub(super) fn name(&self) -> &'static str {
        match (self.length, self.waist) {
            (BraiesLength::Thigh, _) => "Short Braies",
            (BraiesLength::Knee, Waistband::Drawstring) => "Drawers",
            (_, Waistband::Drawstring) if self.fit == Fit::Full => "Sokgot",
            _ => "Braies",
        }
    }

    pub(super) fn base_grams(&self) -> u32 {
        let legs = match self.length {
            BraiesLength::Thigh => 150,
            BraiesLength::Knee => 220,
            BraiesLength::Calf => 280,
            BraiesLength::Ankle => 330,
        };
        match self.fit {
            Fit::Fitted => legs - 20,
            Fit::Loose => legs,
            Fit::Full => legs + 50,
        }
    }
}
