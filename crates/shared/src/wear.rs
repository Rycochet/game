//! Wearing things: the rules both sides check clothes by. See
//! `docs/context/shared/creature.md`.
//!
//! An item has one layer and the locations it covers. It fits a body if every one
//! of them exists there and takes that layer. It can only go on or come off if
//! nothing on a higher layer is at any of its locations.

use crate::armour::hands::Fingers;
use crate::creature::{CreatureKind, Layer, Location, Size};
use crate::id::Id;
use crate::item::{Favour, Inventory, Item, Made, Quality, WearDef};
use crate::item_form::ItemForm;

/// Whether something is made for a number of fingers, so only fits hands with that
/// many unless magical: anything over each finger, but not mittens, which take any
/// (the owner's decision), nor rings, which take one.
fn made_for_fingers(item: &Item, wear: &WearDef) -> bool {
    let mitten =
        matches!(&item.form, Some(ItemForm::Hands(hands)) if hands.fingers == Fingers::Mitten);
    !wear.one_place && wear.locations.contains(&Location::Fingers) && !mitten
}

/// How an item fits its wearer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Exact,
    /// This many sizes off, with a penalty when penalties exist.
    Loose(u8),
}

/// Why something can't be put on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Something on a higher layer covers one of its locations.
    Covered(Id),
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

impl Inventory {
    pub fn is_worn(&self, id: Id) -> bool {
        self.worn.iter().any(|item| item.id == id)
    }

    /// What is on a higher layer than any of these, at the same location.
    fn covering(&self, places: &[(Location, Layer)], not: Id) -> Option<Id> {
        self.worn
            .iter()
            .filter(|other| other.id != not)
            .find(|other| {
                coverage(other).iter().any(|&(theirs, their_layer)| {
                    places
                        .iter()
                        .any(|&(location, layer)| theirs == location && their_layer > layer)
                })
            })
            .map(|other| other.id)
    }

    /// What is on top of this worn item, stopping it coming off.
    pub fn covered_by(&self, id: Id) -> Option<Id> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        self.covering(&coverage(item), id)
    }

    /// Whether this carried item can be put on, and how it would fit.
    pub fn check_wear(&self, id: Id) -> Result<Fit, WearRefused> {
        if self.is_worn(id) {
            return Err(WearRefused::AlreadyWorn);
        }
        let item = self.find(id).ok_or(WearRefused::NoSuchItem)?;
        self.check_wear_new(item)
    }

    /// Whether this could be put on, carried or not, and how it would fit.
    pub fn check_wear_new(&self, item: &Item) -> Result<Fit, WearRefused> {
        let id = item.id;
        let wear = item.kind.def().wear.ok_or(WearRefused::NotClothing)?;
        let body = self.creature.def();
        let places = coverage(item);
        for &(location, layer) in &places {
            let part = body
                .part(location)
                .ok_or(WearRefused::WrongBody(location))?;
            if !part.layers.contains(&layer) {
                return Err(WearRefused::WrongBody(location));
            }
        }
        let fit = item
            .made
            .fit(wear.adjustable, body.size, item.quality)
            .ok_or(WearRefused::WrongSize)?;
        let whole_fingers = made_for_fingers(item, &wear);
        if whole_fingers && !item.is_magical() && item.made.fingers != body.fingers() {
            return Err(WearRefused::WrongFingers);
        }
        if let Some(other) = self.clash(item) {
            return Err(WearRefused::Clash(other));
        }
        if let Some(other) = self.covering(&places, id) {
            return Err(WearRefused::Covered(other));
        }
        Ok(fit)
    }

    /// What on the same layer is in its way. Things taking one place only get in
    /// each other's way once every place is taken.
    fn clash(&self, item: &Item) -> Option<Id> {
        let wear = item.kind.def().wear?;
        for (location, layer) in coverage(item) {
            let here = || {
                self.worn
                    .iter()
                    .filter(move |other| coverage(other).contains(&(location, layer)))
            };
            let whole = |other: &&Item| {
                other
                    .kind
                    .def()
                    .wear
                    .is_some_and(|theirs| !theirs.one_place)
            };
            if !wear.one_place {
                if let Some(other) = here().next() {
                    return Some(other.id);
                }
            } else if let Some(other) = here().find(whole) {
                return Some(other.id);
            } else {
                let places = self
                    .creature
                    .def()
                    .part(location)
                    .map_or(0, |part| part.places);
                if here().count() >= places as usize {
                    return here().next().map(|other| other.id);
                }
            }
        }
        None
    }

    /// Puts this carried item on, if it can go.
    pub fn wear(&mut self, id: Id) -> Result<Fit, WearRefused> {
        let fit = self.check_wear(id)?;
        let item = self.take(id).ok_or(WearRefused::NoSuchItem)?;
        self.worn.push(item);
        Ok(fit)
    }

    /// Puts on something not carried, if it can go on.
    pub fn wear_new(&mut self, item: Item) -> Result<Fit, WearRefused> {
        let fit = self.check_wear_new(&item)?;
        self.worn.push(item);
        Ok(fit)
    }

    /// Whether this person may take this off, by what lies on it: anything not worn
    /// may go. What this says belongs to whoever is acting, and not to `move_item` or
    /// `hand_over`: a body falling, or an ability, isn't someone trying.
    pub fn may_remove(&self, id: Id, by: Remover) -> bool {
        let Some(item) = self.worn.iter().find(|item| item.id == id) else {
            return true;
        };
        match item.favour {
            None => true,
            Some(Favour::Blessed) => by == Remover::Wearer,
            Some(Favour::Cursed) => false,
        }
    }

    /// Finds out that this worn thing is cursed or blessed.
    pub fn favour_found_out(&mut self, id: Id) {
        if let Some(item) = self.worn.iter_mut().find(|item| item.id == id) {
            item.known.favour = true;
        }
    }

    /// Whether putting this on just now was only possible by its magic, which gives
    /// it away. A thing that would fit anyway keeps its secret.
    pub fn magic_found_out(&mut self, id: Id) -> bool {
        let body = self.creature.def();
        let Some(item) = self.worn.iter_mut().find(|item| item.id == id) else {
            return false;
        };
        let Some(wear) = item.kind.def().wear else {
            return false;
        };
        if !item.is_magical() {
            return false;
        }
        let without = Made {
            magical: false,
            ..item.made
        };
        let plain = without.fit(wear.adjustable, body.size, item.quality);
        let with = item.made.fit(wear.adjustable, body.size, item.quality);
        let whole_fingers = made_for_fingers(item, &wear);
        let needed = (plain != with) || (whole_fingers && item.made.fingers != body.fingers());
        if needed {
            item.known.magic = true;
        }
        needed
    }

    /// As a client may see it: see `Item::hide_secrets`.
    pub fn hide_secrets(&mut self) {
        let placed = self.pockets.iter_mut().map(|placed| &mut placed.item);
        for item in placed.chain(&mut self.worn).chain(&mut self.bin) {
            item.hide_secrets();
        }
    }

    /// Takes out something to hand over, with all in it, unless it isn't carried or
    /// is worn under something.
    pub fn hand_over(&mut self, id: Id) -> Option<Item> {
        if self.covered_by(id).is_some() {
            return None;
        }
        self.take(id)
    }

    /// Everything carried and worn, each with all in it, as what is left of a body.
    pub fn spilled(self) -> Vec<Item> {
        let pockets = self.pockets.into_iter().map(|placed| placed.item);
        pockets.chain(self.worn).collect()
    }

    /// How a worn item fits its wearer.
    pub fn worn_fit(&self, id: Id) -> Option<Fit> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        let wear = item.kind.def().wear?;
        item.made
            .fit(wear.adjustable, self.creature.def().size, item.quality)
    }

    /// For a character becoming another kind of creature (dev tools): everything
    /// comes off and goes in the new pockets, as far as it fits. Gives back what
    /// didn't.
    pub fn change_body(&mut self, kind: CreatureKind) -> Vec<Item> {
        let old = std::mem::take(self);
        self.creature = kind;
        self.outfitted = old.outfitted;
        self.pockets_size = kind.def().pockets;
        let everything = old
            .pockets
            .into_iter()
            .map(|placed| placed.item)
            .chain(old.worn);
        everything
            .filter_map(|item| self.give(item).err())
            .collect()
    }
}

#[cfg(test)]
mod tests;
