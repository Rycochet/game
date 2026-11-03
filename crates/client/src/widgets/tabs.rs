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
    selected: usize,
}

/// One of a row: its row, and where in it.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Tab {
    row: Entity,
    index: usize,
}

/// The player chose another of this row: its entity, and which.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabChosen(pub Entity, pub usize);

/// Joined buttons with these words, to pick one option, all on show, with `selected`
/// chosen. Returns the row, for telling which `TabChosen` is about.
pub fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    labels: &[&str],
    selected: usize,
    size: TextSize,
) -> Entity {
    let node = Node {
        padding: BUTTON_PADDING,
        ..default()
    };
    let row = parent.spawn((Tabs { selected }, Node::default())).id();
    spawn_row(
        parent,
        row,
        labels,
        selected,
        size,
        node,
        ButtonStyle::Plain,
    );
    row
}

/// Folder tabs with these words, to switch between views of one thing, with
/// `selected` chosen. Returns the row, for telling which `TabChosen` is about.
pub fn spawn_tabs(
    parent: &mut ChildSpawnerCommands,
    labels: &[&str],
    selected: usize,
    size: TextSize,
) -> Entity {
    let node = Node {
        padding: BUTTON_PADDING,
        margin: UiRect::bottom(px(-TAB_LINE)),
        ..default()
    };
    let row = parent
        .spawn((
            Tabs { selected },
            Node {
                column_gap: px(TAB_GAP),
                border: UiRect::bottom(px(TAB_LINE)),
                ..default()
            },
            BorderColor::from(SHADED_EDGE),
        ))
        .id();
    spawn_row(
        parent,
        row,
        labels,
        selected,
        size,
        node,
        ButtonStyle::Folder,
    );
    row
}

/// Puts the buttons in a row.
fn spawn_row(
    parent: &mut ChildSpawnerCommands,
    row: Entity,
    labels: &[&str],
    selected: usize,
    size: TextSize,
    node: Node,
    style: ButtonStyle,
) {
    parent.commands().entity(row).with_children(|row_inside| {
        for (index, label) in labels.iter().enumerate() {
            spawn_button(
                row_inside,
                state_of(index, selected),
                *label,
                size,
                node.clone(),
            )
            .insert((Tab { row, index }, style, Pressable));
        }
    });
}

fn state_of(index: usize, selected: usize) -> ButtonState {
    if index == selected {
        ButtonState::Selected
    } else {
        ButtonState::Normal
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
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn exactly_one_tab_stays_chosen() {
        let mut app = App::new();
        app.add_message::<ButtonPressed>()
            .add_message::<TabChosen>();
        let row = app.world_mut().spawn(Tabs { selected: 0 }).id();
        let tab = |index| (Tab { row, index }, state_of(index, 0));
        let first = app.world_mut().spawn(tab(0)).id();
        let second = app.world_mut().spawn(tab(1)).id();

        // The one already chosen: nothing happens.
        app.world_mut().write_message(ButtonPressed(first));
        app.world_mut().run_system_once(press_tabs).unwrap();
        assert_eq!(
            app.world()
                .resource::<Messages<TabChosen>>()
                .iter_current_update_messages()
                .count(),
            0
        );

        app.world_mut().write_message(ButtonPressed(second));
        app.world_mut().run_system_once(press_tabs).unwrap();
        let state = |entity| *app.world().get::<ButtonState>(entity).unwrap();
        assert_eq!(state(first), ButtonState::Normal);
        assert_eq!(state(second), ButtonState::Selected);
        let chosen: Vec<_> = app
            .world()
            .resource::<Messages<TabChosen>>()
            .iter_current_update_messages()
            .copied()
            .collect();
        assert_eq!(chosen, vec![TabChosen(row, 1)]);
    }
}
