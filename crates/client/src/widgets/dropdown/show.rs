//! Keeping a dropdown's words and arrow up to date.

use bevy::prelude::*;

use super::super::{ButtonState, INK, Pressable, spawn_button};
use super::{Dropdown, FONT, ListOption, ListRows, OpenList, ROW, ROWS, label_of, on_show};

/// Keeps a dropdown's words and an open list's rows up to date.
pub(in crate::widgets) fn show_dropdowns(
    mut commands: Commands,
    mut open: ResMut<OpenList>,
    changed: Query<(&Dropdown, &Children), Changed<Dropdown>>,
    dropdowns: Query<&Dropdown>,
    mut labels: Query<&mut Text>,
    rows: Query<Entity, With<ListRows>>,
) {
    for (dropdown, children) in &changed {
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = label_of(dropdown);
            }
        }
    }
    let Some(list) = &mut open.0 else {
        return;
    };
    let (Ok(dropdown), Ok(holder)) = (dropdowns.get(list.dropdown), rows.single()) else {
        return;
    };
    let shown = on_show(&dropdown.options, &list.filter);
    let now = (shown, list.first, list.highlight);
    if list.drawn.as_ref() == Some(&now) {
        return;
    }
    commands.entity(holder).despawn_children();
    commands.entity(holder).with_children(|holder| {
        if now.0.is_empty() {
            holder.spawn((Text::new("No matches"), FONT.font(), TextColor(INK)));
        }
        for (row, &at) in now.0.iter().enumerate().skip(now.1).take(ROWS) {
            let state = if now.2 == Some(row) {
                ButtonState::Selected
            } else {
                ButtonState::Normal
            };
            let node = Node {
                height: px(ROW),
                ..default()
            };
            spawn_button(holder, state, dropdown.options[at].clone(), FONT, node)
                .insert((ListOption(at), Pressable));
        }
        if now.0.len() > ROWS {
            let last = (now.1 + ROWS).min(now.0.len());
            holder.spawn((
                Text::new(format!("{} to {last} of {}", now.1 + 1, now.0.len())),
                FONT.font(),
                TextLayout::justify(Justify::Center),
                TextColor(INK),
            ));
        }
    });
    list.drawn = Some(now);
}
