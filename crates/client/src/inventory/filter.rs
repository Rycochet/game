//! Filters: a small button above each grid, opening a list of categories. What
//! isn't in the chosen one is faded, but can still be picked up. Each grid has its
//! own, which goes with its panel, so a panel always opens showing everything.

use bevy::prelude::*;
use shared::item::{Holder, Item, ItemCategory};

use super::Carried;
use super::ground::Ground;
use crate::widgets::{ButtonTooltip, Dropdown, spawn_icon_dropdown};

// Split into files (`cargo xtask lint` keeps files small).
mod icons;

use icons::filter_icon;

/// A filter's button, and the grid it is for: `None` for the ground.
#[derive(Component, Clone, Copy)]
pub(super) struct GridFilter(pub Option<Holder>);

/// The pictures for All and each category, in the order of the list.
#[derive(Resource)]
pub(super) struct FilterIcons(Vec<Handle<Image>>);

pub(super) fn make_icons(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let all = std::iter::once(None).chain(ItemCategory::ALL.map(Some));
    let icons = all.map(|category| images.add(filter_icon(category)));
    commands.insert_resource(FilterIcons(icons.collect()));
}

/// A filter's button, showing All.
pub(super) fn spawn_filter(parent: &mut ChildSpawnerCommands, of: Option<Holder>) {
    let names = ItemCategory::ALL.map(|category| category.name().to_owned());
    let dropdown = Dropdown {
        options: std::iter::once("All".to_owned()).chain(names).collect(),
        ..default()
    };
    spawn_icon_dropdown(parent, dropdown, Node::default()).insert((
        GridFilter(of),
        ButtonTooltip("Filter".into()),
        super::ui_ids::grid_id(of, "filter"),
    ));
}

/// The category a filter shows clearly, or `None` for all of them.
pub(super) fn chosen(dropdown: &Dropdown) -> Option<ItemCategory> {
    let at = dropdown.selected.checked_sub(1)?;
    ItemCategory::ALL.get(at).copied()
}

/// Whether a filter fades this. A stack goes by its top, which is what shows.
pub(super) fn dims(filter: Option<ItemCategory>, item: &Item) -> bool {
    filter.is_some_and(|only| item.top().kind.category() != only)
}

/// Gives new filters their pictures, greys the categories their grid has nothing
/// of, and tells the ground's grid what its filter shows.
pub(super) fn keep_filters(
    icons: Res<FilterIcons>,
    carried: Res<Carried>,
    mut ground: ResMut<Ground>,
    mut filters: Query<(&GridFilter, &mut Dropdown)>,
) {
    let mut ground_filter = None;
    for (filter, mut dropdown) in &mut filters {
        if dropdown.icons.is_empty() {
            dropdown.icons.clone_from(&icons.0);
        }
        let had: Vec<ItemCategory> = match filter.0 {
            Some(holder) => carried
                .0
                .grid(holder)
                .into_iter()
                .flatten()
                .map(|placed| placed.item.top().kind.category())
                .collect(),
            None => {
                ground_filter = chosen(&dropdown);
                let items = ground.grid.items();
                items.map(|item| item.top().kind.category()).collect()
            }
        };
        let greyed = greyed(&had);
        if dropdown.greyed != greyed {
            dropdown.greyed = greyed;
        }
    }
    if ground.grid.filter != ground_filter {
        ground.grid.filter = ground_filter;
        ground.grid.changed = true;
    }
}

/// Which options to grey, All first: categories with nothing in them.
fn greyed(had: &[ItemCategory]) -> Vec<bool> {
    let categories = ItemCategory::ALL.iter();
    let empty = categories.map(|category| !had.contains(category));
    std::iter::once(false).chain(empty).collect()
}

#[cfg(test)]
mod tests;
