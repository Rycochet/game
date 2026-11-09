//! Underclothes, all worn on the Skin layer: shirts and chemises, braies, breast bands,
//! loincloths, hose, socks, petticoats, linen coifs and wimples. Medieval and earlier
//! first. What one is like is rolled once and saved; each choice comes from what the
//! ones before allow, so nothing is thrown away. See
//! `docs/context/shared/underclothes.md` and the research in
//! `docs/context/research-underclothing.md`.

mod band;
mod braies;
mod cloth;
mod coif;
mod garment;
mod grid;
mod hose;
mod loincloth;
mod names;
mod petticoat;
mod piece;
mod roll;
mod shirt;
mod socks;
mod table;
mod weight;
mod wimple;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use band::{Band, BandForm};
pub use braies::{Braies, BraiesLength, Ends, Fit, Waistband};
pub use cloth::{Cloth, Decoration};
pub use coif::{Coif, CoifForm};
pub use garment::Garment;
pub use hose::{Colours, Held, Hose, HoseFeet, HoseHeight, HoseMake};
pub use loincloth::{LoinForm, Loincloth};
pub use petticoat::{Petticoat, SkirtFullness, SkirtLength, SkirtMake, SkirtWaist};
pub use piece::UnderPiece;
pub use roll::roll;
pub use shirt::{Cut, Length, Neck, Shirt, Sleeves, Stays};
pub use socks::{SockHeight, SockMake, SockTop, Socks, Toe};
pub use wimple::{Wimple, WimpleForm};

use crate::armour::Dye;
use crate::material::Material;

/// One piece of underclothing: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnderForm {
    /// Its shape: which garment it is and how that is cut.
    pub garment: Garment,
    pub material: Material,
    /// Fine lawn instead of plain linen, or the net of a hairnet.
    pub cloth: Cloth,
    pub decoration: Decoration,
    pub dye: Dye,
    /// A second colour: parti-coloured or striped hose.
    pub contrast: Option<Dye>,
}

impl UnderForm {
    /// Which kind of underclothing it is.
    pub fn piece(&self) -> UnderPiece {
        self.garment.piece()
    }
}
