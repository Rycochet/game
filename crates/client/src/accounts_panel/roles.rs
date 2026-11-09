//! The Roles page: a role's name and the title shown in its place, if any.

use bevy::prelude::*;

use super::FONT;
use super::fill::{control, labelled, words};
use super::state::{FORM, Panel};
use crate::widgets::{
    ButtonState, ButtonStyle, Dropdown, Pressable, TextBox, UiId, spawn_button, spawn_dropdown,
    spawn_text_box,
};

#[derive(Component)]
pub(super) struct RoleList;

#[derive(Component)]
pub(super) struct RoleNameBox;

#[derive(Component)]
pub(super) struct RoleTitleBox;

#[derive(Component)]
pub(super) struct SaveButton;

pub(super) fn spawn_roles(page: &mut ChildSpawnerCommands, panel: &Panel) {
    if panel.roles.is_empty() {
        page.spawn(words("Asking the server..."));
        return;
    }
    labelled(page, "Role", |row| {
        let roles = Dropdown {
            options: panel.roles.iter().map(|b| b.id.0.clone()).collect(),
            selected: panel.role,
            ..default()
        };
        spawn_dropdown(row, roles, FONT, control(240.0))
            .insert((RoleList, UiId::new(FORM, "role")));
    });
    for (label, text, name) in [
        ("Name", &panel.role_name, "role_name"),
        ("Title", &panel.role_title, "role_title"),
    ] {
        labelled(page, label, |row| {
            let words = TextBox {
                text: text.clone(),
                takes_keys: true,
                ..default()
            };
            let mut made = spawn_text_box(row, words, FONT, control(240.0));
            made.insert(UiId::new(FORM, name));
            match name {
                "role_name" => made.insert(RoleNameBox),
                _ => made.insert(RoleTitleBox),
            };
        });
    }
    page.spawn(words("A title, if given, is shown instead of the name."));
    spawn_button(page, ButtonState::Normal, "Save", FONT, control(110.0)).insert((
        SaveButton,
        Pressable,
        ButtonStyle::Main,
        UiId::new(FORM, "save"),
    ));
}
