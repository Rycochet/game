//! Developer helpers, controlled by environment variables. Native only; does
//! nothing unless a variable is set. See `docs/context/client/dev_tools.md`.
//!
//! - `GAME_POS=x,y`: starting grid position.
//! - `GAME_TRAVEL=x,y`: start travelling to a grid position (as if clicked).
//! - `GAME_WALK=1`: start with auto-walk on (as if G was pressed).
//! - `GAME_SNOW=n`: start with the snow line at `n` (128 normal, 0 frozen).
//! - `GAME_WINDOW_SIZE=w,h`: resize the window (logical pixels) a second after
//!   starting, to check that views handle resizing.
//! - `GAME_TOKEN=shape,attitude[,1]`: the player's token, as numbers (indexes
//!   into `TokenShape::ALL` and `Attitude::ALL`), e.g. `3,2` is a blue
//!   hexagon; a third value of `1` makes it completed.
//! - `GAME_YAW=degrees`: start with the map view turned (positive moves the
//!   camera towards the east; 90 looks west, 180 looks south).
//! - `GAME_SCREENSHOT=path.png`: save a screenshot after `GAME_SCREENSHOT_DELAY`
//!   seconds (default 5), then quit. Lets tools check visuals without a person.

use std::env;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;

use crate::common::{Attitude, Token, TokenShape, Weather};
use crate::demo::AutoWalk;
use crate::player::{MapPosition, Player, TravelTo};
use crate::world_map::OrbitCamera;

/// Time between taking the screenshot and quitting, so it can be saved.
const SAVE_SECONDS: f32 = 2.0;

pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        if let Some((x, y)) = env::var("GAME_POS").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(
                PostStartup,
                move |mut player: Single<&mut MapPosition, With<Player>>| {
                    **player = MapPosition { x, y };
                },
            );
        }

        if let Some((x, y)) = env::var("GAME_TRAVEL").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(PostStartup, move |mut travel: MessageWriter<TravelTo>| {
                travel.write(TravelTo { x, y });
            });
        }

        if env::var("GAME_WALK").is_ok() {
            app.insert_resource(AutoWalk(true));
        }

        if let Some((width, height)) = env::var("GAME_WINDOW_SIZE")
            .ok()
            .and_then(|size| parse_pos(&size))
        {
            let (width, height) = (width as f32, height as f32);
            app.add_systems(
                Update,
                move |time: Res<Time>,
                      mut done: Local<bool>,
                      mut window: Single<&mut Window, With<PrimaryWindow>>| {
                    if !*done && time.elapsed_secs() > 1.0 {
                        *done = true;
                        window.resolution.set(width, height);
                    }
                },
            );
        }

        if let Some(snow_line) = env::var("GAME_SNOW").ok().and_then(|v| v.parse().ok()) {
            app.insert_resource(Weather { snow_line });
        }

        if let Some(new_token) = env::var("GAME_TOKEN").ok().and_then(|t| parse_token(&t)) {
            app.add_systems(
                PostStartup,
                move |mut token: Single<&mut Token, With<Player>>| {
                    **token = new_token;
                },
            );
        }

        if let Some(degrees) = env::var("GAME_YAW")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
        {
            app.add_systems(PostStartup, move |mut cameras: Query<&mut OrbitCamera>| {
                for mut camera in &mut cameras {
                    camera.yaw = degrees.to_radians();
                }
            });
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

/// `shape,attitude` or `shape,attitude,completed`, all numbers.
fn parse_token(text: &str) -> Option<Token> {
    let mut numbers = text.split(',').map(|n| n.trim().parse::<usize>().ok());
    let shape = *TokenShape::ALL.get(numbers.next()??)?;
    let attitude = *Attitude::ALL.get(numbers.next()??)?;
    let completed = numbers.next().flatten() == Some(1);
    Some(Token {
        shape,
        attitude,
        completed,
    })
}

fn parse_pos(text: &str) -> Option<(i64, i64)> {
    let (x, y) = text.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
