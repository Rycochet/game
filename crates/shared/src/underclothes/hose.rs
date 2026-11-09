//! Hose: close-fitting legs of bias-cut cloth, tied up to the braies belt, pointed to
//! the doublet or gartered. Footed hose cover the feet too, so no socks go with them;
//! footless and stirrup hose leave the feet bare (owner, 2026-11-09).

use serde::{Deserialize, Serialize};

use super::table::{has, when};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoseFeet {
    Footed,
    /// A strap under the foot, the foot bare.
    Stirrup,
    Footless,
}

impl HoseFeet {
    /// Whether the feet are covered, so the item is worn on them.
    pub fn covers_feet(self) -> bool {
        self == Self::Footed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HoseHeight {
    Knee,
    Thigh,
    /// Joined legs to the waist (late medieval).
    Waist,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoseMake {
    /// Woven cloth cut on the bias, seamed up the back.
    BiasCut,
    /// The earliest leggings and stockings.
    Leather,
    /// Knitted (16th century).
    Knitted,
}

/// What holds them up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Held {
    /// Tied to the braies belt.
    Ties,
    /// Pointed to the doublet.
    Points,
    /// Gartered below the knee.
    Garters,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Colours {
    Plain,
    Striped,
    /// Each leg a different colour.
    Parti,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hose {
    pub feet: HoseFeet,
    pub height: HoseHeight,
    pub make: HoseMake,
    /// A leather sole sewn under, for going without shoes.
    pub sole: bool,
    pub held: Held,
    pub colours: Colours,
    /// Joined hose only (late 15th to 16th century).
    pub codpiece: bool,
}

const FEET: [(HoseFeet, u64); 3] = [
    (HoseFeet::Footed, 65),
    (HoseFeet::Stirrup, 20),
    (HoseFeet::Footless, 15),
];

fn heights(feet: HoseFeet) -> [(HoseHeight, u64); 3] {
    [
        (HoseHeight::Knee, 30),
        (HoseHeight::Thigh, 45),
        (HoseHeight::Waist, when(feet != HoseFeet::Footless, 25)),
    ]
}

fn makes(feet: HoseFeet, height: HoseHeight) -> [(HoseMake, u64); 3] {
    let short = height != HoseHeight::Waist;
    [
        (HoseMake::BiasCut, 75),
        (
            HoseMake::Leather,
            when(short && feet != HoseFeet::Stirrup, 10),
        ),
        (HoseMake::Knitted, when(short, 15)),
    ]
}

fn helds(height: HoseHeight) -> &'static [(Held, u64)] {
    match height {
        HoseHeight::Knee => &[(Held::Garters, 1)],
        HoseHeight::Thigh => &[(Held::Ties, 80), (Held::Garters, 20)],
        HoseHeight::Waist => &[(Held::Points, 1)],
    }
}

fn colourings(make: HoseMake, height: HoseHeight) -> [(Colours, u64); 3] {
    match make {
        HoseMake::BiasCut => [
            (Colours::Plain, 60),
            (Colours::Striped, 15),
            (Colours::Parti, when(height != HoseHeight::Knee, 25)),
        ],
        HoseMake::Knitted => [
            (Colours::Plain, 80),
            (Colours::Striped, 20),
            (Colours::Parti, 0),
        ],
        HoseMake::Leather => [
            (Colours::Plain, 1),
            (Colours::Striped, 0),
            (Colours::Parti, 0),
        ],
    }
}

impl Hose {
    fn may_be_soled(&self) -> bool {
        self.feet == HoseFeet::Footed && self.make != HoseMake::Knitted
    }

    pub(super) fn roll(rng: &mut Rng) -> Self {
        let feet = rng.pick_weighted(&FEET);
        let height = rng.pick_weighted(&heights(feet));
        let make = rng.pick_weighted(&makes(feet, height));
        let mut hose = Self {
            feet,
            height,
            make,
            sole: false,
            held: rng.pick_weighted(helds(height)),
            colours: rng.pick_weighted(&colourings(make, height)),
            codpiece: height == HoseHeight::Waist && rng.range_u64(0, 2) == 0,
        };
        hose.sole = hose.may_be_soled() && rng.range_u64(0, 6) == 0;
        hose
    }

    pub(super) fn allowed(&self) -> bool {
        has(&FEET, self.feet)
            && has(&heights(self.feet), self.height)
            && has(&makes(self.feet, self.height), self.make)
            && (!self.sole || self.may_be_soled())
            && has(helds(self.height), self.held)
            && has(&colourings(self.make, self.height), self.colours)
            && (!self.codpiece || self.height == HoseHeight::Waist)
    }

    pub(super) fn name(&self) -> &'static str {
        match self.feet {
            HoseFeet::Footless => "Leggings",
            HoseFeet::Stirrup => "Stirrup Hose",
            HoseFeet::Footed if self.sole => "Soled Hose",
            HoseFeet::Footed if self.height == HoseHeight::Waist => "Joined Hose",
            HoseFeet::Footed if self.held == Held::Garters && self.height == HoseHeight::Knee => {
                "Stockings"
            }
            HoseFeet::Footed => "Hose",
        }
    }

    /// Grams for a pair of woven wool at an ordinary size, the sole and codpiece left out.
    pub(super) fn base_grams(&self) -> u32 {
        let legs: u32 = match self.height {
            HoseHeight::Knee => 160,
            HoseHeight::Thigh => 250,
            HoseHeight::Waist => 380,
        };
        let feet = match self.feet {
            HoseFeet::Footed => 0,
            HoseFeet::Stirrup => 20,
            HoseFeet::Footless => 40,
        };
        legs - feet
    }

    /// Grams the sole and codpiece add.
    pub(super) fn extra_grams(&self) -> u32 {
        let sole = if self.sole { 150 } else { 0 };
        let codpiece = if self.codpiece { 40 } else { 0 };
        sole + codpiece
    }

    /// Whether it needs a second colour.
    pub(super) fn two_coloured(&self) -> bool {
        self.colours != Colours::Plain
    }
}
