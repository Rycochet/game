//! The window as the maps' nodes see it.

use bevy::prelude::*;

use crate::common::interface_area;

/// The window as the maps' nodes see it: its size in the interface's pixels,
/// which `UiScale` makes bigger or smaller, so the maps keep their share of the
/// window whatever size the interface is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Room {
    size: Vec2,
    /// The screen's pixels to an interface pixel.
    scale: f32,
    /// The screen's pixels to a logical window pixel.
    screen: f32,
}

impl Room {
    pub(crate) fn of(window: &Window, ui_scale: &UiScale) -> Self {
        Self {
            size: interface_area(window, ui_scale),
            scale: window.scale_factor() * ui_scale.0,
            screen: window.scale_factor(),
        }
    }

    pub(crate) fn width(&self) -> f32 {
        self.size.x
    }

    pub(crate) fn height(&self) -> f32 {
        self.size.y
    }

    /// The screen's pixels to an interface pixel.
    pub(crate) fn scale_factor(&self) -> f32 {
        self.scale
    }

    /// The screen's pixels to a logical window pixel: for what is drawn in the
    /// maps' pictures, which follows the screen as the maps do.
    pub(crate) fn screen_factor(&self) -> f32 {
        self.screen
    }
}
