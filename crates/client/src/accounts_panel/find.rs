//! The Find page: a character's name to look up, then their account's role,
//! and, if the player may change it, a role to give, for how long and why.

use bevy::prelude::*;

use super::FONT;
use super::fill::{control, labelled, words};
use super::state::{FOR, FORM, Panel};
use crate::widgets::{
    ButtonState, ButtonStyle, Dropdown, Pressable, TextBox, UiId, spawn_button, spawn_dropdown,
    spawn_text_box,
};

#[derive(Component)]
pub(super) struct NameBox;

#[derive(Component)]
pub(super) struct FindButton;

#[derive(Component)]
pub(super) struct GiveRoleList;

#[derive(Component)]
pub(super) struct GiveForList;

#[derive(Component)]
pub(super) struct ReasonBox;

#[derive(Component)]
pub(super) struct ApplyButton;

pub(super) fn spawn_find(page: &mut ChildSpawnerCommands, panel: &Panel) {
    labelled(page, "Character", |row| {
        let name = TextBox {
            text: panel.name.clone(),
            takes_keys: true,
            ..default()
        };
        spawn_text_box(row, name, FONT, control(240.0)).insert((NameBox, UiId::new(FORM, "name")));
        spawn_button(row, ButtonState::Normal, "Find", FONT, control(90.0)).insert((
            FindButton,
            Pressable,
            ButtonStyle::Main,
            UiId::new(FORM, "find"),
        ));
    });
    let Some(found) = &panel.found else {
        return;
    };
    let mut about = format!("{}: {}", found.name, found.role.shown());
    if let Some(until) = &found.until {
        about += &format!(" until {until}");
    }
    if !found.reason.is_empty() {
        about += &format!(" ({})", found.reason);
    }
    page.spawn(words(about));
    if !found.others.is_empty() {
        page.spawn(words(format!("Also plays {}.", found.others.join(", "))));
    }
    if found.may_give.is_empty() {
        let why = if found.yours {
            "You can't change your own role."
        } else if found.guest {
            "A guest can't be given a role."
        } else {
            "You can't change this account's role."
        };
        page.spawn(words(why));
        return;
    }
    labelled(page, "Role", |row| {
        let roles = Dropdown {
            options: found
                .may_give
                .iter()
                .map(|b| b.shown().to_owned())
                .collect(),
            selected: panel.give_role,
            ..default()
        };
        spawn_dropdown(row, roles, FONT, control(240.0))
            .insert((GiveRoleList, UiId::new(FORM, "give_role")));
    });
    labelled(page, "For", |row| {
        let times = Dropdown {
            options: FOR.iter().map(|(words, _)| (*words).to_owned()).collect(),
            selected: panel.give_for,
            ..default()
        };
        spawn_dropdown(row, times, FONT, control(240.0))
            .insert((GiveForList, UiId::new(FORM, "give_for")));
    });
    labelled(page, "Why", |row| {
        let reason = TextBox {
            text: panel.reason.clone(),
            takes_keys: true,
            ..default()
        };
        spawn_text_box(row, reason, FONT, control(340.0))
            .insert((ReasonBox, UiId::new(FORM, "reason")));
    });
    spawn_button(page, ButtonState::Normal, "Apply", FONT, control(110.0)).insert((
        ApplyButton,
        Pressable,
        ButtonStyle::Main,
        UiId::new(FORM, "apply"),
    ));
}
