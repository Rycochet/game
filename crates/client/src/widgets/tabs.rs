//! Rows of buttons of which exactly one is chosen, as radio buttons are: joined
//! toggles to pick an option, and folder tabs to switch what is shown below them.

use bevy::prelude::*;

use super::{
    BUTTON_BORDER, BUTTON_PADDING, ButtonPressed, ButtonState, ButtonStyle, Pressable, SHADED_EDGE,
    TextSize, spawn_button,
};

/// Between folder tabs.
const TAB_GAP: f32 = 4.0;
/// The line under folder tabs, which the chosen one covers.
const TAB_LINE: f32 = 2.0;

/// A row, and which is chosen.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Tabs {
    pub(super) selected: usize,
}

/// A folder tab's own piece of the line under the row, over its bottom border, so
/// an unchosen tab's bottom is one straight line with no corners cut by its sides'
/// shading (owner, 2026-11-07). Hidden on the chosen tab, which is open below.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct TabFoot(Entity);

/// One of a row: its row, and where in it.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Tab {
    pub(super) row: Entity,
    pub(super) index: usize,
}

/// The player chose another of this row: its entity, and which.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabChosen(pub Entity, pub usize);

/// How a row's buttons are made, kept for making them again (`set_tabs`).
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct TabsLook {
    style: ButtonStyle,
    size: TextSize,
}

/// Joined buttons with these words, to pick one option, all on show, with `selected`
/// chosen. Returns the row, for telling which `TabChosen` is about.
pub fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    labels: &[&str],
    selected: usize,
    size: TextSize,
) -> Entity {
    let style = ButtonStyle::Plain;
    let look = TabsLook { style, size };
    let row = parent.spawn((Tabs { selected }, look, Node::default()));
    fill_row(row, labels, selected, look)
}

/// Folder tabs with these words, to switch between views of one thing, with
/// `selected` chosen. Returns the row, for telling which `TabChosen` is about.
pub fn spawn_tabs(
    parent: &mut ChildSpawnerCommands,
    labels: &[&str],
    selected: usize,
    size: TextSize,
) -> Entity {
    let style = ButtonStyle::Folder;
    let look = TabsLook { style, size };
    let node = Node {
        column_gap: px(TAB_GAP),
        border: UiRect::bottom(px(TAB_LINE)),
        ..default()
    };
    let edge = BorderColor::from(SHADED_EDGE);
    let row = parent.spawn((Tabs { selected }, look, node, edge));
    fill_row(row, labels, selected, look)
}

/// Gives a row these words instead of its own, with the first chosen. False
/// if `row` isn't one.
pub fn set_tabs(world: &mut World, row: Entity, labels: &[String]) -> bool {
    let Some(look) = world.get::<TabsLook>(row).copied() else {
        return false;
    };
    if let Some(mut tabs) = world.get_mut::<Tabs>(row) {
        tabs.selected = 0;
    }
    world.entity_mut(row).despawn_related::<Children>();
    let labels: Vec<&str> = labels.iter().map(String::as_str).collect();
    fill_row(world.commands().entity(row), &labels, 0, look);
    true
}

/// Puts the buttons in a row, and gives the row back.
fn fill_row(mut row: EntityCommands, labels: &[&str], selected: usize, look: TabsLook) -> Entity {
    let TabsLook { style, size } = look;
    let folder = style == ButtonStyle::Folder;
    let node = Node {
        padding: BUTTON_PADDING,
        // A folder tab stands on the line under the row.
        margin: UiRect::bottom(px(if folder { -TAB_LINE } else { 0.0 })),
        ..default()
    };
    let id = row.id();
    row.with_children(|row_inside| {
        for (index, label) in labels.iter().enumerate() {
            spawn_button(
                row_inside,
                state_of(index, selected),
                *label,
                size,
                node.clone(),
            )
            .insert((Tab { row: id, index }, style, Pressable))
            .with_children(|tab| {
                if folder {
                    let tab_entity = tab.target_entity();
                    tab.spawn(foot(tab_entity, index == selected));
                }
            });
        }
    });
    id
}

/// Covers the tab's bottom border, reaching under its sides' borders too.
fn foot(tab: Entity, chosen: bool) -> impl Bundle {
    (
        TabFoot(tab),
        Node {
            position_type: PositionType::Absolute,
            left: px(-BUTTON_BORDER),
            right: px(-BUTTON_BORDER),
            bottom: px(-BUTTON_BORDER),
            height: px(TAB_LINE),
            ..default()
        },
        BackgroundColor(SHADED_EDGE),
        if chosen {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        },
    )
}

/// Shows each tab's foot unless it is chosen.
pub(super) fn show_tab_feet(
    mut feet: Query<(&TabFoot, &mut Visibility)>,
    tabs: Query<&ButtonState, With<Tab>>,
) {
    for (foot, mut visibility) in &mut feet {
        let chosen = tabs
            .get(foot.0)
            .is_ok_and(|state| *state == ButtonState::Selected);
        visibility.set_if_neq(if chosen {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
}

fn state_of(index: usize, selected: usize) -> ButtonState {
    if index == selected {
        ButtonState::Selected
    } else {
        ButtonState::Normal
    }
}

/// Asks for another of this row to be chosen, as a press on it would but without
/// `TabChosen` (whoever asked knows).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChooseTab(pub Entity, pub usize);

/// Chooses the tabs asked for.
pub(super) fn choose_tabs(
    mut asked: MessageReader<ChooseTab>,
    mut rows: Query<&mut Tabs>,
    mut tabs: Query<(&Tab, &mut ButtonState)>,
) {
    for &ChooseTab(row, index) in asked.read() {
        let Ok(mut tabs_row) = rows.get_mut(row) else {
            continue;
        };
        tabs_row.selected = index;
        for (tab, mut state) in &mut tabs {
            if tab.row == row {
                state.set_if_neq(state_of(tab.index, index));
            }
        }
    }
}

/// A press on a tab chooses it. Choosing the one already chosen says nothing.
pub(super) fn press_tabs(
    mut pressed: MessageReader<ButtonPressed>,
    mut rows: Query<&mut Tabs>,
    mut tabs: Query<(&Tab, &mut ButtonState)>,
    mut chosen: MessageWriter<TabChosen>,
) {
    for press in pressed.read() {
        let Ok(tab) = tabs.get(press.0).map(|(tab, _)| *tab) else {
            continue;
        };
        let Ok(mut row) = rows.get_mut(tab.row) else {
            continue;
        };
        if row.selected == tab.index {
            continue;
        }
        row.selected = tab.index;
        chosen.write(TabChosen(tab.row, tab.index));
        for (other, mut state) in &mut tabs {
            if other.row == tab.row {
                state.set_if_neq(state_of(other.index, tab.index));
            }
        }
    }
}

#[cfg(test)]
mod tests;
