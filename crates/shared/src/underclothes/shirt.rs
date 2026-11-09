//! Shirts, chemises, smocks, shifts and undertunics: rectangles and gussets of linen,
//! tucked under whatever goes over them. Stays are an option on the shirt itself
//! (owner, 2026-11-09), drawn over its body, so they always sit over the chemise.

use serde::{Deserialize, Serialize};

use super::table::{has, when};
use crate::rng::Rng;

/// Where the hem falls on a standing body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Length {
    Hip,
    /// Halfway from the hip to the knee.
    Thigh,
    Knee,
    /// Halfway from the knee to the ankle.
    Calf,
    Ankle,
}

impl Length {
    /// Calf or longer: a chemise rather than a shirt.
    pub fn long(self) -> bool {
        self >= Self::Calf
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Sleeves {
    /// Sleeveless: the arms are bare.
    None,
    /// To halfway down the upper arm.
    Short,
    Elbow,
    Wrist,
    /// Wide and gathered at the wrist (late medieval on).
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Neck {
    /// A keyhole slit at the front.
    Slit,
    Round,
    /// Chemises only (late medieval on).
    Square,
    /// Gathered on a drawstring or narrow band (late medieval on).
    Gathered,
    /// A small standing collar (16th century).
    Collar,
    /// Wrapped across the front and tied: juban and zhongyi.
    Crossover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cut {
    /// Rectangles with gussets under the arms.
    Gussets,
    /// Widened with gores at the sides: long ones only.
    Gored,
    /// The body gathered full at the neck (16th century).
    Full,
}

/// Stays worn over the shirt, as one item with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stays {
    None,
    /// Two parts laced front and back, stiffened with reeds or buckram, a busk (1500s).
    PairOfBodies,
    /// Whalebone, laced behind, tabs at the waist (1600s on).
    Stays,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shirt {
    pub length: Length,
    pub sleeves: Sleeves,
    pub neck: Neck,
    pub cut: Cut,
    /// Made for sleeping in, though it may be worn at any time (owner, 2026-11-09).
    pub night: bool,
    pub stays: Stays,
}

const LENGTHS: [(Length, u64); 5] = [
    (Length::Hip, 20),
    (Length::Thigh, 25),
    (Length::Knee, 15),
    (Length::Calf, 20),
    (Length::Ankle, 20),
];

const SLEEVES: [(Sleeves, u64); 5] = [
    (Sleeves::None, 8),
    (Sleeves::Short, 10),
    (Sleeves::Elbow, 12),
    (Sleeves::Wrist, 60),
    (Sleeves::Full, 6),
];

fn necks(length: Length, sleeves: Sleeves) -> [(Neck, u64); 6] {
    let wraps = match length {
        Length::Hip => true,
        Length::Ankle => sleeves == Sleeves::Wrist,
        _ => false,
    };
    let long_sleeves = sleeves >= Sleeves::Wrist;
    [
        (Neck::Slit, 30),
        (Neck::Round, 25),
        (Neck::Square, when(length.long(), 8)),
        (Neck::Gathered, 12),
        (Neck::Collar, when(long_sleeves, 4)),
        (Neck::Crossover, when(wraps, 10)),
    ]
}

fn cuts(length: Length, sleeves: Sleeves, neck: Neck) -> [(Cut, u64); 3] {
    let gathered = matches!(neck, Neck::Gathered | Neck::Collar);
    [
        (Cut::Gussets, 60),
        (
            Cut::Gored,
            when(length.long() && neck != Neck::Crossover, 25),
        ),
        (Cut::Full, when(gathered && sleeves >= Sleeves::Wrist, 15)),
    ]
}

impl Shirt {
    fn may_be_for_night(&self) -> bool {
        self.length.long()
            && self.sleeves == Sleeves::Wrist
            && matches!(self.neck, Neck::Slit | Neck::Round | Neck::Gathered)
            && self.cut != Cut::Full
    }

    fn stays_allowed(&self) -> [(Stays, u64); 3] {
        let may = self.length.long()
            && !self.night
            && self.sleeves != Sleeves::None
            && matches!(self.neck, Neck::Round | Neck::Square | Neck::Gathered);
        [
            (Stays::None, 90),
            (Stays::PairOfBodies, when(may, 5)),
            (Stays::Stays, when(may, 5)),
        ]
    }

    pub(super) fn roll(rng: &mut Rng) -> Self {
        let length = rng.pick_weighted(&LENGTHS);
        let sleeves = rng.pick_weighted(&SLEEVES);
        let neck = rng.pick_weighted(&necks(length, sleeves));
        let cut = rng.pick_weighted(&cuts(length, sleeves, neck));
        let mut shirt = Self {
            length,
            sleeves,
            neck,
            cut,
            night: false,
            stays: Stays::None,
        };
        shirt.night = shirt.may_be_for_night() && rng.range_u64(0, 6) == 0;
        shirt.stays = rng.pick_weighted(&shirt.stays_allowed());
        shirt
    }

    pub(super) fn allowed(&self) -> bool {
        has(&LENGTHS, self.length)
            && has(&SLEEVES, self.sleeves)
            && has(&necks(self.length, self.sleeves), self.neck)
            && has(&cuts(self.length, self.sleeves, self.neck), self.cut)
            && (!self.night || self.may_be_for_night())
            && has(&self.stays_allowed(), self.stays)
    }

    /// Haircloth can be worn as a hair shirt: plain and short, never for the night.
    pub(super) fn may_be_haircloth(&self) -> bool {
        self.length <= Length::Knee
            && self.stays == Stays::None
            && matches!(self.neck, Neck::Slit | Neck::Round)
            && self.cut == Cut::Gussets
            && matches!(
                self.sleeves,
                Sleeves::None | Sleeves::Short | Sleeves::Wrist
            )
    }

    /// Its name from the research's signatures; "Shirt" or "Chemise" when none fits.
    pub(super) fn name(&self) -> &'static str {
        let (length, sleeves, neck) = (self.length, self.sleeves, self.neck);
        match neck {
            Neck::Crossover if length == Length::Ankle => return "Nagajuban",
            Neck::Crossover if sleeves == Sleeves::Wrist => return "Zhongyi",
            Neck::Crossover => return "Hadajuban",
            _ => {}
        }
        if self.night {
            return match neck {
                Neck::Gathered => "Night Chemise",
                _ => "Nightshirt",
            };
        }
        match length {
            Length::Ankle if neck == Neck::Slit && self.cut == Cut::Gored => "Undertunic",
            _ if length.long() && neck == Neck::Gathered && self.cut == Cut::Full => "Smock",
            _ if length.long() => "Chemise",
            Length::Knee if neck == Neck::Round && sleeves <= Sleeves::Short => "Subucula",
            Length::Knee if neck == Neck::Slit && sleeves == Sleeves::Wrist => "Særk",
            Length::Knee => "Long Shirt",
            _ => "Shirt",
        }
    }

    /// Grams in plain linen at an ordinary size, stays left out.
    pub(super) fn base_grams(&self) -> u32 {
        let body = match self.length {
            Length::Hip => 220,
            Length::Thigh => 260,
            Length::Knee => 330,
            Length::Calf => 400,
            Length::Ankle => 460,
        };
        let sleeves = match self.sleeves {
            Sleeves::Full => 60,
            _ => 0,
        };
        let less = match self.sleeves {
            Sleeves::None => 50,
            Sleeves::Short => 35,
            Sleeves::Elbow => 20,
            _ => 0,
        };
        let cut = match self.cut {
            Cut::Gussets => 0,
            Cut::Gored => 40,
            Cut::Full => 80,
        };
        body + sleeves + cut - less
    }

    /// Grams the stays add.
    pub(super) fn stays_grams(&self) -> u32 {
        match self.stays {
            Stays::None => 0,
            Stays::PairOfBodies => 350,
            Stays::Stays => 450,
        }
    }
}
