//! Footwear worn in the Middle layer on the feet: shoes, boots up to the knee (and the
//! cavalier boot over it), sandals, clogs and slippers. What a pair is like is rolled
//! once and saved; each choice comes from what the ones before allow, so nothing is
//! thrown away. Foot armour, worn over it, is `armour::feet`. See
//! `docs/context/shared/footwear.md`.

mod details;
mod finish;
mod grid;
mod holding;
mod materials;
mod names;
mod parts;
mod roll;
mod tables;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use details::{Decoration, Fastening, Top, Under};
pub use parts::{BROAD, Build, FootwearPiece, Heel, Height, LONG_POINT, Toe};
pub use roll::roll;

use crate::armour::Dye;
use crate::material::Material;

/// A pair of shoes, boots, sandals, clogs or slippers: everything about them that is
/// rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FootwearForm {
    pub piece: FootwearPiece,
    pub build: Build,
    pub height: Height,
    /// How high the top is above the ground, in hundredths of the foot's length.
    pub shaft: u16,
    pub toe: Toe,
    /// How far a long point reaches past the toes, in hundredths of the foot's length;
    /// 0 for other toes.
    pub point: u8,
    /// Hundredths of the foot's width at its widest: over 100 only for broad toes.
    pub width: u8,
    pub heel: Heel,
    /// Millimetres under the heel at an ordinary size.
    pub heel_mm: u8,
    /// No back, the heel bare: mules and babouches.
    pub backless: bool,
    pub fastening: Fastening,
    pub top: Top,
    pub decoration: Decoration,
    pub under: Under,
    /// The upper, or the straps of sandals and wooden soles.
    pub upper: Material,
    pub sole: Material,
    pub dye: Dye,
    /// A second colour: a top boot's cuff, slashing's lining, a saddle shoe's saddle.
    pub contrast: Option<Dye>,
}

impl FootwearForm {
    /// Over the ankle.
    pub fn boot(&self) -> bool {
        self.height > Height::Low
    }

    /// A sole with straps rather than an upper over the foot.
    pub fn open(&self) -> bool {
        self.toe == Toe::Open
    }
}
