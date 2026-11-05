//! A button showing a choice, which opens a list to choose from; with search, the
//! list also has a text box that takes the keyboard and narrows it down. While a
//! list is open it has all the input. It scrolls by the wheel, a finger, or the
//! arrow keys, a window of rows at a time, as nodes outside a clip still take
//! presses here.

use super::{Stroke, TextSize};

// Split into files (`cargo xtask lint` keeps files small).
mod list;
mod parts;
mod rows;
mod run;
mod show;

use list::spawn_list;
pub use parts::{Dropdown, DropdownChosen, spawn_dropdown, spawn_icon_dropdown};
pub(super) use parts::{List, ListBacking, ListOption, ListRows, OpenList};
use parts::{drawn_words, label_of};
use rows::{list_corner, on_show, window_start};
pub(super) use run::run_dropdowns;
pub(super) use show::show_dropdowns;

/// Above anything a dropdown can be on.
const LIST_LAYER: i32 = 100;
/// How many options show at once.
const ROWS: usize = 10;
const ROW: f32 = 32.0;
/// A picture beside an option in the list, across.
const ROW_ICON: f32 = 20.0;
const GAP: f32 = 4.0;
const FONT: TextSize = TextSize::Medium;
const SEARCH_HEIGHT: f32 = 32.0;
/// How far a finger drags to move by a row.
const DRAG_PER_ROW: f32 = 32.0;
const LEAST_WIDTH: f32 = 160.0;
/// The arrow on the button: how far across, from the button's right, and its strokes.
const ARROW_WIDE: f32 = 12.0;
const ARROW_RIGHT: f32 = 4.0;
const ARROW_STROKES: [Stroke; 2] = [(8.0, -1.0, 4.0, 45.0), (8.0, 4.0, 4.0, -45.0)];

#[cfg(test)]
mod tests;
