//! The menu itself: the warning button, the full-screen lists (the actions, and
//! a submenu above them), and the keys and presses that work them.

use crate::widgets::TextSize;

// Split into files (`cargo xtask lint` keeps files small).
mod columns;
mod input;
mod layout;
mod on_show;
mod parts;
mod show;
mod spawn;
mod state;

use columns::{columns_in, rows_in};
pub(super) use input::take_input;
use layout::{Slot, first_column, lay_out};
use on_show::{entries_on_show, headings_of, on_show, return_chooses};
use parts::Entry;
pub(super) use parts::{
    FilterBox, MenuClose, MenuEntry, MenuGrid, MenuRoot, PageButton, WarningButton,
};
pub(super) use show::show_menu;
pub(super) use spawn::{bind_menu, spawn_menu};
pub use state::DebugMenu;

/// How long a toggled entry stays on show before the menu closes: just past the
/// press's ripple (`widgets`, 0.35 s).
const TOGGLE_SECONDS: f32 = 0.4;

/// The layer of the actions, and of a submenu above them.
const ACTIONS: usize = 0;
const SUBMENU: usize = 1;
/// Each layer's form in the markup.
const FORMS: [&str; 2] = ["debug.menu", "debug.submenu"];

const MARGIN: f32 = 16.0;
const TITLE_ROW: f32 = 36.0;
const FILTER_HEIGHT: f32 = 38.0;
const ROW: f32 = 34.0;
const GAP: f32 = 4.0;
const COLUMN_GAP: f32 = GAP * 2.0;
const COLUMN_WIDTH: f32 = 300.0;
const FONT: TextSize = TextSize::Large;
/// The groups' headings.
const HEADING: TextSize = TextSize::Medium;

#[cfg(test)]
mod tests;
