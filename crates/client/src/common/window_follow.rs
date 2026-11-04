//! The window follows `DisplayInUse` (native), on the monitor it is on.

use bevy::prelude::*;
use bevy::window::{
    Monitor, MonitorSelection, OnMonitor, PrimaryWindow, VideoModeSelection, WindowMode,
};

use super::display::closest_mode;
use super::{DisplayInUse, DisplayMode, Preferences};

/// Shows the window as `DisplayInUse` says whenever it changes, and at the
/// start once the window is on a monitor (it is made windowed, where it was,
/// so full screen opens on that monitor).
pub(super) fn follow_display(
    in_use: Res<DisplayInUse>,
    preferences: Res<Preferences>,
    mut window: Single<(&mut Window, Option<&OnMonitor>), With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    mut shown: Local<Option<DisplayInUse>>,
) {
    let (window, on) = &mut *window;
    let Some(&OnMonitor(monitor)) = *on else {
        return;
    };
    if *shown == Some(*in_use) {
        return;
    }
    let first = shown.is_none();
    *shown = Some(*in_use);
    let choice = in_use.0;
    let on_it = MonitorSelection::Entity(monitor);
    let mode = match choice.mode {
        DisplayMode::Windowed => WindowMode::Windowed,
        DisplayMode::Borderless => WindowMode::BorderlessFullscreen(on_it),
        DisplayMode::FullScreen => {
            let modes = monitors
                .get(monitor)
                .map(|monitor| &monitor.video_modes[..]);
            let found = choice
                .resolution
                .zip(modes.ok())
                .and_then(|(wanted, modes)| Some((wanted, closest_mode(modes, wanted)?)));
            if let Some((wanted, found)) = found
                && found.physical_size != UVec2::new(wanted.width, wanted.height)
            {
                warn!(
                    "This screen can't show {} × {}, so using {} × {}",
                    wanted.width, wanted.height, found.physical_size.x, found.physical_size.y
                );
            }
            let video = found.map_or(VideoModeSelection::Current, |(_, found)| {
                VideoModeSelection::Specific(found)
            });
            WindowMode::Fullscreen(on_it, video)
        }
    };
    if window.mode != mode {
        window.mode = mode;
        // Back from full screen: maximised again, if it was.
        if !first && choice.mode == DisplayMode::Windowed && preferences.window.maximised {
            window.set_maximized(true);
        }
    }
}
