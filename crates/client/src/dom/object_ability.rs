//! Abilities in `object-grid` and `object-slot` (`type="ability"`): a list of
//! abilities flowing in rows, and a slot holding one, which draw themselves.
//! Whoever owns them says what they show (`AbilityGridShown`,
//! `AbilitySlotShown`); what a press, a drag or a drop there does is still its
//! owner's, which finds the ability by its `AbilityCell`.
//!
//! Any node with one of these is drawn so: the element in markup, or a node a
//! screen not yet moved to markup spawns.

mod slot;

use std::collections::HashMap;

use bevy::prelude::*;
use shared::ability::Ability;
pub(super) use slot::draw_ability_slots;

use crate::widgets::{ABILITY, AbilityLook, spawn_ability};

/// Between abilities in a list.
const GAP: f32 = 6.0;

/// An ability as a grid or a slot shows it, and as it is held.
#[derive(Clone, Debug, PartialEq)]
pub struct AbilityThing {
    pub ability: Ability,
    pub look: AbilityLook,
}

/// The abilities an `<object type="ability" for="<name>"/>` can show, by
/// name, as whoever knows them (`abilities`) last said: how each looks now.
#[derive(Resource, Default)]
pub struct KnownAbilities(HashMap<String, AbilityThing>);

impl KnownAbilities {
    /// Says how the ability called `name` looks now.
    pub fn set(&mut self, name: impl Into<String>, thing: AbilityThing) {
        self.0.insert(name.into(), thing);
    }

    pub fn get(&self, name: &str) -> Option<&AbilityThing> {
        self.0.get(name)
    }
}

/// A list of abilities, set by its owner. They have no shape: each takes one
/// place, `columns` to a row. It is drawn again when this changes.
#[derive(Component, Clone, Debug, Default, PartialEq)]
#[require(Node)]
pub struct AbilityGridShown {
    pub columns: u8,
    pub things: Vec<AbilityThing>,
}

/// What a slot for an ability shows, set by its owner. It is drawn again when
/// this changes.
#[derive(Component, Clone, Debug, Default, PartialEq)]
#[require(Node)]
pub struct AbilitySlotShown {
    pub thing: Option<AbilityThing>,
    /// Its key's short name, over its bottom right.
    pub key: String,
    /// Used by itself: a sliding metal edge.
    pub auto: bool,
    /// A toggle that is on: a steady green edge.
    pub on: bool,
    /// Waiting to be used: stripes chasing round inside it.
    pub queued: bool,
    /// It has a shade over it (`SlotShade`), which its owner sizes.
    pub shade: bool,
    /// Something held is over it: green if it may go here, red if not.
    pub aim: Option<bool>,
}

/// On each ability drawn in a list or a slot: which list or slot, and which
/// ability.
#[derive(Component, Clone, Copy, Debug)]
pub struct AbilityCell {
    pub of: Entity,
    pub ability: Ability,
}

/// On a part of a drawn slot its owner may want to find: the slot it is of.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotPart {
    /// The edge of a toggle that is on.
    On(Entity),
    /// The stripes of one waiting to be used.
    Queued(Entity),
    /// The shade over it: a node at its bottom, as tall as its owner makes it.
    Shade(Entity),
}

/// Draws each list of abilities again when what it shows changes.
pub(super) fn draw_ability_grids(
    mut commands: Commands,
    changed: Query<(Entity, &AbilityGridShown), Changed<AbilityGridShown>>,
) {
    for (grid, shown) in &changed {
        let columns = shown.columns.max(1) as f32;
        let node = Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(GAP),
            row_gap: px(GAP),
            width: px(columns * ABILITY + (columns - 1.0) * GAP),
            ..default()
        };
        let mut entity = commands.entity(grid);
        entity.insert(node).despawn_related::<Children>();
        entity.with_children(|list| {
            for thing in &shown.things {
                let frame = spawn_ability(list, thing.look.clone());
                let cell = AbilityCell {
                    of: grid,
                    ability: thing.ability,
                };
                list.commands().entity(frame).insert(cell);
            }
        });
    }
}

#[cfg(test)]
mod tests;
