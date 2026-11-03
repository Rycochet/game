//! Opening the window maximised, and how often frames run.

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

use super::FOCUSED_FPS;

/// Natively the window opens filling the screen, which Auto sizes the interface by.
/// (A browser's canvas fills the page anyway.)
pub(super) fn open_maximised(mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    if cfg!(not(target_arch = "wasm32")) {
        for mut window in &mut windows {
            window.set_maximized(true);
        }
    }
}

/// How often frames run. Out of focus they are a second apart, except while
/// loading, which shouldn't wait for them.
pub fn frame_settings(loading: bool) -> WinitSettings {
    let focused = UpdateMode::reactive(Duration::from_secs_f64(1.0 / FOCUSED_FPS));
    WinitSettings {
        focused_mode: focused,
        unfocused_mode: if loading {
            focused
        } else {
            UpdateMode::reactive_low_power(Duration::from_secs(1))
        },
    }
}
