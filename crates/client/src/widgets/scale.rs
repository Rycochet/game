//! How big the interface is drawn: Bevy's `UiScale`, from the player's
//! `InterfaceSize` and "Enable touch". The minimap undoes it for itself.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::common::{InterfaceSize, Preferences};

/// The window (logical pixels) in which Auto is 100%: the commonest screen.
const AUTO_BASE: Vec2 = Vec2::new(1920.0, 1080.0);
/// The smallest and largest the interface is drawn.
const SMALLEST: f32 = 0.5;
const LARGEST: f32 = 3.0;
/// With touch on, never smaller than this, so fingers can hit things.
const TOUCH_SMALLEST: f32 = 1.0;

/// The sizes offered in Settings, in percent, after Auto.
pub const INTERFACE_PERCENTS: [u16; 11] = [50, 75, 100, 125, 150, 175, 200, 225, 250, 275, 300];

/// Whether the device has only a touch screen (no mouse or trackpad), so touch
/// is on whatever the setting says. Only browsers say; natively it is false.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchOnly(pub bool);

impl TouchOnly {
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let fine_pointer = web_sys::window()
                .and_then(|window| window.match_media("(any-pointer: fine)").ok().flatten())
                .map(|query| query.matches());
            // A browser that can't say has a mouse, as most do.
            Self(fine_pointer == Some(false))
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self(false)
    }
}

/// How much bigger than its own size the interface is drawn, in a window this
/// size (logical pixels).
pub fn interface_scale(size: InterfaceSize, touch: bool, window: Vec2) -> f32 {
    let chosen = match size {
        InterfaceSize::Auto => (window / AUTO_BASE).min_element(),
        InterfaceSize::Percent(percent) => f32::from(percent) / 100.0,
    };
    let smallest = if touch { TOUCH_SMALLEST } else { SMALLEST };
    chosen.clamp(smallest, LARGEST)
}

/// Keeps `UiScale` as the preferences and the window say. Runs before the
/// pointer is read, which is given in the interface's pixels too.
pub(super) fn apply_interface_scale(
    window: Single<&Window, With<PrimaryWindow>>,
    preferences: Res<Preferences>,
    touch_only: Res<TouchOnly>,
    mut scale: ResMut<UiScale>,
) {
    let size = window.size();
    // Minimised: kept as it was for coming back.
    if size.min_element() < 1.0 {
        return;
    }
    let wanted = interface_scale(
        preferences.interface_size,
        preferences.touch || touch_only.0,
        size,
    );
    if scale.0 != wanted {
        scale.0 = wanted;
    }
}

#[cfg(test)]
mod tests;
