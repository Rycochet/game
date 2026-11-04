//! The menu itself: the warning button, the full-screen lists (the actions, and
//! a submenu above them), and the keys and presses that work them.

use crate::widgets::TextSize;

// Split into files (`cargo xtask lint` keeps files small).
mod columns;
mod input;
mod on_show;
mod parts;
mod show;
mod spawn;
mod state;

use columns::{columns_in, columns_of, first_column, rows_in};
pub(super) use input::take_input;
use on_show::{entries_on_show, on_show, return_chooses};
use parts::Entry;
pub(super) use parts::{
    FilterBox, MenuClose, MenuEntry, MenuGrid, MenuRoot, PageButton, WarningButton,
};
pub(super) use show::show_menu;
pub(super) use spawn::spawn_menu;
pub use state::DebugMenu;

/// How long a toggled entry stays on show before the menu closes: just past the
/// press's ripple (`widgets`, 0.35 s).
const TOGGLE_SECONDS: f32 = 0.4;

/// The layer of the actions, and of a submenu above them.
const ACTIONS: usize = 0;
const SUBMENU: usize = 1;

const WARNING_BUTTON: f32 = 34.0;
const CORNER_GAP: f32 = 8.0;

const MARGIN: f32 = 16.0;
const TITLE_ROW: f32 = 36.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;
const FILTER_HEIGHT: f32 = 38.0;
const ROW: f32 = 34.0;
const GAP: f32 = 4.0;
const COLUMN_GAP: f32 = GAP * 2.0;
const COLUMN_WIDTH: f32 = 300.0;
const PAGE_BUTTON: f32 = 34.0;
const FONT: TextSize = TextSize::Large;

#[cfg(test)]
mod tests;
