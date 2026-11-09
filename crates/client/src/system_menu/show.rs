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

/// Play at the startup menu, Continue while playing, Logout while playing on
/// another computer's server or at the startup menu with an account kept for
/// Play's server (`AccountEmail`), and Connect... only usable at the startup
/// menu.
pub(super) fn show_buttons(
    session: Res<Session>,
    clock: Res<ServerClock>,
    account: Res<crate::common::AccountEmail>,
    playable: Res<crate::common::PlayableHere>,
    role: Res<crate::common::AccountRole>,
    mut buttons: Query<(&MenuButton, &mut Node, &mut ButtonState)>,
) {
    for (button, mut node, mut state) in &mut buttons {
        let shown = match button {
            MenuButton::Play => {
                // Only with a character to go back to there.
                state.set_if_neq(if playable.0 {
                    ButtonState::Normal
                } else {
                    ButtonState::Disabled
                });
                !session.is_playing()
            }
            MenuButton::Characters => !session.is_playing(),
            MenuButton::Continue => session.is_playing(),
            MenuButton::Accounts => session.is_playing() && role.has_accounts_panel(),
            MenuButton::LogOut => match session.is_playing() {
                true => !clock.local_server,
                false => account.0.is_some(),
            },
            // Its looks are `online::show_online`'s.
            MenuButton::Online => true,
            MenuButton::Connect => {
                // Log out first to go to another server.
                state.set_if_neq(if session.is_playing() {
                    ButtonState::Disabled
                } else {
                    ButtonState::Normal
                });
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
