//! Armour for the waist: faulds, culets, tassets, tonlets and steel bases, mail skirts and
//! breeches, pteruges, kusazuri and other hip panels, lamellar and scale skirts, war
//! girdles and the mitra. Everything hung from the waist is Waist however far down it
//! reaches; only cuirassier tassets also take Legs. Waist never takes Middle.
//! See `docs/context/shared/armour.md`.

mod counts;
mod extras;
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

pub use parts::{Length, Opening, WaistKind, WaistPiece};
pub use roll::roll;

use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// A piece of waist armour: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WaistForm {
    /// Waist armour, or cuirassier tassets: the item kind it was made for.
    pub kind: WaistKind,
    pub construction: Construction,
    pub style: Style,
    pub piece: WaistPiece,
    pub length: Length,
    /// Hoops of a fauld or plate skirt, lames of each tasset (1 is a single plate), rows
    /// of a panel, scale or lamellar skirt, or of pteruges; 0 where it has none.
    pub lames: u8,
    /// Hoops of a culet behind a fauld or tassets; 0 for none.
    pub culet: u8,
    /// Tassets (2, or 4 with small ones at the sides), or panels round the hips.
    pub hangs: u8,
    /// Pteruges over the front only, as the Roman belt's apron; a girdle with a plate
    /// over the belly (the mitra).
    pub front: bool,
    pub opening: Opening,
    /// A mail skirt under a plate fauld or tassets, made as one item (the owner's
    /// decision: mail skirts are also items of their own).
    pub mail_beneath: bool,
    /// A plain plate over the groin (the owner's decision: never a codpiece's shape).
    pub groin: bool,
    /// Leather with rows of rivet heads.
    pub studded: bool,
    /// The plates, rings, scales, leather or padding.
    pub material: Material,
    pub finish: Finish,
    /// The colour of the cloth a piece is built on or covered with, or of its lacing.
    pub cloth: Option<Dye>,
    /// Edges of another metal.
    pub trim: Option<Material>,
}

impl WaistForm {
    /// Japanese pieces: kusazuri.
    pub fn japanese(&self) -> bool {
        self.style == Style::Japanese
    }

    /// Holds its shape: plate, splints, laminar lames and hardened leather.
    pub fn rigid(&self) -> bool {
        match self.construction {
            Construction::Plate | Construction::Splint | Construction::Laminar => true,
            Construction::Leather => self.material == Material::HardenedLeather,
            _ => false,
        }
    }

    /// Hangs in rows of small plates or scales.
    pub fn rows(&self) -> bool {
        matches!(
            self.construction,
            Construction::Scale | Construction::Lamellar | Construction::Laminar
        )
    }
}
