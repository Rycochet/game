//! Rows of buttons of which exactly one is chosen, as radio buttons are: joined
//! toggles to pick an option, and folder tabs to switch what is shown below them.

use bevy::prelude::*;

use super::{
    BUTTON_PADDING, ButtonPressed, ButtonState, ButtonStyle, Pressable, SHADED_EDGE, TextSize,
    spawn_button,
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

/// On a folder tab. The line under the row is the row's own bottom border, one
/// node, so it is straight at any interface size: an unchosen tab has no bottom
/// border and stands on it, so its sides end square on the line with no corners
/// cut by their shading (owner, 2026-11-07); the chosen one reaches down over
/// it, open below. (Each tab drawing its own piece of the line rounded apart
/// from the row's, a pixel off in the gaps at some sizes: owner, 2026-11-13.)
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct FolderTab;

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
            .insert_if(FolderTab, || folder);
        }
    });
    id
}

/// Where a folder tab's bottom is: on the line, with no border of its own, or
/// (chosen) over it, as far down as the line is thick, padded so its words
/// stay where an unchosen tab's are.
pub(super) fn bottom_of(chosen: bool) -> (Val, Val, Val) {
    let Val::Px(padding) = BUTTON_PADDING.bottom else {
        unreachable!("a button's padding is in pixels")
    };
    match chosen {
        true => (px(0), px(-TAB_LINE), px(padding + TAB_LINE)),
        false => (px(0), px(0), px(padding)),
    }
}

/// Fits each folder tab's bottom to whether it is chosen.
pub(super) fn show_tab_bottoms(mut tabs: Query<(&ButtonState, &mut Node), With<FolderTab>>) {
    for (state, mut node) in &mut tabs {
        let (border, margin, padding) = bottom_of(*state == ButtonState::Selected);
        if node.border.bottom != border
            || node.margin.bottom != margin
            || node.padding.bottom != padding
        {
            node.border.bottom = border;
            node.margin.bottom = margin;
            node.padding.bottom = padding;
        }
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
