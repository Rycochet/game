//! Items written into text (`shared::item_text`): what is known of each, for
//! showing them, and dropping items onto boxes that take them.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;

/// How long what is known of an item is shown before it is asked after again,
/// while it is still on show, in seconds of real time.
pub const ITEM_KEPT_FOR: f64 = 60.0;
/// How long an answer is waited for before asking again.
const ASKED_FOR: f64 = 10.0;

/// What is known of an item written in text.
#[derive(Clone, Debug, PartialEq)]
pub enum KnownItem {
    /// Asked after at this time, with no answer yet.
    Asked(f64),
    /// As the server (or what the player carries) said at this time.
    Found(Box<Item>, f64),
    /// The server knew nothing of it at this time.
    Unknown(f64),
}

/// What each item written in text is, as far as the client knows (resource).
/// Whatever shows one says it `want`s it every frame; `client::item_text` asks
/// for those not known, or known too long ago, and fills in the answers.
#[derive(Resource, Default)]
pub struct KnownItems {
    items: BTreeMap<Id, KnownItem>,
    wanted: BTreeSet<Id>,
}

impl KnownItems {
    /// What is known of it, if anything has been asked.
    pub fn get(&self, id: Id) -> Option<&KnownItem> {
        self.items.get(&id)
    }

    /// It is on show at `now`: it is asked after unless known, or being asked,
    /// recently enough.
    pub fn want(&mut self, id: Id, now: f64) {
        let fresh = match self.items.get(&id) {
            Some(KnownItem::Asked(at)) => now - at < ASKED_FOR,
            Some(KnownItem::Found(_, at) | KnownItem::Unknown(at)) => now - at < ITEM_KEPT_FOR,
            None => false,
        };
        if !fresh {
            self.wanted.insert(id);
        }
    }

    /// The items to ask after, taken.
    pub fn take_wanted(&mut self) -> BTreeSet<Id> {
        std::mem::take(&mut self.wanted)
    }

    /// Sets what is known of it.
    pub fn set(&mut self, id: Id, known: KnownItem) {
        self.items.insert(id, known);
    }
}

/// On a text box that takes items, and the parts of it the pointer may be on:
/// the box it is.
#[derive(Component, Clone, Copy, Debug)]
pub struct TakesItemText(pub Entity);

/// The box taking items that something held is over now (resource).
#[derive(Resource, Default, PartialEq, Eq)]
pub struct HeldOverText(pub Option<Entity>);

/// An item was let go on a box that takes them: it is written in at its caret,
/// and the item goes back where it was.
#[derive(Message, Clone, Copy, Debug)]
pub struct DroppedOnText {
    pub on: Entity,
    pub item: Id,
}

#[cfg(test)]
mod tests;
