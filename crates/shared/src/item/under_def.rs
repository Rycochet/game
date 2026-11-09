//! The underclothes' rows of the table of kinds (`def.rs`): how each is worn, all on
//! the Skin layer. Their names, cells and weights are their forms' (`underclothes`);
//! these are for items made before forms. Braies, loincloths and petticoats are on the
//! Waist alone, as they hang from it, so hose and stockings go with them (owner,
//! 2026-11-09).

use super::item_def::{plain, wear};
use super::{ItemDef, ItemKind, Shape};
use crate::creature::Layer::Skin;
use crate::creature::Location::{Arms, Feet, Head, Legs, Neck, Torso, Waist};

impl ItemKind {
    /// The underclothes with forms, in `underclothes::UnderPiece`'s order.
    pub const ALL_UNDER: [Self; 9] = [
        Self::Shirt,
        Self::Braies,
        Self::BreastBand,
        Self::Loincloth,
        Self::Hose,
        Self::Socks,
        Self::Petticoat,
        Self::LinenCoif,
        Self::Wimple,
    ];

    /// Underclothes, including the old T-shirt (no form, no longer made).
    pub(super) const fn is_under(self) -> bool {
        matches!(
            self,
            Self::Shirt
                | Self::Braies
                | Self::BreastBand
                | Self::Loincloth
                | Self::Hose
                | Self::Socks
                | Self::Petticoat
                | Self::LinenCoif
                | Self::Wimple
                | Self::TShirt
        )
    }

    pub(super) const fn under_def(self) -> ItemDef {
        let wide = Shape::rectangle(2, 1);
        let shirt = Shape::rectangle(2, 2);
        let small = Shape::rectangle(1, 1);
        match self {
            Self::Braies => plain("Braies", 220, shirt).worn(wear(Skin, &[Waist], (1, 1), true)),
            Self::BreastBand => {
                plain("Breast Band", 60, wide).worn(wear(Skin, &[Torso], (1, 1), true))
            }
            Self::Hose => plain("Hose", 250, shirt).worn(wear(Skin, &[Legs, Feet], (1, 0), false)),
            Self::LinenCoif => {
                plain("Linen Coif", 40, small).worn(wear(Skin, &[Head], (1, 0), false))
            }
            Self::Loincloth => {
                plain("Loincloth", 100, wide).worn(wear(Skin, &[Waist], (1, 1), true))
            }
            Self::Petticoat => {
                plain("Petticoat", 450, shirt).worn(wear(Skin, &[Waist], (1, 1), true))
            }
            Self::Shirt => {
                plain("Shirt", 260, shirt).worn(wear(Skin, &[Torso, Arms], (1, 0), false))
            }
            Self::Socks => plain("Socks", 50, small).worn(wear(Skin, &[Feet], (1, 1), true)),
            Self::TShirt => {
                plain("T-Shirt", 200, shirt).worn(wear(Skin, &[Torso, Arms], (1, 0), false))
            }
            Self::Wimple => plain("Wimple", 60, small).worn(wear(Skin, &[Neck], (1, 1), true)),
            _ => panic!("not underclothes"),
        }
    }
}
