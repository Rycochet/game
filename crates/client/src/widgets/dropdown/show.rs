//! Keeping a dropdown's words and arrow up to date.

use bevy::prelude::*;

use super::super::{BUTTON_BORDER, ButtonState, INK, Pressable, spawn_button};
use super::{
    Dropdown, FONT, GAP, ListOption, ListRows, OpenList, ROW, ROW_ICON, ROWS, label_of, on_show,
};

/// Keeps a dropdown's words or picture and an open list's rows up to date.
pub(in crate::widgets) fn show_dropdowns(
    mut commands: Commands,
    mut open: ResMut<OpenList>,
    changed: Query<(&Dropdown, &Children), Changed<Dropdown>>,
    dropdowns: Query<&Dropdown>,
    mut labels: Query<&mut Text>,
    mut pictures: Query<&mut ImageNode>,
    rows: Query<Entity, With<ListRows>>,
) {
    for (dropdown, children) in &changed {
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = label_of(dropdown);
            }
            let icon = dropdown.icons.get(dropdown.selected);
            if let (Ok(mut picture), Some(icon)) = (pictures.get_mut(child), icon)
                && picture.image != *icon
            {
                picture.image = icon.clone();
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
            let state = if dropdown.is_greyed(at) {
                ButtonState::Disabled
            } else if now.2 == Some(row) {
                ButtonState::Selected
            } else {
                ButtonState::Normal
            };
            let node = Node {
                height: px(ROW),
                ..default()
            };
            let mut button = spawn_button(holder, state, dropdown.options[at].clone(), FONT, node);
            button.insert((ListOption(at), Pressable));
            if let Some(icon) = dropdown.icons.get(at) {
                with_icon(&mut button, icon.clone());
            }
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

/// Puts a picture before an option's words, and both at its left.
fn with_icon(button: &mut EntityCommands, icon: Handle<Image>) {
    button.insert(Node {
        height: px(ROW),
        border: UiRect::all(px(BUTTON_BORDER)),
        padding: UiRect::horizontal(px(GAP * 2.0)),
        column_gap: px(GAP * 2.0),
        align_items: AlignItems::Center,
        ..default()
    });
    let node = Node {
        width: px(ROW_ICON),
        height: px(ROW_ICON),
        flex_shrink: 0.0,
        ..default()
    };
    let picture = button.commands().spawn((ImageNode::new(icon), node)).id();
    button.insert_children(0, &[picture]);
}
