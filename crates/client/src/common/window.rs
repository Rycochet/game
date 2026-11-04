//! How often frames run.

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

use super::FOCUSED_FPS;

/// Whether frames keep their focused rate with the window in the background.
/// Off for players; `dev_tools` turns it on while the debug API or a
/// screenshot is running, so tools aren't held to a frame a second.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeepDrawing(pub bool);

/// How often frames run. Out of focus they are a second apart, unless
/// `keep_drawing` (while loading, which shouldn't wait for them, or with
/// `KeepDrawing` on).
pub fn frame_settings(keep_drawing: bool) -> WinitSettings {
    let focused = UpdateMode::reactive(Duration::from_secs_f64(1.0 / FOCUSED_FPS));
    WinitSettings {
        focused_mode: focused,
        unfocused_mode: if keep_drawing {
            focused
        } else {
            UpdateMode::reactive_low_power(Duration::from_secs(1))
        },
    }
}
