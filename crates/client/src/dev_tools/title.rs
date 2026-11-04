//! `HOW_DEBUG_TITLE`: text put before the window's title, with a space, so a
//! copy started for development (`Dev: Hall of Worlds`) can be told apart from
//! one being played. Unset or blank, the title stays as it is.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const VARIABLE: &str = "HOW_DEBUG_TITLE";

pub(super) fn add(app: &mut App) {
    let Ok(prefix) = std::env::var(VARIABLE) else {
        return;
    };
    let world = app.world_mut();
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    if let Ok(mut window) = windows.single_mut(world) {
        window.title = prefixed(&prefix, &window.title);
    }
}

/// `title` with `prefix` and a space before it, or as it is if `prefix` is blank.
fn prefixed(prefix: &str, title: &str) -> String {
    let prefix = prefix.trim();
    if prefix.is_empty() {
        title.to_string()
    } else {
        format!("{prefix} {title}")
    }
}

#[cfg(test)]
mod tests;
