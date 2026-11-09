//! Settings' Gameplay page: "Shown As", the role others see the character as,
//! for accounts that may be shown as another role (staff playing as a plain
//! player, say). Asked of the server (`ShowRoleAs`), which says what it now
//! is (`ShownRole`). Here rather than on the Character page, which players
//! share pictures of.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::role::{RoleId, ShownRole};

use super::rows::{spawn_dropdown_row, tip};
use crate::connection::{FromServer, ToServer};
use crate::widgets::{Dropdown, DropdownChosen, UiId};

const WIDTH: f32 = 200.0;

const WORDS: &str = "The role others see this character as. Staff are shown with a mark before \
     their name; shown as Player, there is none.";

/// The character's shown role and what else it may be, as the server last said.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::system_menu) struct ShownAs(pub Option<ShownRole>);

/// The dropdown, with the role each option is.
#[derive(Component)]
pub(in crate::system_menu) struct ShownList(Vec<RoleId>);

/// Keeps what the server says.
pub(in crate::system_menu) fn keep_shown_role(
    mut from_server: MessageReader<FromServer>,
    mut shown: ResMut<ShownAs>,
) {
    for FromServer(message) in from_server.read() {
        let now = match message {
            ServerMessage::Welcome { shown, .. } | ServerMessage::ShownRole(shown) => shown,
            _ => continue,
        };
        if shown.0.as_ref() != Some(now) {
            shown.0 = Some(now.clone());
        }
    }
}

/// Whether the row is there: only with another role to be shown as.
pub(in crate::system_menu) fn offered(shown: &ShownAs) -> Option<&ShownRole> {
    shown.0.as_ref().filter(|shown| shown.choices.len() > 1)
}

/// Adds the row to the Gameplay page.
pub(in crate::system_menu) fn spawn_shown_row(page: &mut ChildSpawnerCommands, shown: &ShownRole) {
    let dropdown = Dropdown {
        options: shown
            .choices
            .iter()
            .map(|badge| badge.shown().to_owned())
            .collect(),
        selected: shown
            .choices
            .iter()
            .position(|badge| badge.id == shown.current)
            .unwrap_or(0),
        ..default()
    };
    let made = spawn_dropdown_row(page, "Shown As", tip(WORDS, None), dropdown, WIDTH);
    let ids = shown.choices.iter().map(|badge| badge.id.clone()).collect();
    page.commands()
        .entity(made)
        .insert((ShownList(ids), UiId::new("settings.gameplay", "shown_as")));
}

/// A choice is asked of the server.
pub(in crate::system_menu) fn use_shown_role(
    mut chosen: MessageReader<DropdownChosen>,
    lists: Query<&ShownList>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &DropdownChosen(list, index) in chosen.read() {
        if let Some(id) = lists.get(list).ok().and_then(|list| list.0.get(index)) {
            to_server.write(ToServer(ClientMessage::ShowRoleAs(id.clone())));
        }
    }
}
