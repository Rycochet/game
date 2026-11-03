//! The full map: whether it is open, where it is dragged to and its zoom; the darkening behind it.

use bevy::prelude::*;

use super::{FullMapDarkening, OPENING_ZOOM};

/// The full map: whether it is open, where it has been dragged to, and how
/// far it is zoomed in.
#[derive(Resource)]
pub struct FullMap {
    pub open: bool,
    /// The grid position in its middle; `None` follows the player.
    pub centre: Option<(f64, f64)>,
    /// Its scale against the minimap's.
    pub zoom: f32,
    /// Where the cursor was last frame, while dragging.
    pub(super) drag_from: Option<Vec2>,
    /// The floor just changed to, keeping the map's offset from the player
    /// even if nothing known shows there.
    pub(super) kept_on: Option<u32>,
}

impl Default for FullMap {
    fn default() -> Self {
        Self {
            open: false,
            centre: None,
            zoom: OPENING_ZOOM,
            drag_from: None,
            kept_on: None,
        }
    }
}

impl FullMap {
    /// Opens (or closes) it as it first looks: on the player, at the opening
    /// zoom.
    pub(super) fn set_open(&mut self, open: bool) {
        *self = Self { open, ..default() };
    }
}

/// The darkening shows while the full map does.
pub(super) fn show_darkening(
    full: Res<FullMap>,
    mut behind: Single<&mut Visibility, With<FullMapDarkening>>,
) {
    behind.set_if_neq(if full.open {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}
