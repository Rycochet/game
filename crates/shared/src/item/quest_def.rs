//! Quests' own kinds' rows of the table of kinds (`def.rs`): a keepsake found
//! for a quest, a delivery's parcel and a part of a creature brought back
//! for a "Collect" step (owner). Worth nothing beyond their
//! quest (`Item::quest`).

use super::item_def::plain;
use super::{ItemDef, ItemKind, Shape};

impl ItemKind {
    pub(super) const fn quest_def(self) -> ItemDef {
        match self {
            Self::Parcel => plain("Parcel", 1000, Shape::rectangle(2, 2)).without_quality(),
            // Not counted: each is named for its creature and part, which a
            // counted stack couldn't tell apart.
            Self::Remains => plain("Remains", 50, Shape::rectangle(1, 1)).without_quality(),
            _ => plain("Keepsake", 200, Shape::rectangle(1, 1)).without_quality(),
        }
    }
}
