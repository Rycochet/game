//! How often frames run.

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

use super::{Background, VideoPrefs};

/// Whether frames keep their focused rate with the window in the background.
/// Off for players unless they choose it (`Background::KeepDrawing`);
/// `dev_tools` turns it on while the debug API or a screenshot is running, so
/// tools aren't held to a frame a second.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeepDrawing(pub bool);

/// How often frames run: in focus at most the chosen frame rate (input still
/// wakes the app at once). Out of focus they are a second apart, unless
/// `keep_drawing` (while loading, which shouldn't wait for them, or with
/// `KeepDrawing` on) or the player chose to keep drawing.
pub fn frame_settings(keep_drawing: bool, video: &VideoPrefs) -> WinitSettings {
    let focused = match video.frame_rate.per_second() {
        Some(fps) => UpdateMode::reactive(Duration::from_secs_f64(1.0 / fps)),
        None => UpdateMode::Continuous,
    };
    WinitSettings {
        focused_mode: focused,
        unfocused_mode: if keep_drawing || video.background == Background::KeepDrawing {
            focused
        } else {
            UpdateMode::reactive_low_power(Duration::from_secs(1))
        },
    }
}

/// VSync as chosen (native: a browser shows frames when it likes).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn apply_vsync(
    preferences: Res<super::Preferences>,
    mut window: Single<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    use bevy::window::PresentMode;
    if !preferences.is_changed() {
        return;
    }
    let mode = if preferences.video.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    if window.present_mode != mode {
        window.present_mode = mode;
    }
}
