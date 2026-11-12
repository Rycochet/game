//! The Held column, between the clothes and the inventory (owner, 2026-11-07): a
//! slot for each hand, one above the other (in `body`'s markup). Here, what
//! each hand holds.

use bevy::prelude::*;
use shared::creature::Location;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::wear::takes_one_place;

/// The weapons held, but not one in the pointer's hand.
pub(super) fn held_now(inventory: &Inventory, picked_up: Option<Id>) -> Vec<&Item> {
    inventory
        .worn
        .iter()
        .filter(|item| Some(item.id) != picked_up)
        .filter(|item| {
            item.kind
                .def()
                .wear
                .is_some_and(|wear| wear.locations.contains(&Location::Held))
        })
        .collect()
}

/// What each hand holds: a two-handed weapon in both.
pub(super) fn in_hands<'a, 'b>(held: &'b [&'a Item]) -> [&'b [&'a Item]; 2] {
    match held {
        [] => [&[], &[]],
        [both] if !takes_one_place(both) => [&held[..1], &held[..1]],
        [_] => [&held[..1], &[]],
        _ => [&held[..1], &held[1..2]],
    }
}
