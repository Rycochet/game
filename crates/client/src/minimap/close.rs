//! The full map's close button: its place (`full_map/close`) is `Pressable`,
//! so a click or tap on the X, or `client.ui.press`, is a `ButtonPressed` for
//! it, taken by the widgets before `take_input` runs. Presses just outside it
//! (`CLOSE_REACH`) are still `take_input`'s.

use bevy::prelude::*;

use super::FullMap;
use crate::widgets::{ButtonPressed, UiId};

/// Closes the full map when its close button is pressed.
pub(super) fn close_on_button(
    mut pressed: MessageReader<ButtonPressed>,
    ids: Query<&UiId>,
    mut full: ResMut<FullMap>,
) {
    let close = UiId::new("full_map", "close");
    let on_close = |said: &ButtonPressed| ids.get(said.0).is_ok_and(|id| *id == close);
    if pressed.read().filter(|said| on_close(said)).count() > 0 && full.open {
        full.set_open(false);
    }
}

#[cfg(test)]
mod tests;
