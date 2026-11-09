//! `UnderPiece`: the kinds of item that are underclothes.

use serde::{Deserialize, Serialize};

use crate::item::ItemKind;

/// A kind of underclothing, one for each item kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnderPiece {
    Shirt,
    Braies,
    BreastBand,
    Loincloth,
    Hose,
    Socks,
    Petticoat,
    /// A linen coif, undercap, hairnet or nightcap (not the mail coif).
    Coif,
    Wimple,
}

impl UnderPiece {
    pub const ALL: [Self; 9] = [
        Self::Shirt,
        Self::Braies,
        Self::BreastBand,
        Self::Loincloth,
        Self::Hose,
        Self::Socks,
        Self::Petticoat,
        Self::Coif,
        Self::Wimple,
    ];

    /// The piece an item kind is, or `None` for other kinds.
    pub fn of(kind: ItemKind) -> Option<Self> {
        ItemKind::ALL_UNDER
            .iter()
            .position(|&under| under == kind)
            .map(|index| Self::ALL[index])
    }

    /// The item kind this is.
    pub fn kind(self) -> ItemKind {
        ItemKind::ALL_UNDER[self as usize]
    }
}
