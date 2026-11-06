//! Keeping frames coming while loading.

use bevy::prelude::*;
use bevy::winit::WinitSettings;

use crate::common::{KeepDrawing, LoadingStep, Preferences, Session, frame_settings};

/// Keeps loading quick even with the window in the background, where frames are
/// otherwise a second apart (as they are after it, unless `KeepDrawing`).
pub(super) fn keep_frame_rate(
    step: Res<LoadingStep>,
    session: Res<Session>,
    keep: Res<KeepDrawing>,
    preferences: Res<Preferences>,
    mut settings: ResMut<WinitSettings>,
) {
    let changed = step.is_changed() || session.is_changed() || keep.is_changed();
    if changed || preferences.is_changed() {
        // Not at the startup menu, which only waits for the player.
        let loading = *step != LoadingStep::Ready && session.is_playing();
        *settings = frame_settings(loading || keep.0, &preferences.video);
    }
}
