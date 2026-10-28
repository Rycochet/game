//! Developer helpers, controlled by environment variables. Native only; does
//! nothing unless a variable is set. See `docs/context/client/dev_tools.md`.
//!
//! - `GAME_VIEW=mesh|slices`: which view to start in.
//! - `GAME_POS=x,y`: starting grid position.
//! - `GAME_WALK=1`: start with auto-walk on (as if G was pressed).
//! - `GAME_SNOW=n`: start with the snow line at `n` (128 normal, 0 frozen).
//! - `GAME_SCREENSHOT=path.png`: save a screenshot after `GAME_SCREENSHOT_DELAY`
//!   seconds (default 5), then quit. Lets tools check visuals without a person.

use std::env;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::common::{MapView, Weather};
use crate::demo::AutoWalk;
use crate::player::{MapPosition, Player};

/// Time between taking the screenshot and quitting, so it can be saved.
const SAVE_SECONDS: f32 = 2.0;

pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        match env::var("GAME_VIEW").as_deref() {
            Ok("mesh") => app.insert_state(MapView::Mesh),
            Ok("slices") => app.insert_state(MapView::Slices),
            _ => app,
        };

        if let Some((x, y)) = env::var("GAME_POS").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(
                PostStartup,
                move |mut player: Single<&mut MapPosition, With<Player>>| {
                    **player = MapPosition { x, y };
                },
            );
        }

        if env::var("GAME_WALK").is_ok() {
            app.insert_resource(AutoWalk(true));
        }

        if let Some(snow_line) = env::var("GAME_SNOW").ok().and_then(|v| v.parse().ok()) {
            app.insert_resource(Weather { snow_line });
        }

        if let Ok(path) = env::var("GAME_SCREENSHOT") {
            let delay = env::var("GAME_SCREENSHOT_DELAY")
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
}

#[derive(Resource)]
struct ScreenshotPlan {
    path: String,
    at: f32,
    taken: bool,
}

fn screenshot_then_quit(
    mut commands: Commands,
    time: Res<Time>,
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

fn parse_pos(text: &str) -> Option<(i64, i64)> {
    let (x, y) = text.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
