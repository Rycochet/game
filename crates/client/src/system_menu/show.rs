//! Showing what is open, the web's ended page, and asking for a pause.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use shared::protocol::ClientMessage;

use super::{GearButton, Layer, MenuButton, PausedText, SystemMenu};
use crate::common::{GameView, Preferences, ServerClock, Session};
use crate::connection::ToServer;
use crate::widgets::ButtonState;

/// Native: nothing, the window just goes. In a browser the page stays, so it
/// shows its own screen in `index.html`, with a button to load the game again.
pub(super) fn show_ended_page() {
    #[cfg(target_arch = "wasm32")]
    {
        let screen = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("ended"));
        if let Some(screen) = screen {
            let _ = screen.remove_attribute("hidden");
        }
    }
}

/// Shows what is open.
#[allow(clippy::type_complexity)]
pub(super) fn show_menu(
    menu: Res<SystemMenu>,
    view: Res<State<GameView>>,
    clock: Res<ServerClock>,
    mut layers: Query<(&Layer, &mut Visibility)>,
    mut gear: Single<&mut Visibility, (With<GearButton>, Without<Layer>)>,
    mut paused: Single<&mut Node, With<PausedText>>,
) {
    let shown_if = |shown: bool| {
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    for (layer, mut visibility) in &mut layers {
        visibility.set_if_neq(shown_if(layer.0.contains(&menu)));
    }
    gear.set_if_neq(shown_if(*view.get() != GameView::Connecting));
    let display = if clock.paused {
        Display::Flex
    } else {
        Display::None
    };
    if paused.display != display {
        paused.display = display;
    }
}

/// Continue always, but at the startup menu only with this computer's own
/// server there (`HomeServer`), and enabled only with a character of this
/// player's there; Characters at the startup menu, with that server there;
/// Logout while playing on another computer's server, or at the startup menu
/// with an account kept for this computer's own (`AccountEmail`: a browser's
/// page's server); Online for this computer's own server (native) or, while
/// playing on another, for staff who may lock it down; Connect... only usable
/// at the startup menu.
pub(super) fn show_buttons(
    session: Res<Session>,
    clock: Res<ServerClock>,
    account: Res<crate::common::AccountEmail>,
    playable: Res<crate::common::PlayableHere>,
    home: Res<crate::common::HomeServer>,
    lockdown: Res<crate::common::Lockdown>,
    mut buttons: Query<(&MenuButton, &mut Node, &mut ButtonState)>,
) {
    let playing = session.is_playing();
    let home = *home == crate::common::HomeServer::Found;
    let enabled = |on: bool| {
        if on {
            ButtonState::Normal
        } else {
            ButtonState::Disabled
        }
    };
    for (button, mut node, mut state) in &mut buttons {
        let shown = match button {
            MenuButton::Continue => {
                // At the startup menu, only with a character to go back to.
                state.set_if_neq(enabled(playing || playable.0));
                playing || home
            }
            MenuButton::Characters => !playing && home,
            MenuButton::LogOut => match playing {
                true => !clock.local_server,
                false => account.0.is_some(),
            },
            // Its looks are `online::show_online`'s. A browser has no server
            // of its own; another computer's shows it only to its staff.
            MenuButton::Online => match (playing, clock.local_server) {
                (false, _) => !cfg!(target_arch = "wasm32"),
                (true, true) => true,
                (true, false) => lockdown.0.is_some(),
            },
            MenuButton::Connect => {
                // Log out first to go to another server.
                state.set_if_neq(enabled(!playing));
                true
            }
        };
        let display = if shown { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
}

/// While the menu is open, or the window is in the background with "Pause When
/// Not in Focus" on, asks a server that can pause to do so, and to carry on
/// once neither is so. The server's answer is what stops the client
/// (`connection`).
pub(super) fn ask_for_pause(
    menu: Res<SystemMenu>,
    clock: Res<ServerClock>,
    preferences: Res<Preferences>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut asked: Local<bool>,
    mut to_server: MessageWriter<ToServer>,
) {
    let away = preferences.pause_unfocused && !window.focused;
    let wanted = (menu.is_open() || away) && clock.can_pause;
    if wanted != *asked {
        *asked = wanted;
        to_server.write(ToServer(ClientMessage::SetPaused(wanted)));
    }
}
