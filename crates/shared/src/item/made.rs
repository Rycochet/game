//! `Made`: who an item was made for.

use serde::{Deserialize, Serialize};

use crate::creature::{Location, Size};
use crate::item::ItemKind;

/// Who an item was made for. Part of the item itself: one kind of glove comes in
/// any size and number of fingers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Made {
    /// The size it fits.
    pub size: Size,
    /// The smallest and largest it can be worn by at all.
    pub smallest: Size,
    pub largest: Size,
    /// How many fingers it has, if it covers them.
    pub fingers: Option<u8>,
    /// It resizes itself to fit whatever their fingers, and sizes as far from its own
    /// as its quality reaches (`Quality::magic_reach`), but not whatever their shape.
    pub magical: bool,
}

impl Made {
    /// A thing of this kind made for a creature of this size and these fingers.
    pub fn for_body(kind: ItemKind, size: Size, fingers: Option<u8>, magical: bool) -> Self {
        let wear = kind.def().wear;
        let covers_fingers =
            wear.is_some_and(|wear| !wear.one_place && wear.locations.contains(&Location::Fingers));
        Self {
            size,
            smallest: size.stepped(-(wear.map_or(0, |wear| wear.down) as i8)),
            largest: size.stepped(wear.map_or(0, |wear| wear.up) as i8),
            fingers: if covers_fingers { fingers } else { None },
            magical,
        }
    }
}
