//! The neck pieces' rows of the table of kinds (`def.rs`): how each is worn. Their
//! names, cells and weights are their forms' (`armour::neck`).

use super::item_def::{plain, wear};
use super::{ItemDef, ItemKind, Shape};
use crate::creature::{Layer, Location};

impl ItemKind {
    pub(super) const fn neck_def(self) -> ItemDef {
        use Layer::{Middle, Outer};
        use Location::{Neck, Shoulders};
        let square = Shape::rectangle(2, 2);
        match self {
            // Hung from a helmet's lower edge.
            Self::Aventail => {
                plain("Aventail", 1300, square).worn(wear(Outer, &[Neck], (1, 0), false))
            }
            Self::Bevor => plain("Bevor", 1000, square).worn(wear(Outer, &[Neck], (0, 0), false)),
            // Under a gorget, bevor or aventail, or alone: padded, a mail standard, or a
            // Japanese collar lined with plates.
            Self::Collar => plain("Collar", 700, Shape::rectangle(2, 1)).worn(wear(
                Middle,
                &[Neck],
                (1, 0),
                false,
            )),
            Self::Gorget => plain("Gorget", 1100, square).worn(wear(Outer, &[Neck], (0, 0), false)),
            // A short cape of mail or scale over the neck and shoulders, so no pauldrons
            // with it (the owner's decision).
            Self::Mantle => {
                plain("Mantle", 2000, square).worn(wear(Outer, &[Neck, Shoulders], (1, 0), false))
            }
            _ => panic!("not a neck piece"),
        }
    }
}
