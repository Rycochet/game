//! Armour for the feet, worn over shoes and boots: sabatons in every toe shape
//! (armoured boots when they rise over the ankle), mail shoes and kusari tabi, foot
//! guards and kogake, and the Greek bronze feet. One item covers both feet.
//! See `docs/context/shared/armour.md`.

mod grid;
mod materials;
mod names;
mod parts;
mod roll;
mod tables;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use parts::{DETACHABLE_POINT_MM, FeetPiece, POINT_MM, Toe};
pub use roll::roll;

use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// A pair of foot pieces: everything about them that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeetForm {
    pub piece: FeetPiece,
    pub construction: Construction,
    pub style: Style,
    /// Sabatons' own; mail shoes and foot guards follow the foot (`Toe::Round`).
    pub toe: Toe,
    /// Millimetres a pointed toe reaches past the toes; 0 for other toes.
    pub point: u16,
    /// The point comes off, by a catch over the big toe, for fighting on foot.
    pub detachable: bool,
    /// Hundredths of the foot's width at its widest: over 100 only for broad toes.
    pub width: u8,
    /// Lames across the top of the foot, between the toe cap and the ankle plate; 0
    /// for pieces without.
    pub lames: u8,
    /// A hinged cap round the back of the heel.
    pub heel: bool,
    /// Rising over the ankle, under the greave: armoured boots.
    pub high: bool,
    /// Ridges along each lame: Maximilian plate.
    pub fluted: bool,
    /// Foot guards: over the toes as well as the instep.
    pub toes: bool,
    /// Mail shoes: a leather sole rather than a strap under the shoe.
    pub soled: bool,
    /// Leather with rows of rivet heads.
    pub studded: bool,
    pub material: Material,
    pub finish: Finish,
    /// The colour of the cloth kogake and kusari tabi are built on.
    pub cloth: Option<Dye>,
    /// Edges of another metal.
    pub trim: Option<Material>,
}

impl FeetForm {
    /// Japanese pieces on cloth with split toes: kogake and kusari tabi.
    pub fn japanese(&self) -> bool {
        self.style == Style::Japanese
    }

    /// Bronze shaped like a bare foot, toes and nails and all.
    pub fn bare_toes(&self) -> bool {
        self.style == Style::Classical && self.piece == FeetPiece::FootGuards && self.toes
    }

    /// Holds its shape: plate, hardened leather and splints.
    pub fn rigid(&self) -> bool {
        match self.construction {
            Construction::Plate | Construction::Splint => true,
            Construction::Leather => self.material == Material::HardenedLeather,
            _ => false,
        }
    }
}
