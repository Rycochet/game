//! `Garment`: which piece of underclothing something is and how that one is cut,
//! passing each question on to its own file.

use serde::{Deserialize, Serialize};

use super::{Band, Braies, Coif, Hose, Loincloth, Petticoat, Shirt, Socks, UnderPiece, Wimple};
use crate::creature::Location;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Garment {
    Shirt(Shirt),
    Braies(Braies),
    Band(Band),
    Loincloth(Loincloth),
    Hose(Hose),
    Socks(Socks),
    Petticoat(Petticoat),
    Coif(Coif),
    Wimple(Wimple),
}

impl Garment {
    pub fn piece(&self) -> UnderPiece {
        match self {
            Self::Shirt(_) => UnderPiece::Shirt,
            Self::Braies(_) => UnderPiece::Braies,
            Self::Band(_) => UnderPiece::BreastBand,
            Self::Loincloth(_) => UnderPiece::Loincloth,
            Self::Hose(_) => UnderPiece::Hose,
            Self::Socks(_) => UnderPiece::Socks,
            Self::Petticoat(_) => UnderPiece::Petticoat,
            Self::Coif(_) => UnderPiece::Coif,
            Self::Wimple(_) => UnderPiece::Wimple,
        }
    }

    pub(super) fn roll(rng: &mut Rng, piece: UnderPiece) -> Self {
        match piece {
            UnderPiece::Shirt => Self::Shirt(Shirt::roll(rng)),
            UnderPiece::Braies => Self::Braies(Braies::roll(rng)),
            UnderPiece::BreastBand => Self::Band(Band::roll(rng)),
            UnderPiece::Loincloth => Self::Loincloth(Loincloth::roll(rng)),
            UnderPiece::Hose => Self::Hose(Hose::roll(rng)),
            UnderPiece::Socks => Self::Socks(Socks::roll(rng)),
            UnderPiece::Petticoat => Self::Petticoat(Petticoat::roll(rng)),
            UnderPiece::Coif => Self::Coif(Coif::roll(rng)),
            UnderPiece::Wimple => Self::Wimple(Wimple::roll(rng)),
        }
    }

    pub(super) fn allowed(&self) -> bool {
        match self {
            Self::Shirt(shirt) => shirt.allowed(),
            Self::Braies(braies) => braies.allowed(),
            Self::Band(band) => band.allowed(),
            Self::Loincloth(loincloth) => loincloth.allowed(),
            Self::Hose(hose) => hose.allowed(),
            Self::Socks(socks) => socks.allowed(),
            Self::Petticoat(petticoat) => petticoat.allowed(),
            Self::Coif(coif) => coif.allowed(),
            Self::Wimple(wimple) => wimple.allowed(),
        }
    }

    /// Locations of its kind it leaves bare: a sleeveless shirt's arms, the feet of
    /// footless and stirrup hose. Worn and clashing go by what is left.
    pub fn bare(&self) -> &'static [Location] {
        match self {
            Self::Shirt(shirt) if shirt.sleeves == super::Sleeves::None => &[Location::Arms],
            Self::Hose(hose) if !hose.feet.covers_feet() => &[Location::Feet],
            _ => &[],
        }
    }
}
