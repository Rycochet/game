//! Showing what is in each open grid, and how much it weighs (`widgets::weight_words`).

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Holder, Inventory, Placed};

use super::super::filter::{GridFilter, chosen, dims};
use super::super::{Carried, Holding};
use super::shown::shown_size;
use super::{GridOwner, WeightText};
use crate::common::CarryLimit;
use crate::dom::{GridShown, GridThing};
use crate::widgets::{Dropdown, weight_words};

/// Says what is in each open grid again when it changes, or its filter does, and
/// how big it is. What is held isn't shown where it was, though it is still there
/// until dropped.
pub(in crate::inventory) fn show_grids(
    carried: Res<Carried>,
    holding: Res<Holding>,
    new_grids: Query<(), Added<GridOwner>>,
    mut grids: Query<(&GridOwner, &mut GridShown)>,
    filters: Query<(&GridFilter, Ref<Dropdown>)>,
    mut drawn_without: Local<Option<Id>>,
) {
    let held = holding.held.map(|held| held.item.id);
    let filtered = filters.iter().any(|(_, dropdown)| dropdown.is_changed());
    if !carried.is_changed() && new_grids.is_empty() && *drawn_without == held && !filtered {
        return;
    }
    *drawn_without = held;
    for (owner, mut shown) in &mut grids {
        let holder = owner.holder;
        let filter = filters
            .iter()
            .find(|(filter, _)| filter.0 == Some(holder))
            .and_then(|(_, dropdown)| chosen(&dropdown));
        let placed = carried
            .0
            .grid(holder)
            .into_iter()
            .flat_map(|grid| grid.iter());
        let things = placed
            .filter(|placed| Some(placed.item.id) != held)
            .map(|placed| thing(placed, dims(filter, &placed.item)))
            .collect();
        let (columns, rows) = shown_size(&carried.0, holder).unwrap_or((shown.columns, shown.rows));
        let now = GridShown {
            columns,
            rows,
            things,
            aim: shown.aim.clone(),
            takes_gifts: None,
        };
        // (Only when it differs: the grid draws itself again when changed.)
        if *shown != now {
            *shown = now;
        }
    }
}

/// Writes each open grid's weight under it again when what is carried, or the most
/// that can be, changes.
pub(in crate::inventory) fn show_weights(
    carried: Res<Carried>,
    limit: Res<CarryLimit>,
    new_weights: Query<(), Added<WeightText>>,
    mut weights: Query<(&WeightText, &mut Text)>,
) {
    if !carried.is_changed() && !limit.is_changed() && new_weights.is_empty() {
        return;
    }
    for (weight, mut text) in &mut weights {
        **text = grid_weight(&carried.0, weight.holder, limit.0);
    }
}

/// A placed item, as a grid shows it.
fn thing(placed: &Placed, dimmed: bool) -> GridThing {
    GridThing {
        item: placed.item.clone(),
        x: placed.x,
        y: placed.y,
        rotation: placed.rotation,
        dimmed,
    }
}

pub(super) fn grid_weight(carried: &Inventory, holder: Holder, limit: u32) -> String {
    match holder {
        Holder::Inventory => weight_words(carried.total_weight(), Some(limit)),
        Holder::Inside(id) => {
            let Some(item) = carried.find(id) else {
                return String::new();
            };
            let held: u32 = item
                .contents
                .iter()
                .map(|placed| placed.item.total_weight())
                .sum();
            weight_words(
                held,
                item.kind.def().container.and_then(|bag| bag.max_total),
            )
        }
    }
}
