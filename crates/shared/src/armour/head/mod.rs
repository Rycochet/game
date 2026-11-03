//! Armour for the head: helmets of every kind, mail and padded coifs, and the arming
//! cap worn under them. Every helmet is rolled from one of the named rows in `table`,
//! then named by what it turned out like. See `docs/context/shared/armour.md`.

mod grid;
mod materials;
mod names;
mod parts;
mod rim;
mod roll;
mod table;
mod weight;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

pub use parts::{Back, Bowl, Brim, Face, Ornament, Skull, Top, Visor};
pub use rim::{Edge, OFF_HEAD};
pub use roll::roll;

use super::{Construction, Dye, Finish, Style};
use crate::material::Material;

/// Which item a head form is for: each is worn differently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeadKind {
    /// A padded cap on the Skin layer (item kind `ArmingCap`).
    Cap,
    /// A hood of mail or quilting down to the shoulders, on Middle, over Head and Neck
    /// (`Coif`).
    Coif,
    /// Outer, on the Head (`Helmet`).
    Helmet,
    /// A helm whose own plates guard the throat, Outer on Head and Neck (`BevoredHelm`).
    BevoredHelm,
}

/// One head piece: everything about it that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeadForm {
    pub kind: HeadKind,
    pub construction: Construction,
    pub style: Style,
    /// The bowl, rings, leather, rattan or padding.
    pub material: Material,
    pub bowl: Bowl,
    pub skull: Skull,
    pub face: Face,
    /// Cheek pieces.
    pub cheeks: bool,
    pub back: Back,
    pub brim: Brim,
    pub top: Top,
    pub ornament: Ornament,
    /// Rows of rivet heads, on leather.
    pub studded: bool,
    pub finish: Finish,
    /// The colour of cloth: a cap's or coif's, or a crest's or plume's.
    pub cover: Option<Dye>,
    /// The colour of lacing: lamellar, and a kabuto's lames.
    pub lacing: Option<Dye>,
}

impl HeadForm {
    /// A helmet of either kind, rather than a cap or coif.
    pub fn helmet(&self) -> bool {
        matches!(self.kind, HeadKind::Helmet | HeadKind::BevoredHelm)
    }

    /// Plates of a boar's tusks over leather: a conical leather helmet.
    pub fn tusks(&self) -> bool {
        self.construction == Construction::Leather && self.skull == Skull::Conical
    }

    /// Laced, so it has a lacing colour: lamellar, and Japanese lames.
    pub fn laced(&self) -> bool {
        self.construction == Construction::Lamellar
            || (self.style == Style::Japanese && self.back == Back::Lames)
    }

    /// Shaped like a skull cap: open, small, with nothing added round it.
    pub fn skull_cap(&self) -> bool {
        self.face == Face::Open
            && !self.cheeks
            && self.back == Back::None
            && self.brim == Brim::None
            && matches!(self.skull, Skull::Round | Skull::Low)
    }
}
