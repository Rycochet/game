//! Armour for the torso: breastplates and cuirasses, leather, mail, ring mail and
//! padding, and the arming vest worn under them. Backplates are part of it, so armour
//! never stops a pack on the back. See `docs/context/shared/armour.md`.

mod grid;
mod names;
mod parts;
mod roll;
mod tables;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use parts::{Breast, Fluting, Links, Plackart, Quilting, Rings, TorsoBody, Weave};
pub use roll::roll;

use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// Which item a torso form is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TorsoKind {
    /// Outer armour (item kind `TorsoArmour`).
    Armour,
    /// A sleeveless padded vest worn under it, on the Middle layer (`ArmingVest`).
    ArmingVest,
}

/// One piece of torso armour: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TorsoForm {
    pub body: TorsoBody,
    pub construction: Construction,
    pub style: Style,
    /// The plates, rings, leather or padding.
    pub material: Material,
    /// A backplate as well as a breast; without, straps cross behind.
    pub back: bool,
    /// Worn over a mail shirt that is part of it, showing at the arm holes and hem.
    pub over_mail: bool,
    /// With its own padded lining, so it takes the Middle layer too.
    pub lined: bool,
    /// An arming vest, worn under armour.
    pub vest: bool,
    pub finish: Finish,
    /// The colour of cloth over or of padding.
    pub cover: Option<Dye>,
    /// The colour of lacing.
    pub lacing: Option<Dye>,
    /// Edges of another metal.
    pub trim: Option<Material>,
}

impl TorsoForm {
    pub fn kind(&self) -> TorsoKind {
        if self.vest {
            TorsoKind::ArmingVest
        } else {
            TorsoKind::Armour
        }
    }

    /// Whether it takes the Middle layer as well as its own: lined, or padding itself.
    pub fn padded(&self) -> bool {
        self.lined || (self.construction == Construction::Padded && !self.vest)
    }

    /// The breast's shape, for shaped plate and leather.
    pub fn breast(&self) -> Option<Breast> {
        match self.body {
            TorsoBody::Plate { breast, .. } | TorsoBody::Leather { breast, .. } => Some(breast),
            _ => None,
        }
    }

    /// Short sleeves, on mail, ring mail and padding.
    pub fn sleeves(&self) -> bool {
        match self.body {
            TorsoBody::Mail { sleeves, .. }
            | TorsoBody::RingMail { sleeves }
            | TorsoBody::Padded { sleeves, .. } => sleeves,
            _ => false,
        }
    }

    /// Holds its own shape rather than hanging: plate and hardened leather.
    pub fn rigid(&self) -> bool {
        match self.body {
            TorsoBody::Plate { .. } => true,
            TorsoBody::Leather { .. } => self.material != Material::BuffLeather,
            _ => false,
        }
    }
}
