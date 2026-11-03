//! Showing the menu's layers, and building a grid again when needed.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::FactSources;
use super::super::actions::kind_name;
use super::{
    COLUMN_WIDTH, DebugMenu, Entry, FONT, FilterBox, GAP, MenuEntry, MenuGrid, MenuRoot,
    PageButton, ROW, WarningButton, columns_in, columns_of, entries_on_show, rows_in,
};
use crate::common::{GameView, interface_area};
use crate::widgets::{ButtonState, Pressable, TextBox, spawn_check_button};

/// What a grid was last built from.
#[derive(PartialEq)]
pub(in crate::debug_menu) struct GridDrawn {
    entries: Vec<(Entry, ButtonState, Option<bool>)>,
    rows: usize,
    first_column: usize,
    fit: usize,
}

/// Shows or hides the menu's layers, and builds a grid again when what it says
/// changes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::debug_menu) fn show_menu(
    mut commands: Commands,
    menu: Res<DebugMenu>,
    (window, ui_scale): (Single<&Window, With<PrimaryWindow>>, Res<UiScale>),
    sources: FactSources,
    mut roots: Query<(&MenuRoot, &mut Visibility)>,
    mut warning: Single<
        &mut Visibility,
        (With<WarningButton>, Without<MenuRoot>, Without<PageButton>),
    >,
    grids: Query<(Entity, &MenuGrid)>,
    mut filter_boxes: Query<(&FilterBox, &mut TextBox)>,
    mut pages: Query<
        (&PageButton, &mut Visibility, &mut ButtonState),
        (Without<MenuRoot>, Without<WarningButton>),
    >,
    mut drawn: Local<[Option<GridDrawn>; 2]>,
) {
    warning.set_if_neq(if *sources.view.get() == GameView::Connecting {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    for (root, mut visibility) in &mut roots {
        let shows = menu.open && menu.level(root.0).is_some();
        visibility.set_if_neq(if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if !menu.open {
        return;
    }
    let facts = sources.facts();
    for (filter_box, mut text_box) in &mut filter_boxes {
        let filter = menu.level(filter_box.0).map_or("", |level| &level.filter);
        if text_box.text != filter {
            text_box.text = filter.to_owned();
        }
    }
    for (grid, &MenuGrid(layer)) in &grids {
        let Some(level) = menu.level(layer) else {
            continue;
        };
        let room = interface_area(&window, &ui_scale);
        let rows = rows_in(room.y, layer);
        let fit = columns_in(room.x, layer);
        let entries: Vec<_> = entries_on_show(layer, &level.filter)
            .into_iter()
            .enumerate()
            .map(|(at, entry)| {
                let highlighted = level.highlight == Some(at);
                let available = match entry {
                    Entry::Action(action) => action.is_available(&facts),
                    Entry::Kind(_) => true,
                };
                let state = match (available, highlighted) {
                    (true, true) => ButtonState::Selected,
                    (true, false) => ButtonState::Normal,
                    (false, true) => ButtonState::DisabledSelected,
                    (false, false) => ButtonState::Disabled,
                };
                let on = match entry {
                    Entry::Action(action) => action.is_on(&facts),
                    Entry::Kind(_) => None,
                };
                (entry, state, on)
            })
            .collect();
        let columns = columns_of(entries.len(), rows);
        for (page, mut visibility, mut state) in &mut pages {
            if page.layer != layer {
                continue;
            }
            visibility.set_if_neq(if columns > fit {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
            let can_go = if page.forward {
                level.first_column + fit < columns
            } else {
                level.first_column > 0
            };
            state.set_if_neq(if can_go {
                ButtonState::Normal
            } else {
                ButtonState::Disabled
            });
        }
        let now = GridDrawn {
            entries,
            rows,
            first_column: level.first_column,
            fit,
        };
        if drawn[layer].as_ref() == Some(&now) {
            continue;
        }
        commands.entity(grid).despawn_children();
        commands.entity(grid).with_children(|grid| {
            let on_show = now.entries.chunks(now.rows);
            for column in on_show.skip(now.first_column).take(now.fit) {
                grid.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(GAP),
                    width: px(COLUMN_WIDTH),
                    ..default()
                })
                .with_children(|column_node| {
                    for &(entry, state, on) in column {
                        let node = Node {
                            height: px(ROW),
                            ..default()
                        };
                        let label = match entry {
                            Entry::Action(action) => action.label(),
                            Entry::Kind(kind) => kind_name(kind),
                        };
                        spawn_check_button(column_node, state, on, label, FONT, node)
                            .insert((MenuEntry(entry), Pressable));
                    }
                });
            }
        });
        drawn[layer] = Some(now);
    }
}
