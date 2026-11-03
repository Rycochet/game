//! Armour for the shoulders, as a pair: pauldrons, spaulders, hanging shoulder guards
//! (sode and their eastern cousins), mail and scale doublings and the linothorax yoke,
//! shoulder discs, ailettes and the Chukchi wing. What a pair is like is rolled once
//! and saved; how big a shoulder is comes from the body frame, so the same pair is drawn
//! alone and worn. See `docs/context/shared/armour.md` and the research in
//! `docs/context/research-armour-shoulders.md`.

use serde::{Deserialize, Serialize};

use super::{Construction, Style};
use crate::material::Material;

mod adornment;
mod grid;
mod heraldry;
mod names;
mod parts;
mod piece;
mod roll;
mod tables;
mod weight;

pub use adornment::{Adornment, Finish};
pub use heraldry::{Charge, Heraldry, Tincture};
pub use parts::{AiletteShape, CapeCut, Cut, GuardShape, HautePiece, Lacing, Length, Span, Wings};
pub use piece::{ShouldersKind, ShouldersPiece};
pub use roll::roll;

// What the files here share, as `super::...`.
use tables::{
    adornments, cloths, constructions_in, finishes, kinds_for, laced, materials, may_have_trim,
    needs_cloth, pauldron_cut,
};

/// A pair of shoulder pieces: everything about them that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShouldersForm {
    pub piece: ShouldersPiece,
    pub construction: Construction,
    pub adornment: Adornment,
    pub finish: Finish,
    /// What it is mostly made of: the plates, rings, scales, leather, padding or board.
    pub material: Material,
    /// The cloth over plates in cloth, or what lamellae are laced with.
    pub cloth: Option<Material>,
    /// Edges of another metal.
    pub trim: Option<Material>,
    pub style: Style,
}

impl ShouldersForm {
    pub fn kind(&self) -> ShouldersKind {
        match self.piece {
            ShouldersPiece::Pauldrons { .. } => ShouldersKind::Pauldrons,
            ShouldersPiece::Spaulders { .. } => ShouldersKind::Spaulders,
            ShouldersPiece::Guards { .. } => ShouldersKind::Guards,
            ShouldersPiece::Cape { .. } => ShouldersKind::Cape,
            ShouldersPiece::Discs { .. } => ShouldersKind::Discs,
            ShouldersPiece::Ailettes { .. } => ShouldersKind::Ailettes,
            ShouldersPiece::Wing => ShouldersKind::Wing,
        }
    }

    /// Roman laminar spaulders.
    pub fn roman(&self) -> bool {
        self.kind() == ShouldersKind::Spaulders
            && self.construction == Construction::Laminar
            && self.style == Style::Classical
    }

    /// Mycenaean bronze spaulders, as at Dendra: one big plate each.
    pub fn dendra(&self) -> bool {
        self.kind() == ShouldersKind::Spaulders
            && self.construction == Construction::Plate
            && self.style == Style::Classical
    }

    /// Japanese shoulder guards, sode: with a solid top plate.
    pub fn japanese(&self) -> bool {
        self.kind() == ShouldersKind::Guards && self.style == Style::Japanese
    }
}

#[cfg(test)]
mod tests;
