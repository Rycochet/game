//! The Held column, between the clothes and the inventory (owner, 2026-11-07): a
//! slot for each hand, one above the other.

use bevy::prelude::*;
use shared::creature::Location;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::wear::takes_one_place;

use super::body::{BodyNode, spawn_slot, words};
use crate::widgets::CELL;

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

/// The column, for a body with somewhere to hold things.
pub(super) fn spawn_held_column(
    parent: &mut ChildSpawnerCommands,
    inventory: &Inventory,
    picked_up: Option<Id>,
    god: bool,
) {
    let def = inventory.creature.def();
    if !def.body.iter().any(|part| part.location == Location::Held) {
        return;
    }
    let holding = held_now(inventory, picked_up);
    let hands = in_hands(&holding);
    parent
        .spawn((
            BodyNode,
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|column| {
            // Its heading level with the layers', under a line as tall as the
            // table's first.
            column.spawn(words(" ", CELL));
            column.spawn(words("Held", CELL));
            for hand in hands {
                spawn_slot(column, inventory, hand, god);
            }
        });
}
