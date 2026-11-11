//! The Graphics page's Window section: Display mode and full screen's
//! Resolution, which change nothing until Apply. In a browser they are
//! shown, disabled, as the browser looks after the window.

use bevy::prelude::*;
use bevy::window::Monitor;

use super::super::rows::tip_words;
use crate::common::{DisplayChoice, Resolution, resolutions};
use crate::dom::Dom;

const DISPLAY_MODE: &str = "How the game fills the screen: in a window, in a window \
    without edges covering the screen, or full screen.";
const RESOLUTION: &str = "The screen's resolution in full screen.";
const BROWSER_WINDOW: &str = "The browser looks after the window.";
/// Whether this build can change the window.
pub(super) const CAN_CHANGE: bool = cfg!(not(target_arch = "wasm32"));

/// Which entry is this resolution: the same one, else the same size, else the
/// screen's own.
pub(super) fn resolution_index(
    entries: &[Option<Resolution>],
    wanted: Option<Resolution>,
) -> usize {
    let size = |entry: &Option<Resolution>| entry.map(|seen| (seen.width, seen.height));
    entries
        .iter()
        .position(|&entry| entry == wanted)
        .or_else(|| {
            entries
                .iter()
                .position(|entry| size(entry) == size(&wanted))
        })
        .unwrap_or(0)
}

/// The resolution dropdown's entries for this monitor, with their labels.
pub(super) fn resolution_entries(
    monitor: Option<&Monitor>,
) -> (Vec<Option<Resolution>>, Vec<String>) {
    let Some(monitor) = monitor else {
        return (vec![None], vec!["Desktop".into()]);
    };
    let (width, height) = (monitor.physical_width, monitor.physical_height);
    let found = resolutions(&monitor.video_modes);
    let entries = std::iter::once(None).chain(found.iter().copied().map(Some));
    let labels = std::iter::once(format!("Desktop ({width} × {height})"))
        .chain(found.iter().map(|resolution| resolution.label()));
    (entries.collect(), labels.collect())
}

/// The rows' tips, and the note under them in a browser: these never change
/// while the game runs.
pub(super) fn show_fixed(dom: &mut Dom) {
    let why_not = (!CAN_CHANGE).then_some(BROWSER_WINDOW);
    let tip = tip_words(DISPLAY_MODE, why_not);
    dom.set("settings.graphics/display_mode_row", "title", &tip);
    let tip = tip_words(RESOLUTION, Some(why_not.unwrap_or("Only for Full screen.")));
    dom.set("settings.graphics/resolution_row", "title", &tip);
    dom.set_flag("settings.graphics/browser_note", "hidden", CAN_CHANGE);
}

/// Shows how the window is, and the resolutions its monitor has. Each
/// resolution's value is its place in the list, as labels may be the same.
pub(super) fn show_window(
    dom: &mut Dom,
    in_use: DisplayChoice,
    entries: &[Option<Resolution>],
    labels: Vec<String>,
) {
    dom.set(
        "settings.graphics/display_mode",
        "value",
        in_use.mode.name(),
    );
    let numbered = labels.into_iter().enumerate();
    let options = numbered.map(|(at, label)| (at.to_string(), label));
    dom.set_options("settings.graphics/resolution", options);
    let chosen = resolution_index(entries, in_use.resolution);
    dom.set("settings.graphics/resolution", "value", &chosen.to_string());
}

/// What can be used: Display mode, Resolution (only for full screen) and
/// Apply (only with something to apply), in that order.
pub(super) fn show_usable(dom: &mut Dom, usable: [bool; 3]) {
    let ids = ["display_mode", "resolution", "apply"];
    for (id, usable) in ids.into_iter().zip(usable) {
        dom.set_flag(&format!("settings.graphics/{id}"), "disabled", !usable);
    }
}

#[cfg(test)]
mod tests;
