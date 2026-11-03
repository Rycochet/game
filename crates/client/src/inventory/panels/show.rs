//! Showing what is in each open grid, and how much it weighs.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Holder, Inventory, Placed};

use super::super::{Carried, Holding};
use super::{ItemCell, ItemLayer, WeightText};
use crate::widgets::{ItemLook, spawn_item_cells};

pub(super) fn kilograms(grams: u32) -> String {
    format!("{:.2} kg", grams as f32 / 1000.0)
}

/// Draws what is in each open grid again when it changes. What is held isn't
/// drawn where it was, though it is still there until dropped.
#[allow(clippy::too_many_arguments)]
pub(in crate::inventory) fn show_grids(
    carried: Res<Carried>,
    holding: Res<Holding>,
    new_layers: Query<(), Added<ItemLayer>>,
    layers: Query<(Entity, &ItemLayer)>,
    mut weights: Query<(&WeightText, &mut Text)>,
    mut commands: Commands,
    mut drawn_without: Local<Option<Id>>,
) {
    let held = holding.held.map(|held| held.item.id);
    if !carried.is_changed() && new_layers.is_empty() && *drawn_without == held {
        return;
    }
    *drawn_without = held;
    for (entity, layer) in &layers {
        commands.entity(entity).despawn_related::<Children>();
        let Some(grid) = carried.0.grid(layer.holder) else {
            continue;
        };
        commands.entity(entity).with_children(|layer| {
            for placed in grid.iter().filter(|placed| Some(placed.item.id) != held) {
                spawn_placed(layer, placed);
            }
        });
    }
    for (weight, mut text) in &mut weights {
        **text = weight_words(&carried.0, weight.holder);
    }
}

pub(super) fn spawn_placed(layer: &mut ChildSpawnerCommands, placed: &Placed) {
    let (id, rotation) = (placed.item.id, placed.rotation);
    spawn_item_cells(
        layer,
        ItemLook::of(&placed.item).turned(rotation),
        placed.shape(),
        (placed.x as f32, placed.y as f32),
        true,
        |x, y| ItemCell {
            id,
            rotation,
            grip: (x, y),
        },
    );
}

pub(super) fn weight_words(carried: &Inventory, holder: Holder) -> String {
    match holder {
        Holder::Pockets => format!("Carrying {}", kilograms(carried.total_weight())),
        Holder::Inside(id) => {
            let Some(item) = carried.find(id) else {
                return String::new();
            };
            let held: u32 = item
                .contents
                .iter()
                .map(|placed| placed.item.total_weight())
                .sum();
            match item.kind.def().container.and_then(|bag| bag.max_total) {
                Some(most) => format!("Holding {} of {}", kilograms(held), kilograms(most)),
                None => format!("Holding {}", kilograms(held)),
            }
        }
    }
}
