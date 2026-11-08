//! Logging in to an account: the screen asking how (a provider, or as a guest)
//! and saying to finish in the browser, and the account's email address in the
//! corner of the startup menu and while making a character. `connection` does
//! the logging in; this only shows `common::LoggingIn` and sends
//! `common::LoginChoice`. See `docs/context/client/login.md`.

use bevy::prelude::*;

mod corner;
mod screen;

pub struct LoginPlugin;

impl Plugin for LoginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (screen::spawn_screen, corner::spawn_corner))
            .add_systems(
                Update,
                (
                    screen::show_screen,
                    corner::show_corner,
                    screen::press_buttons,
                ),
            );
    }
}
