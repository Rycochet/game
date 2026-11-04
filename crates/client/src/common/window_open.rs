//! Opening the window where it was left (native): its place and size set
//! before it is made, then maximised if it was; once the monitors are known,
//! back on the monitor it was on, or the main one if its place is on none.

use bevy::prelude::*;
use bevy::window::{Monitor, MonitorSelection, PrimaryMonitor, PrimaryWindow, WindowPosition};

use super::{Preferences, RememberWindow, WindowPrefs};

/// Sets up the window, not yet made, from the saved preferences.
pub(super) fn open_as_saved(app: &mut App, saved: &WindowPrefs) {
    if cfg!(target_arch = "wasm32") {
        // A browser's canvas fills the page anyway.
        return;
    }
    let world = app.world_mut();
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    let Ok(mut window) = windows.single_mut(world) else {
        return;
    };
    if let Some(place) = saved.place {
        window.position = WindowPosition::At(IVec2::new(place.x, place.y));
        window.resolution.set(place.width, place.height);
    }
}

/// Maximised, if it was. (Asked for once the window is made: a request before
/// that is lost.)
pub(super) fn maximise_as_saved(
    preferences: Res<Preferences>,
    remember: Res<RememberWindow>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if cfg!(not(target_arch = "wasm32")) && remember.0 && preferences.window.maximised {
        window.set_maximized(true);
    }
}

/// Once there are monitors: if the window isn't on the one it was on, it goes
/// to the middle of that one if it is still there, else, if it is on none, to
/// the middle of the main one.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(super) fn place_on_monitor(
    preferences: Res<Preferences>,
    remember: Res<RememberWindow>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    monitors: Query<(Entity, &Monitor, Has<PrimaryMonitor>)>,
    mut done: Local<bool>,
) {
    if *done || monitors.is_empty() {
        return;
    }
    *done = true;
    let saved = &preferences.window;
    // (Not where dev tools put it, or a first run, which the system places.)
    let Some(place) = saved.place.filter(|_| remember.0) else {
        return;
    };
    let middle = IVec2::new(
        place.x + place.width as i32 / 2,
        place.y + place.height as i32 / 2,
    );
    let holding = monitors
        .iter()
        .find(|(_, monitor, _)| holds(monitor, middle))
        .map(|(entity, ..)| entity);
    let named = saved.monitor.as_ref().and_then(|name| {
        monitors
            .iter()
            .find(|(_, monitor, _)| monitor.name.as_ref() == Some(name))
            .map(|(entity, ..)| entity)
    });
    let to = match (named, holding) {
        (Some(named), Some(holding)) if named == holding => return,
        (Some(named), _) => MonitorSelection::Entity(named),
        (None, Some(_)) => return,
        (None, None) => MonitorSelection::Primary,
    };
    window.position = WindowPosition::Centered(to);
}

/// Whether this point on the desktop is on the monitor.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
fn holds(monitor: &Monitor, point: IVec2) -> bool {
    let size = IVec2::new(
        monitor.physical_width as i32,
        monitor.physical_height as i32,
    );
    let corner = monitor.physical_position;
    point.cmpge(corner).all() && point.cmplt(corner + size).all()
}
