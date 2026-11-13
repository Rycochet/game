//! Quests' own kinds' rows of the table of kinds (`def.rs`): a keepsake found
//! for a quest and a delivery's parcel (owner). Worth nothing beyond their
//! quest (`Item::quest`).

use super::item_def::plain;
use super::{ItemDef, ItemKind, Shape};

impl ItemKind {
    pub(super) const fn quest_def(self) -> ItemDef {
        match self {
            Self::Parcel => plain("Parcel", 1000, Shape::rectangle(2, 2)).without_quality(),
            _ => plain("Keepsake", 200, Shape::rectangle(1, 1)).without_quality(),
        }
    }
}
