//! Armour sets: one item made of several locations' pieces, all of one style,
//! material, quality and size (the owner's plan). Each piece is a location's own form,
//! so a set's model is its pieces' models, built in place on the body frame.

mod names;
mod roll;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use super::arms::ArmsForm;
use super::feet::FeetForm;
use super::hands::HandsForm;
use super::head::HeadForm;
use super::legs::LegsForm;
use super::neck::NeckForm;
use super::shoulders::ShouldersForm;
use super::torso::TorsoForm;
use super::waist::WaistForm;
use super::{Construction, Style};
use crate::creature::Location;
use crate::material::Material;

pub use roll::roll;

pub(crate) use roll::{roll_around, torso_fits, with_back};

/// Which locations a set covers (item kinds `ArmourCoat`, `Corslet`, `HalfArmour`,
/// `FullArmour`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SetKind {
    /// Torso, arms and a skirt, all of one construction: a hauberk, a lamellar coat.
    Coat,
    /// Gorget, breast and back, and tassets.
    Corslet,
    /// Everything but the legs and feet.
    Half,
    /// All nine locations: a harness.
    Full,
}

impl SetKind {
    pub const ALL: [Self; 4] = [Self::Coat, Self::Corslet, Self::Half, Self::Full];

    /// The locations its pieces cover, the same as its item kind's.
    pub fn locations(self) -> &'static [Location] {
        use Location::{Arms, Feet, Hands, Head, Legs, Neck, Shoulders, Torso, Waist};
        match self {
            Self::Coat => &[Arms, Torso, Waist],
            Self::Corslet => &[Neck, Torso, Waist],
            Self::Half => &[Head, Neck, Shoulders, Arms, Torso, Waist, Hands],
            Self::Full => &[Head, Neck, Shoulders, Arms, Torso, Waist, Hands, Legs, Feet],
        }
    }
}

/// A set: its torso piece, which decides the rest, and a piece for each other
/// location it covers. Pieces it doesn't cover are `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetForm {
    pub kind: SetKind,
    pub torso: TorsoForm,
    pub head: Option<HeadForm>,
    pub neck: Option<NeckForm>,
    pub shoulders: Option<ShouldersForm>,
    pub arms: Option<ArmsForm>,
    pub waist: Option<WaistForm>,
    pub hands: Option<HandsForm>,
    pub legs: Option<LegsForm>,
    pub feet: Option<FeetForm>,
}

impl SetForm {
    /// The style every piece shares.
    pub fn style(&self) -> Style {
        self.torso.style
    }

    /// The construction it is known by: its torso's.
    pub fn construction(&self) -> Construction {
        self.torso.construction
    }

    /// The material every piece is made of, where it can be (padding and leather
    /// straps keep their own).
    pub fn material(&self) -> Material {
        self.torso.material
    }

    /// Its torso piece has its own padding as well, which also takes the Middle layer
    /// (as a lined coat does).
    pub fn padded(&self) -> bool {
        self.torso.padded()
    }
}
