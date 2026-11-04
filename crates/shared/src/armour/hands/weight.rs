//! What a piece for the hands weighs.

use super::{Cuff, Fingers, HandsForm, Knuckles};
use crate::armour::Construction;
use crate::creature::Size;

/// The material its main weight is worked out against, in tenths of a gram per cm³.
pub(super) const STEEL_DENSITY: u32 = 78;
pub(super) const LEATHER_DENSITY: u32 = 9;
pub(super) const LINEN_DENSITY: u32 = 15;

impl HandsForm {
    /// What the pair weighs at an ordinary size, in grams: its main material's share
    /// by that material's density, then backing, straps and extras. Worked out from
    /// area × thickness × density (see the research in `docs/context/research-armour-hands.md`).
    pub fn grams(&self) -> u32 {
        use Construction as C;
        // Grams at steel's density, over the hand and over the cuff.
        let hand = match self.fingers {
            Fingers::Fingered => 320,
            Fingers::Mitten => 280,
            Fingers::Open => 150,
        };
        let cuff = match self.cuff {
            Cuff::Wrist => 120,
            Cuff::Hourglass => 360,
            Cuff::Bell => 380,
            Cuff::Pointed => 430,
            Cuff::Long => 650,
        };
        let cuff = if self.fingers == Fingers::Open {
            cuff * 3 / 5
        } else {
            cuff
        };
        // Main weight at its reference density, the reference, and everything else.
        let (main, reference, rest) = match self.construction {
            C::Plate => {
                let backing = if self.fingers == Fingers::Open {
                    70
                } else {
                    180
                };
                (hand + cuff, STEEL_DENSITY, backing)
            }
            C::PlatesInCloth => ((hand + cuff) * 9 / 10, STEEL_DENSITY, 260),
            C::Mail => (hand * 2 + cuff - 80, STEEL_DENSITY, 60),
            // Flat rings over the back and fingers only, sewn on a leather glove.
            C::RingMail => ((hand + cuff) * 3 / 4, STEEL_DENSITY, 120),
            C::Scale => ((hand + cuff) * 9 / 10, STEEL_DENSITY, 200),
            C::MailAndPlate if self.fingers == Fingers::Open => (150, STEEL_DENSITY, 40),
            C::MailAndPlate => (hand * 3 / 5 + 400 + cuff, STEEL_DENSITY, 60),
            C::Leather => (200 + cuff * 3 / 7, LEATHER_DENSITY, 0),
            _ => (150, LINEN_DENSITY, 0),
        };
        let extras = match self.knuckles {
            Knuckles::Plain | Knuckles::Ridged => 0,
            Knuckles::Spiked => 40,
            Knuckles::Rondel => 60,
        } + if self.claws { 80 } else { 0 }
            + if self.locking { 100 } else { 0 };
        main * self.material.density() / reference + rest + extras
    }

    /// What the pair weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
