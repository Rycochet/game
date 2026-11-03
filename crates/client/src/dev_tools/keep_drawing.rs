//! Full frame rate in the background while tools are driving the game.

use bevy::prelude::*;
use std::env;

use crate::common::KeepDrawing;

/// Turns `KeepDrawing` on while the debug API (`HOW_DEBUG_API`) or a
/// screenshot (`HOW_DEBUG_SCREENSHOT`) is running, as the window is usually
/// in the background then; `HOW_DEBUG_KEEP_DRAWING=false` leaves it off.
pub(super) fn add(app: &mut App) {
    let api = cfg!(feature = "debug-api")
        && server::debug_api::port_from(env::var(server::debug_api::VARIABLE).ok().as_deref())
            .is_some();
    let screenshot = env::var("HOW_DEBUG_SCREENSHOT").is_ok();
    if keeps_drawing(
        api,
        screenshot,
        server::option_on("HOW_DEBUG_KEEP_DRAWING", true),
    ) {
        app.insert_resource(KeepDrawing(true));
    }
}

/// Whether tools are running and the variable hasn't turned it off.
pub(super) fn keeps_drawing(api: bool, screenshot: bool, allowed: bool) -> bool {
    (api || screenshot) && allowed
}

#[cfg(test)]
mod tests;
