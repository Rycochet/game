//! Wearing things: the rules both sides check clothes by. See
//! `docs/context/shared/creature.md`.
//!
//! An item has one layer and the locations it covers. It fits a body if every one
//! of them exists there and takes that layer. It can only go on or come off if
//! nothing on a higher layer is at any of its locations.

use crate::armour::hands::Fingers;
use crate::blade::Hands;
use crate::creature::{Layer, Location, Size};
use crate::id::Id;
use crate::item::{Item, Made, Quality, WearDef};
use crate::item_form::ItemForm;

mod changes;
mod check;
mod death;
mod secrets;
mod shown;
mod swap;

pub use death::kept_on_hard;
pub use shown::WornLook;
pub use swap::Swapped;

/// Whether something is made for a number of fingers, so only fits hands with that
/// many unless magical: anything over each finger, but not mittens, which take any
/// (the owner's decision), nor rings, which take one.
fn made_for_fingers(item: &Item, wear: &WearDef) -> bool {
    let mitten =
        matches!(&item.form, Some(ItemForm::Hands(hands)) if hands.fingers == Fingers::Mitten);
    !wear.one_place && wear.locations.contains(&Location::Fingers) && !mitten
}

/// Whether it takes only one place of its location (a ring, a one-handed weapon),
/// rather than all of it: a two-handed blade takes both hands, though its kind
/// says one place, as blades of one kind differ.
pub fn takes_one_place(item: &Item) -> bool {
    let two_handed =
        matches!(&item.form, Some(ItemForm::Blade(blade)) if blade.hands == Hands::Two);
    item.kind.def().wear.is_some_and(|wear| wear.one_place) && !two_handed
}

/// How an item fits its wearer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Exact,
    /// This many sizes off, with a penalty when penalties exist.
    Loose(u8),
}

/// Why something can't be put on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WearRefused {
    /// It isn't carried.
    NoSuchItem,
    AlreadyWorn,
    NotClothing,
    /// This body has no such place, or that place doesn't take this layer.
    WrongBody(Location),
    /// Made for a much bigger or smaller creature.
    WrongSize,
    /// Made for another number of fingers.
    WrongFingers,
    /// Something on the same layer is in the way.
    Clash(Id),
    /// Something on a higher layer covers one of its locations, or one of what
    /// would come off for it.
    Covered(Id),
    /// What is in its way won't come off: cursed, or blessed for someone else.
    Stuck(Id),
    /// What is in its way has no room in the inventory to go.
    NoRoom(Id),
}

impl WearRefused {
    /// Whether this might only be how it looks from here: magic that hasn't been found
    /// out would fit, so a client lets the server decide.
    pub fn worth_trying(self) -> bool {
        matches!(self, Self::WrongSize | Self::WrongFingers)
    }
}

/// Who is trying to take something off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Remover {
    Wearer,
    Other,
}

impl Made {
    /// How this fits a creature of this size, with its quality deciding how far any
    /// magic reaches: `None` if it doesn't. Past that reach, it is as if it had none.
    pub fn fit(&self, adjustable: bool, size: Size, quality: Option<Quality>) -> Option<Fit> {
        let off = self.size.apart(size);
        let reach = quality.map_or(0, Quality::magic_reach);
        if self.magical && off <= reach && reach > 0 {
            return Some(Fit::Exact);
        }
        if size < self.smallest || size > self.largest {
            return None;
        }
        Some(if adjustable || off == 0 {
            Fit::Exact
        } else {
            Fit::Loose(off)
        })
    }
}

/// Each location a worn item takes, with the layer it takes there: its own layer
/// everywhere, and Middle too where it is padded.
fn coverage(item: &Item) -> Vec<(Location, Layer)> {
    let Some(wear) = item.kind.def().wear else {
        return Vec::new();
    };
    let own = wear
        .locations
        .iter()
        .map(|&location| (location, wear.layer));
    let padding = item
        .padded()
        .iter()
        .map(|&location| (location, Layer::Middle));
    own.chain(padding).collect()
}

#[cfg(test)]
mod death_tests;
#[cfg(test)]
mod swap_tests;
#[cfg(test)]
mod tests;
