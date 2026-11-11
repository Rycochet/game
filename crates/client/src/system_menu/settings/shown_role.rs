//! Settings' Gameplay page: "Shown As", the role others see the character as,
//! for accounts that may be shown as another role (staff playing as a plain
//! player, say). Asked of the server (`RoleAsk::ShowAs`), which says what it
//! now is (`RoleAnswer::Shown`). Here rather than on the Character page, which players
//! share pictures of.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, RoleAnswer, RoleAsk, ServerMessage};
use shared::role::{RoleId, ShownRole};

use crate::connection::{FromServer, ToServer};
use crate::dom::Dom;
use crate::widgets::DropdownChosen;

const WORDS: &str = "The role others see this character as. Staff are shown with a mark before \
     their name; shown as Player, there is none.";

/// The character's shown role and what else it may be, as the server last said.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::system_menu) struct ShownAs(pub Option<ShownRole>);

/// The dropdown, with the role each option is.
#[derive(Component, Default)]
pub(in crate::system_menu) struct ShownList(Vec<RoleId>);

/// Keeps what the server says.
pub(in crate::system_menu) fn keep_shown_role(
    mut from_server: MessageReader<FromServer>,
    mut shown: ResMut<ShownAs>,
) {
    for FromServer(message) in from_server.read() {
        let now = match message {
            ServerMessage::Welcome { shown, .. }
            | ServerMessage::Role(RoleAnswer::Shown(shown) | RoleAnswer::Changed { shown, .. }) => {
                shown
            }
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

/// Shows the row with these roles to choose from, or hides it with none.
/// Roles may share a name, so each option's value is its place in the list.
pub(in crate::system_menu) fn show_shown_row(
    dom: &mut Dom,
    list: &mut ShownList,
    shown: Option<&ShownRole>,
) {
    const ID: &str = "settings.gameplay/shown_as";
    dom.set_flag("settings.gameplay/shown_as_row", "hidden", shown.is_none());
    let Some(shown) = shown else {
        return;
    };
    let names = shown.choices.iter().map(|badge| badge.shown().to_owned());
    dom.set_options(
        ID,
        names.enumerate().map(|(at, name)| (at.to_string(), name)),
    );
    let current = shown
        .choices
        .iter()
        .position(|badge| badge.id == shown.current);
    dom.set(ID, "value", &current.unwrap_or(0).to_string());
    dom.set("settings.gameplay/shown_as_row", "title", WORDS);
    list.0 = shown.choices.iter().map(|badge| badge.id.clone()).collect();
}

/// A choice is asked of the server.
pub(in crate::system_menu) fn use_shown_role(
    mut chosen: MessageReader<DropdownChosen>,
    lists: Query<&ShownList>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &DropdownChosen(list, index) in chosen.read() {
        if let Some(id) = lists.get(list).ok().and_then(|list| list.0.get(index)) {
            to_server.write(ToServer(ClientMessage::Role(RoleAsk::ShowAs(id.clone()))));
        }
    }
}
