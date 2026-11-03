//! A screenshot after a delay, then quitting.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use std::env;

use super::SAVE_SECONDS;

/// `HOW_DEBUG_SCREENSHOT` and `HOW_DEBUG_SCREENSHOT_DELAY`.
pub(super) fn add(app: &mut App) {
    if let Ok(path) = env::var("HOW_DEBUG_SCREENSHOT") {
        let delay = env::var("HOW_DEBUG_SCREENSHOT_DELAY")
            .ok()
            .and_then(|delay| delay.parse().ok())
            .unwrap_or(5.0);
        app.insert_resource(ScreenshotPlan {
            path,
            at: delay,
            taken: false,
        })
        .add_systems(Update, screenshot_then_quit);
    }
}

#[derive(Resource)]
pub(super) struct ScreenshotPlan {
    path: String,
    at: f32,
    taken: bool,
}

pub(super) fn screenshot_then_quit(
    mut commands: Commands,
    // Real time, to work while the game is paused.
    time: Res<Time<Real>>,
    mut plan: ResMut<ScreenshotPlan>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = time.elapsed_secs();
    if !plan.taken && now >= plan.at {
        plan.taken = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(plan.path.clone()));
    } else if plan.taken && now >= plan.at + SAVE_SECONDS {
        exit.write(AppExit::Success);
    }
}
