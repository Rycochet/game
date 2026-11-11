//! The pages of Settings, under its tabs: Gameplay (`gameplay`), Controls
//! (`controls`), Interface
//! (its size, "Enable Touch" and "Always Show Scroll Bars") and Graphics (`graphics`). Interface choices go
//! straight into `Preferences`.

mod controls;
mod difficulty;
mod gameplay;
mod graphics;
mod keep;
mod rows;
mod shell;
mod shown_role;

pub(super) use controls::{TakenKeys, fill_controls_page, forget_taken, use_controls_page};
pub(super) use difficulty::use_difficulty;
pub(super) use gameplay::{fill_gameplay_page, use_gameplay_page};
pub(super) use graphics::{
    GraphicsForm, GraphicsShown, fill_graphics_page, use_graphics_options, use_graphics_page,
};
pub(super) use keep::{GraphicsTrial, KeepButton, run_trial};
pub(super) use shell::{bind_settings, show_interface_page, spawn_settings};
pub(super) use shown_role::{ShownAs, keep_shown_role, use_shown_role};

use bevy::prelude::*;

use crate::common::{InterfaceSize, Preferences};
use crate::widgets::{DropdownChosen, INTERFACE_PERCENTS, SwitchFlipped, TabChosen, TextSize};

const FONT: TextSize = TextSize::Large;
const ROW_GAP: f32 = 12.0;
const LABEL_GAP: f32 = 16.0;
const ROW_HEIGHT: f32 = 36.0;

const TOUCH: &str = "Keeps the interface at least its own size, for fingers.";
const TOUCH_ONLY: &str = "Always on, as this device only has a touch screen.";
const SCROLL_BARS: &str = "Shows the bar of everything that can scroll, even when all of it fits.";

/// The tabs' row.
#[derive(Component)]
pub(super) struct PageTabs;

/// A page, by its place among the tabs.
#[derive(Component)]
pub(super) struct Page(usize);

#[derive(Component)]
pub(super) struct SizeDropdown;

#[derive(Component)]
pub(super) struct TouchSwitch;

/// The "Always Show Scroll Bars" switch.
#[derive(Component)]
pub(super) struct ScrollBarsSwitch;

/// Which entry of the dropdown a size is.
fn size_index(size: InterfaceSize) -> usize {
    match size {
        InterfaceSize::Auto => 0,
        InterfaceSize::Percent(percent) => {
            INTERFACE_PERCENTS
                .iter()
                .position(|&offered| offered >= percent)
                .unwrap_or(INTERFACE_PERCENTS.len() - 1)
                + 1
        }
    }
}

/// The size an entry of the dropdown is.
fn size_at(index: usize) -> InterfaceSize {
    index
        .checked_sub(1)
        .and_then(|index| INTERFACE_PERCENTS.get(index))
        .map_or(InterfaceSize::Auto, |&percent| {
            InterfaceSize::Percent(percent)
        })
}

fn size_options() -> Vec<String> {
    std::iter::once("Auto".to_string())
        .chain(
            INTERFACE_PERCENTS
                .iter()
                .map(|percent| format!("{percent}%")),
        )
        .collect()
}

/// Shows the page whose tab was chosen, and puts choices into the preferences.
#[allow(clippy::too_many_arguments)]
pub(super) fn use_pages(
    mut tabs: MessageReader<TabChosen>,
    mut chosen: MessageReader<DropdownChosen>,
    mut flipped: MessageReader<SwitchFlipped>,
    page_tabs: Query<(), With<PageTabs>>,
    mut pages: Query<(&Page, &mut Node)>,
    size_dropdowns: Query<(), With<SizeDropdown>>,
    touch_switches: Query<(), With<TouchSwitch>>,
    bar_switches: Query<(), With<ScrollBarsSwitch>>,
    mut preferences: ResMut<Preferences>,
) {
    for &TabChosen(row, index) in tabs.read() {
        if page_tabs.contains(row) {
            for (page, mut node) in &mut pages {
                let display = if page.0 == index {
                    Display::Flex
                } else {
                    Display::None
                };
                if node.display != display {
                    node.display = display;
                }
            }
        }
    }
    for &DropdownChosen(dropdown, index) in chosen.read() {
        if size_dropdowns.contains(dropdown) {
            let size = size_at(index);
            if preferences.interface_size != size {
                preferences.interface_size = size;
            }
        }
    }
    for &SwitchFlipped(switch, on) in flipped.read() {
        if touch_switches.contains(switch) {
            preferences.touch = on;
        }
        if bar_switches.contains(switch) {
            preferences.always_show_scroll_bars = on;
        }
    }
}

#[cfg(test)]
mod tests;
