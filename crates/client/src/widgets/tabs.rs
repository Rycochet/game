//! A row of tabs: buttons of which exactly one is chosen, as radio buttons are.

use bevy::prelude::*;

use super::{BUTTON_PADDING, ButtonPressed, ButtonState, Pressable, spawn_button};

const TAB_GAP: f32 = 4.0;

/// A row of tabs, and which is chosen.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Tabs {
    selected: usize,
}

/// One tab: its row, and where in it.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Tab {
    row: Entity,
    index: usize,
}

/// The player chose another tab of this row: its entity, and which.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabChosen(pub Entity, pub usize);

/// A row of tabs with these words, with `selected` chosen. Returns the row, for
/// telling which `TabChosen` is about.
pub fn spawn_tabs(
    parent: &mut ChildSpawnerCommands,
    labels: &[&str],
    selected: usize,
    font_size: f32,
) -> Entity {
    let mut row = parent.spawn((
        Tabs { selected },
        Node {
            column_gap: px(TAB_GAP),
            ..default()
        },
    ));
    let row_entity = row.id();
    row.with_children(|row| {
        for (index, label) in labels.iter().enumerate() {
            let node = Node {
                padding: BUTTON_PADDING,
                ..default()
            };
            spawn_button(row, state_of(index, selected), *label, font_size, node).insert((
                Tab {
                    row: row_entity,
                    index,
                },
                Pressable,
            ));
        }
    });
    row_entity
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
