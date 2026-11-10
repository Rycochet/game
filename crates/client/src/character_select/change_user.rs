//! Change User, at the bottom left: only while logged in to an account on the
//! server (`common::AccountEmail`; never as a guest or in single player). It
//! forgets that server's login and connects again, so the server asks how to
//! log in (`common::LoginChoice::ChangeUser`, `connection`).

use bevy::prelude::*;

use super::screen::{Button, FORM};
use crate::common::AccountEmail;
use crate::widgets::{ButtonState, ButtonStyle, Pressable, TextSize, UiId, spawn_button};

/// The row holding it, shown only while it is any use.
#[derive(Component)]
pub(super) struct ChangeUserRow;

/// Under the grid of characters.
pub(super) fn spawn_change_user(left: &mut ChildSpawnerCommands) {
    left.spawn((
        ChangeUserRow,
        Node {
            display: Display::None,
            ..default()
        },
    ))
    .with_children(|row| {
        let node = Node {
            padding: UiRect::axes(px(16), px(8)),
            ..default()
        };
        spawn_button(
            row,
            ButtonState::Normal,
            "Change User",
            TextSize::Large,
            node,
        )
        .insert((
            Button::ChangeUser,
            ButtonStyle::Plain,
            Pressable,
            UiId::new(FORM, "change_user"),
        ));
    });
}

/// Shows it while logged in to an account.
pub(super) fn show_change_user(
    email: Res<AccountEmail>,
    mut row: Single<&mut Node, With<ChangeUserRow>>,
) {
    let display = if email.0.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    if row.display != display {
        row.display = display;
    }
}
