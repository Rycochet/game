//! `HOW_DEBUG_SKIP_MENU`: past the startup menu straight into the game, as if
//! Play was pressed. On by default whenever another variable here sets up how
//! the game starts, so they keep working without a press. Its mirror,
//! `HOW_DEBUG_SKIP_CREATION`: a new character is a plain one the server makes,
//! with no creation screen; on by default whenever the menu is skipped.

use bevy::prelude::*;

use crate::common::{MakesCharacters, Session};

const VARIABLE: &str = "HOW_DEBUG_SKIP_MENU";
const CREATION: &str = "HOW_DEBUG_SKIP_CREATION";

/// `HOW_DEBUG_` variables that don't set up the game, so don't skip the menu by
/// themselves (the debug API can press Play).
const NOT_STARTING: &[&str] = &[
    VARIABLE,
    CREATION,
    "HOW_DEBUG_API",
    "HOW_DEBUG_GUEST",
    "HOW_DEBUG_KEEP_DRAWING",
    "HOW_DEBUG_LETS_ENCRYPT_STAGING",
    "HOW_DEBUG_TITLE",
    "HOW_DEBUG_TOOLS",
    "HOW_DEBUG_WINDOW",
];

pub(super) fn add(app: &mut App) {
    let names = std::env::vars().map(|(name, _)| name);
    let skips_menu = server::option_on(VARIABLE, sets_up_start(names));
    if skips_menu {
        app.insert_resource(Session::Playing);
    }
    if server::option_on(CREATION, skips_menu) {
        app.insert_resource(MakesCharacters(false));
    }
}

/// Whether any of these variables sets up how the game starts.
fn sets_up_start(mut names: impl Iterator<Item = String>) -> bool {
    names.any(|name| name.starts_with("HOW_DEBUG_") && !NOT_STARTING.contains(&name.as_str()))
}

#[cfg(test)]
mod tests;
