//! How the window is shown (windowed or full screen, at what resolution) and
//! where it was, kept in `Preferences::window` so it opens the same next time.

use bevy::prelude::*;
use bevy::window::VideoMode;
use serde::{Deserialize, Serialize};

/// Windowed, or filling the screen.
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplayMode {
    #[default]
    Windowed,
    /// A window without borders the size of the screen, at the screen's own
    /// resolution.
    Borderless,
    /// The screen taken over at a resolution of its own ("exclusive").
    FullScreen,
}

impl DisplayMode {
    pub const ALL: [Self; 3] = [Self::Windowed, Self::Borderless, Self::FullScreen];

    pub fn name(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Borderless => "Borderless full screen",
            Self::FullScreen => "Full screen",
        }
    }
}

/// One of a screen's video modes, as saved.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Resolution {
    /// Physical pixels.
    pub width: u32,
    pub height: u32,
    /// 0: whichever is fastest at that size.
    pub refresh_millihertz: u32,
}

impl Resolution {
    pub fn label(self) -> String {
        let hertz = (self.refresh_millihertz + 500) / 1000;
        format!("{} × {}, {hertz} Hz", self.width, self.height)
    }
}

/// How the window is shown.
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(default)]
pub struct DisplayChoice {
    pub mode: DisplayMode,
    /// For `FullScreen`; `None` is the screen's own (the desktop's).
    pub resolution: Option<Resolution>,
}

impl DisplayChoice {
    /// Alt+Enter: windowed goes to the full screen mode last used, and full
    /// screen to windowed.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn toggled(self, last_full_screen: DisplayMode) -> Self {
        let mode = match self.mode {
            DisplayMode::Windowed => last_full_screen,
            _ => DisplayMode::Windowed,
        };
        Self { mode, ..self }
    }
}

/// Where the window was, while windowed and not maximised.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct WindowPlace {
    /// The outside's top left corner, in physical pixels on the desktop.
    pub x: i32,
    pub y: i32,
    /// The inside, in logical pixels.
    pub width: f32,
    pub height: f32,
}

/// The window, as last left.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct WindowPrefs {
    pub display: DisplayChoice,
    /// Which full screen mode Alt+Enter goes to.
    pub last_full_screen: DisplayMode,
    /// `None` until it has been moved or resized: the system's choice.
    pub place: Option<WindowPlace>,
    /// Maximised while windowed; `place` is then the size it goes back to.
    pub maximised: bool,
    /// The name of the monitor it was on.
    pub monitor: Option<String>,
}

impl Default for WindowPrefs {
    fn default() -> Self {
        Self {
            display: DisplayChoice::default(),
            last_full_screen: DisplayMode::Borderless,
            place: None,
            // Filling the screen at first, which Auto interface size goes by.
            maximised: true,
            monitor: None,
        }
    }
}

impl WindowPrefs {
    /// Keeps this way of showing the window.
    pub fn keep(&mut self, choice: DisplayChoice) {
        self.display = choice;
        if choice.mode != DisplayMode::Windowed {
            self.last_full_screen = choice.mode;
        }
    }
}

/// How the window is shown now: change it to change the window. From
/// `Preferences::window` at start, unless `HOW_DEBUG_WINDOW` says otherwise.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayInUse(pub DisplayChoice);

/// Whether the window's place and how it is shown are saved: not while
/// `dev_tools` has set them (`HOW_DEBUG_WINDOW`, the debug API), so a test run
/// never changes the player's own.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RememberWindow(pub bool);

impl Default for RememberWindow {
    fn default() -> Self {
        Self(true)
    }
}

/// A screen's resolutions, each at its fastest refresh, largest first.
pub fn resolutions(modes: &[VideoMode]) -> Vec<Resolution> {
    let mut found: Vec<Resolution> = Vec::new();
    for mode in modes {
        let (width, height) = (mode.physical_size.x, mode.physical_size.y);
        let refresh = mode.refresh_rate_millihertz;
        match found
            .iter_mut()
            .find(|seen| (seen.width, seen.height) == (width, height))
        {
            Some(seen) => seen.refresh_millihertz = seen.refresh_millihertz.max(refresh),
            None => found.push(Resolution {
                width,
                height,
                refresh_millihertz: refresh,
            }),
        }
    }
    found.sort_by_key(|seen| std::cmp::Reverse((seen.width * seen.height, seen.width)));
    found
}

/// The screen's video mode nearest to `wanted`: the same size and refresh,
/// else the same size at its fastest, else the nearest size.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(super) fn closest_mode(modes: &[VideoMode], wanted: Resolution) -> Option<VideoMode> {
    let size_off = |mode: &VideoMode| {
        mode.physical_size.x.abs_diff(wanted.width) + mode.physical_size.y.abs_diff(wanted.height)
    };
    let refresh_off = |mode: &VideoMode| match wanted.refresh_millihertz {
        0 => u32::MAX - mode.refresh_rate_millihertz,
        refresh => mode.refresh_rate_millihertz.abs_diff(refresh),
    };
    modes
        .iter()
        .min_by_key(|mode| (size_off(mode), refresh_off(mode), u16::MAX - mode.bit_depth))
        .copied()
}

#[cfg(test)]
mod tests;
