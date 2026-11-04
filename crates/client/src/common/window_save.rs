//! Remembering where the window is (native): its place and size, whether it
//! is maximised, and the monitor it is on, once it has stopped moving.

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::{Monitor, OnMonitor, PrimaryWindow, WindowMoved, WindowResized};
use bevy::winit::WINIT_WINDOWS;

use super::display::WindowPlace;
use super::{DisplayInUse, DisplayMode, Preferences, RememberWindow};

/// How long after the last move or resize it is saved, so a drag isn't saved
/// at every step.
const SETTLE_SECONDS: f64 = 0.5;

/// The window, and the monitor it is on (and whether that changed).
type TheWindow = (Entity, &'static Window, Option<Ref<'static, OnMonitor>>);

#[allow(clippy::too_many_arguments)]
pub(super) fn remember_window(
    mut moved: MessageReader<WindowMoved>,
    mut resized: MessageReader<WindowResized>,
    time: Res<Time<Real>>,
    remember: Res<RememberWindow>,
    in_use: Res<DisplayInUse>,
    window: Single<TheWindow, With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    mut preferences: ResMut<Preferences>,
    mut due: Local<Option<f64>>,
    // On the main thread, where winit's windows are.
    _main_thread: NonSendMarker,
) {
    let now = time.elapsed_secs_f64();
    let (entity, window, on) = window.into_inner();
    // (On another monitor: its name, even without a move to tell.)
    let other_monitor = on.as_ref().is_some_and(|on| on.is_changed());
    if moved.read().count() + resized.read().count() > 0 || other_monitor {
        *due = Some(now + SETTLE_SECONDS);
    }
    if !remember.0 {
        *due = None;
        return;
    }
    if !due.is_some_and(|at| now >= at) {
        return;
    }
    *due = None;
    // Bevy can ask for maximised, but only winit knows whether it is.
    let state = WINIT_WINDOWS.with_borrow(|windows| {
        let winit = windows.get_window(entity)?;
        Some((
            winit.is_maximized(),
            winit.is_minimized() == Some(true),
            winit.outer_position().ok(),
        ))
    });
    let Some((maximised, minimised, corner)) = state else {
        return;
    };
    if minimised {
        return;
    }
    let mut saved = preferences.window.clone();
    if let Some(name) = on
        .and_then(|on| monitors.get(on.0).ok())
        .and_then(|monitor| monitor.name.clone())
    {
        saved.monitor = Some(name);
    }
    if in_use.0.mode == DisplayMode::Windowed {
        saved.maximised = maximised;
        if let Some(corner) = corner.filter(|_| !maximised) {
            saved.place = Some(WindowPlace {
                x: corner.x,
                y: corner.y,
                width: window.width(),
                height: window.height(),
            });
        }
    }
    if saved != preferences.window {
        preferences.window = saved;
    }
}
