//! Alt+Enter (native): between windowed and the full screen mode last used,
//! straight away, as that mode was kept once already. Not while typing, or
//! while the Video page is asking whether to keep what was applied.

use bevy::input::ButtonState;
use bevy::input::keyboard::{KeyboardFocusLost, KeyboardInput};
use bevy::prelude::*;

use super::SystemMenu;
use crate::common::{DisplayInUse, Preferences, RememberWindow};
use crate::widgets::TextFocus;

/// Before the widgets and the menu, which take the keys while open. Alt is
/// followed by its own presses, as the open menu lets go of every key held.
#[allow(clippy::too_many_arguments)]
pub(super) fn toggle_full_screen(
    mut events: MessageReader<KeyboardInput>,
    mut focus_lost: MessageReader<KeyboardFocusLost>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    focus: Res<TextFocus>,
    menu: Res<SystemMenu>,
    remember: Res<RememberWindow>,
    mut in_use: ResMut<DisplayInUse>,
    mut preferences: ResMut<Preferences>,
    mut alt: Local<bool>,
) {
    // (Alt+Tab away leaves no letting go of Alt to see.)
    if focus_lost.read().count() > 0 {
        *alt = false;
    }
    let mut toggle = false;
    for event in events.read() {
        let down = event.state == ButtonState::Pressed;
        match event.key_code {
            KeyCode::AltLeft | KeyCode::AltRight => *alt = down,
            KeyCode::Enter | KeyCode::NumpadEnter if down && *alt && !event.repeat => {
                keys.clear_just_pressed(event.key_code);
                toggle = true;
            }
            _ => {}
        }
    }
    if !toggle || focus.0.is_some() || *menu == SystemMenu::KeepVideo {
        return;
    }
    in_use.0 = in_use.0.toggled(preferences.window.last_full_screen);
    if remember.0 {
        preferences.window.keep(in_use.0);
    }
}
