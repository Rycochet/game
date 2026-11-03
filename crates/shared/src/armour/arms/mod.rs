//! Armour for the arms: bracers, vambraces, couters, rerebraces, whole arm harness,
//! manicae, kote, bazubands and armoured sleeves. One item covers both arms (Arms has
//! one place); bracers and manicae may cover one. See `docs/context/shared/armour.md`.

mod grid;
mod names;
mod parts;
mod roll;
mod tables;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use parts::{Couter, Reach, Sides, Spikes};
pub use roll::roll;

use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// A piece of arm armour: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArmsForm {
    pub construction: Construction,
    pub style: Style,
    /// How much of the arm it covers.
    pub reach: Reach,
    /// Both arms, or only one.
    pub sides: Sides,
    /// The elbow's cup, on rigid pieces reaching the elbow.
    pub couter: Couter,
    /// Rigid tubes closed all round (hinged), or open at the inside and strapped.
    pub closed: bool,
    pub spikes: Spikes,
    /// Leather with rows of rivet heads.
    pub studded: bool,
    /// The plates, rings, scales, leather or padding.
    pub material: Material,
    pub finish: Finish,
    /// The colour of the cloth a piece is built on or covered with.
    pub cover: Option<Dye>,
    /// Edges of another metal.
    pub trim: Option<Material>,
}

impl ArmsForm {
    /// Japanese armoured sleeves of cloth with plates: kote, or han kote on the
    /// forearm only.
    pub fn kote(&self) -> bool {
        self.style == Style::Japanese
            && matches!(
                self.construction,
                Construction::MailAndPlate | Construction::PlatesInCloth
            )
    }

    /// Whether it takes the Middle layer as well: whole kote have their own padded
    /// sleeve (the owner's decision).
    pub fn padded(&self) -> bool {
        self.kote() && self.reach == Reach::Whole
    }

    /// Holds its shape: plate and hardened leather tubes, and splints.
    pub fn rigid(&self) -> bool {
        match self.construction {
            Construction::Plate | Construction::Splint => true,
            Construction::Leather => self.material == Material::HardenedLeather,
            _ => false,
        }
    }

    /// Indo-Persian plate forearm guards rising to a point over the elbow.
    pub fn bazubands(&self) -> bool {
        self.style == Style::IndoPersian && self.construction == Construction::Plate
    }

    /// Laminar hoops in the Classical way: the Roman and Parthian manica.
    pub fn manica(&self) -> bool {
        self.style == Style::Classical && self.construction == Construction::Laminar
    }
}
