//! Whether something can be worn, and what covers what.

use crate::body::{Horns, Parts, Snout};
use crate::creature::{Layer, Location};
use crate::id::Id;
use crate::item::{Inventory, Item};
use crate::item_form::ItemForm;
use crate::wear::{Fit, WearRefused, coverage, made_for_fingers, takes_one_place};

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
        self.check_wear_new(item.top())
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
        if covers_face(item) && !takes_closed_helmets(&Parts::of(self.creature)) {
            return Err(WearRefused::WrongBody(Location::Head));
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
        let one_place = takes_one_place(item);
        for (location, layer) in coverage(item) {
            let here = || {
                self.worn
                    .iter()
                    .filter(move |other| coverage(other).contains(&(location, layer)))
            };
            let whole = |other: &&Item| !takes_one_place(other);
            if !one_place {
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
}

/// Whether this covers the face, as a closed helmet does, on its own or in a set.
fn covers_face(item: &Item) -> bool {
    let head = match &item.form {
        Some(ItemForm::Head(head)) => Some(*head),
        Some(ItemForm::Set(set)) => set.head,
        _ => None,
    };
    head.is_some_and(|head| head.face.closed())
}

/// Whether a closed helmet goes over this head: not over a snout, a beak or horns
/// (owner, 2026-11-04).
fn takes_closed_helmets(parts: &Parts) -> bool {
    parts.snout == Snout::Flat && parts.horns == Horns::None
}
