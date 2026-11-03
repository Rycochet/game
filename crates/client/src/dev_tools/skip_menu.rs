//! `HOW_DEBUG_SKIP_MENU`: past the startup menu straight into the game, as if
//! Play was pressed. On by default whenever another variable here sets up how
//! the game starts, so they keep working without a press.

use bevy::prelude::*;

use crate::common::Session;

const VARIABLE: &str = "HOW_DEBUG_SKIP_MENU";

/// `HOW_DEBUG_` variables that don't set up the game, so don't skip the menu by
/// themselves (the debug API can press Play).
const NOT_STARTING: &[&str] = &[
    VARIABLE,
    "HOW_DEBUG_API",
    "HOW_DEBUG_GUEST",
    "HOW_DEBUG_KEEP_DRAWING",
    "HOW_DEBUG_LETS_ENCRYPT_STAGING",
    "HOW_DEBUG_TOOLS",
];

pub(super) fn add(app: &mut App) {
    let names = std::env::vars().map(|(name, _)| name);
    if server::option_on(VARIABLE, sets_up_start(names)) {
        app.insert_resource(Session::Playing);
    }
}

/// Whether any of these variables sets up how the game starts.
fn sets_up_start(mut names: impl Iterator<Item = String>) -> bool {
    names.any(|name| name.starts_with("HOW_DEBUG_") && !NOT_STARTING.contains(&name.as_str()))
}

#[cfg(test)]
mod tests;
