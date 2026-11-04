//! `HOW_DEBUG_WINDOW`: how the window opens, the same every run, whatever was
//! saved, and without saving anything about it (`common::RememberWindow`).
//!
//! - `full-screen,WxH`: full screen at that resolution (physical pixels; the
//!   nearest the screen has, if not that).
//! - `full-screen`: borderless full screen.
//! - `maximised`: windowed and maximised.
//! - `WxH`: windowed at that size (logical pixels), in the middle of the main
//!   monitor.

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowPosition};

use crate::common::{DisplayChoice, DisplayInUse, DisplayMode, RememberWindow, Resolution};

const VARIABLE: &str = "HOW_DEBUG_WINDOW";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowStart {
    FullScreen(u32, u32),
    Borderless,
    Maximised,
    Windowed(u32, u32),
}

/// `WxH`, as two numbers.
fn parse_size(text: &str) -> Option<(u32, u32)> {
    let (width, height) = text.trim().split_once(['x', 'X'])?;
    let size = (width.trim().parse().ok()?, height.trim().parse().ok()?);
    (size.0 > 0 && size.1 > 0).then_some(size)
}

pub(super) fn parse_window(text: &str) -> Option<WindowStart> {
    let text = text.trim().to_ascii_lowercase();
    match text.split_once(',') {
        Some(("full-screen", size)) => {
            parse_size(size).map(|(width, height)| WindowStart::FullScreen(width, height))
        }
        Some(_) => None,
        None => match text.as_str() {
            "full-screen" => Some(WindowStart::Borderless),
            "maximised" | "maximized" => Some(WindowStart::Maximised),
            size => parse_size(size).map(|(width, height)| WindowStart::Windowed(width, height)),
        },
    }
}

pub(super) fn add(app: &mut App) {
    let Ok(text) = std::env::var(VARIABLE) else {
        return;
    };
    let Some(start) = parse_window(&text) else {
        warn!("{VARIABLE}: can't read \"{text}\" (full-screen,WxH, full-screen, maximised or WxH)");
        return;
    };
    let choice = match start {
        WindowStart::FullScreen(width, height) => DisplayChoice {
            mode: DisplayMode::FullScreen,
            resolution: Some(Resolution {
                width,
                height,
                refresh_millihertz: 0,
            }),
        },
        WindowStart::Borderless => DisplayChoice {
            mode: DisplayMode::Borderless,
            resolution: None,
        },
        WindowStart::Maximised | WindowStart::Windowed(..) => DisplayChoice::default(),
    };
    app.insert_resource(DisplayInUse(choice))
        .insert_resource(RememberWindow(false));
    let world = app.world_mut();
    let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
    if let Ok(mut window) = windows.single_mut(world) {
        // Full screen opens on the main monitor too.
        window.position = WindowPosition::Centered(MonitorSelection::Primary);
        if let WindowStart::Windowed(width, height) = start {
            window.resolution.set(width as f32, height as f32);
        }
    }
    if start == WindowStart::Maximised {
        app.add_systems(
            Startup,
            |mut window: Single<&mut Window, With<PrimaryWindow>>| window.set_maximized(true),
        );
    }
}

#[cfg(test)]
mod tests;
