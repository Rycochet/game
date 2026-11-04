//! The footwear's rows of the table of kinds (`def.rs`): how each is worn. Their
//! names, cells and weights are their forms' (`footwear`); these are for items made
//! before forms.

use super::item_def::{plain, wear};
use super::{ItemDef, ItemKind, Shape};
use crate::creature::{Layer, Location};

impl ItemKind {
    pub(super) const fn feet_def(self) -> ItemDef {
        // All Middle, so foot armour goes over them.
        let on_feet = wear(Layer::Middle, &[Location::Feet], (1, 0), false);
        let low = Shape::rectangle(2, 1);
        match self {
            Self::Boots => plain("Boots", 1100, Shape::rectangle(2, 2)).worn(on_feet),
            Self::Clogs => plain("Clogs", 900, low).worn(on_feet),
            Self::Sandals => plain("Sandals", 300, low).worn(on_feet),
            Self::Shoes => plain("Shoes", 500, low).worn(on_feet),
            Self::Slippers => plain("Slippers", 350, low).worn(on_feet),
            _ => panic!("not footwear"),
        }
    }
}
