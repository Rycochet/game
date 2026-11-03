//! The menu's presses and keys.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use super::{Action, SystemMenu, show_ended_page};
use crate::common::{GameView, Session, TouchGesture};
use crate::debug_menu::DebugMenu;
use crate::minimap::FullMap;
use crate::widgets::ButtonPressed;

/// After the text boxes and dropdowns have used what they want, and before the
/// debug menu, the full map and the game. While open, the menu has all input.
/// At the startup menu it is always open: nothing closes it but Play.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    debug_menu: Res<DebugMenu>,
    full_map: Res<FullMap>,
    mut menu: ResMut<SystemMenu>,
    mut session: ResMut<Session>,
    mut exit: MessageWriter<AppExit>,
) {
    let at_start = !session.is_playing();
    if at_start && !menu.is_open() {
        *menu = SystemMenu::Menu;
    }
    let mut action = pressed
        .read()
        .filter_map(|press| actions.get(press.0).ok())
        .last()
        .copied();
    let escape = keys.just_pressed(KeyCode::Escape);
    if menu.is_open() {
        if escape {
            action = Some(Action::Back);
        }
        keys.reset_all();
        mouse.reset(MouseButton::Right);
        scroll.delta = Vec2::ZERO;
        *gesture = TouchGesture::default();
    } else if escape && !debug_menu.open && !full_map.open {
        // (Those two close on it instead, seeing it after this.)
        keys.clear_just_pressed(KeyCode::Escape);
        action = Some(Action::Open);
    }
    let Some(action) = action else {
        return;
    };
    match (action, *menu) {
        (Action::ReallyQuit, SystemMenu::Confirming) => {
            exit.write(AppExit::Success);
            show_ended_page();
        }
        (Action::Play, SystemMenu::Menu) if at_start => {
            *session = Session::Playing;
            *menu = SystemMenu::Closed;
        }
        (Action::LogOut, SystemMenu::Menu) if !at_start => {
            // The menu stays, as the startup menu.
            *session = Session::AtStart;
        }
        // Nothing to go back to from the startup menu.
        (Action::Back, SystemMenu::Menu) if at_start => {}
        _ => {
            menu.set_if_neq(menu.after(action));
        }
    }
}

/// While the menu takes input: at the startup menu, or once there is a world.
pub(super) fn takes_input(session: Res<Session>, view: Res<State<GameView>>) -> bool {
    !session.is_playing() || *view.get() != GameView::Connecting
}
