//! Armour for the neck: padded collars, mail standards and Japanese plated collars
//! (Middle), gorgets and colletins, nodowa, bevors, aventails hung from a helmet, and
//! the bishop's mantle over the neck and shoulders. See `docs/context/shared/armour.md`.

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

pub use parts::{Edging, Front, Hem, NeckKind, Plates};
pub use roll::roll;

use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// A neck piece: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeckForm {
    pub kind: NeckKind,
    pub construction: Construction,
    pub style: Style,
    /// Gorgets' own; `Plates::Collar` for every other kind.
    pub plates: Plates,
    /// A gorget's tall collar of lames up under the chin, rather than a low rim; a
    /// bevor reaching up to the mouth, rather than the chin.
    pub high: bool,
    /// A gorget's collar lames (or the nodowa's rows), or the lames under a bevor's
    /// chin plate; 0 for the rest.
    pub lames: u8,
    /// A bevor's front drawn to a point, with a ridge down it.
    pub pointed: bool,
    /// Aventails' own; `Front::Open` for every other kind.
    pub front: Front,
    /// Reaching out over the shoulders: aventails and mail standards that may, and
    /// every mantle.
    pub cape: bool,
    pub hem: Hem,
    pub edging: Edging,
    /// A ring of spikes round a gorget's collar (fantasy).
    pub spikes: bool,
    /// Rows of rivet heads: leather gorgets and quilted aventails.
    pub studded: bool,
    pub material: Material,
    pub finish: Finish,
    /// The colour of a padded piece's cover, of the cloth plates are set in, or of
    /// Japanese lacing.
    pub cloth: Option<Dye>,
    /// Edges of another metal on a plate piece.
    pub trim: Option<Material>,
}

impl NeckForm {
    /// Japanese: lacquered lames and laces, plates on cloth.
    pub fn japanese(&self) -> bool {
        self.style == Style::Japanese
    }

    /// Holds its shape: plate, laminar and hardened leather.
    pub fn rigid(&self) -> bool {
        match self.construction {
            Construction::Plate | Construction::Laminar => true,
            Construction::Leather => self.material == Material::HardenedLeather,
            _ => false,
        }
    }

    /// A plate gorget with plates over the chest and back, worn over the breastplate
    /// rather than under it: a low collar, as in the 17th century. A tall collar of
    /// lames went under it, as in the 16th.
    pub fn over_breastplate(&self) -> bool {
        self.kind == NeckKind::Gorget
            && self.construction == Construction::Plate
            && matches!(self.plates, Plates::Plates | Plates::Shoulders)
            && !self.high
    }
}
