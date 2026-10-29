//! Developer helpers, controlled by environment variables. Native only; does
//! nothing unless a variable is set. See `docs/context/client/dev_tools.md`.
//! Starting positions and states are applied once connected to the server.
//!
//! - `GAME_POS=x,y`: starting grid position on the world map (the embedded
//!   server moves the player there).
//! - `GAME_ENTER=kind[,seed[,level]]`: start inside a made-up site at the
//!   starting position (the embedded server allows it): `kind` is an index
//!   into `FeatureKind::ALL`, `seed` its map seed (default 1), `level` the
//!   level (default 0, the way in).
//! - `GAME_MOVE=dx,dy`: a second after starting, step the player this far
//!   (e.g. off the stairs they arrived on, to see them).
//! - `GAME_TRAVEL=x,y`: start travelling to a grid position (as if clicked),
//!   once at `GAME_POS` (if set).
//! - `GAME_WALK=1`: start with auto-walk on (as if G was pressed).
//! - `GAME_SNOW=n`: start with the snow line at `n` (128 normal, 0 frozen).
//! - `GAME_WINDOW_SIZE=w,h`: resize the window (logical pixels) a second after
//!   starting, to check that views handle resizing.
//! - `GAME_TOKEN=shape,attitude[,completed[,size]]`: the player's token, as
//!   numbers (indexes into `TokenShape::ALL` and `Attitude::ALL`), e.g. `3,2`
//!   is a blue hexagon; a third value of `1` makes it completed, a fourth is
//!   an index into `TokenSize::ALL` (`0` tiny ... `5` titan).
//! - `GAME_VIEW=yaw[,pitch[,zoom]]`: start with the map view turned by `yaw`
//!   degrees (positive moves the camera towards the east; 90 looks west, 180
//!   looks south), tilted to `pitch` degrees above the horizon and zoomed in
//!   (the camera distance is divided by `zoom`; 1.6667 is the wheel's
//!   closest).
//! - `GAME_SCREENSHOT=path.png`: save a screenshot after `GAME_SCREENSHOT_DELAY`
//!   seconds (default 5), then quit. Lets tools check visuals without a person.

use std::env;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;

use shared::feature::FeatureKind;
use shared::movement::steps_between;
use shared::protocol::ClientMessage;

use crate::common::{Attitude, GameView, MapPosition, Token, TokenShape, TokenSize, Weather};
use crate::connection::ToServer;
use crate::demo::AutoWalk;
use crate::player::{Player, PlayerSystems, TakeStep, TravelTo};
use crate::view3d::OrbitCamera;

/// Time between taking the screenshot and quitting, so it can be saved.
const SAVE_SECONDS: f32 = 2.0;

pub struct DevToolsPlugin;

/// The order of what is asked of the server on starting: move, then enter.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
enum Start {
    Place,
    Enter,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnExit(GameView::Connecting),
            (Start::Place, Start::Enter).chain(),
        );

        if let Some((x, y)) = env::var("GAME_POS").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(
                OnExit(GameView::Connecting),
                (move |mut to_server: MessageWriter<ToServer>| {
                    to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
                })
                .in_set(Start::Place),
            );
        }

        if let Ok(enter) = env::var("GAME_ENTER") {
            let numbers: Vec<u64> = enter
                .split(',')
                .filter_map(|n| n.trim().parse().ok())
                .collect();
            let kind = numbers
                .first()
                .and_then(|&k| FeatureKind::ALL.get(k as usize).copied())
                .unwrap_or(FeatureKind::Castle);
            let seed = numbers.get(1).copied().unwrap_or(1);
            let level = numbers.get(2).copied().unwrap_or(0) as u32;
            app.add_systems(
                OnExit(GameView::Connecting),
                (move |mut to_server: MessageWriter<ToServer>| {
                    // Where the player is (after `GAME_POS`), as its way out.
                    to_server.write(ToServer(ClientMessage::DevEnterSite { kind, seed, level }));
                })
                .in_set(Start::Enter),
            );
        }

        if let Some((dx, dy)) = env::var("GAME_MOVE").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(
                Update,
                (move |time: Res<Time>,
                       mut done: Local<bool>,
                       mut steps: MessageWriter<TakeStep>| {
                    if !*done && time.elapsed_secs() > 1.0 {
                        *done = true;
                        for step in steps_between((0, 0), (dx, dy)) {
                            steps.write(TakeStep(step));
                        }
                    }
                })
                .in_set(PlayerSystems::Input),
            );
        }

        if let Some((x, y)) = env::var("GAME_TRAVEL").ok().and_then(|pos| parse_pos(&pos)) {
            let start = env::var("GAME_POS").ok().and_then(|pos| parse_pos(&pos));
            app.add_systems(
                Update,
                (move |mut done: Local<bool>,
                       player: Single<&MapPosition, With<Player>>,
                       mut travel: MessageWriter<TravelTo>| {
                    // Not before the server has moved the player to `GAME_POS`:
                    // that would end the journey.
                    let ready = start.is_none_or(|(sx, sy)| (player.x, player.y) == (sx, sy));
                    if !*done && ready {
                        *done = true;
                        travel.write(TravelTo { x, y });
                    }
                })
                .in_set(PlayerSystems::Input)
                .run_if(in_state(GameView::WorldMap)),
            );
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
                OnExit(GameView::Connecting),
                move |mut token: Single<&mut Token, With<Player>>| {
                    **token = new_token;
                },
            );
        }

        if let Ok(view) = env::var("GAME_VIEW") {
            let numbers: Vec<f32> = view
                .split(',')
                .filter_map(|n| n.trim().parse().ok())
                .collect();
            app.add_systems(
                OnExit(GameView::Connecting),
                move |mut cameras: Query<&mut OrbitCamera>| {
                    for mut camera in &mut cameras {
                        if let Some(yaw) = numbers.first() {
                            camera.yaw = yaw.to_radians();
                        }
                        if let Some(pitch) = numbers.get(1) {
                            camera.pitch = pitch.to_radians();
                        }
                        if let Some(zoom) = numbers.get(2) {
                            camera.distance /= zoom;
                        }
                    }
                },
            );
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
    let size = numbers
        .next()
        .flatten()
        .and_then(|n| TokenSize::ALL.get(n).copied())
        .unwrap_or_default();
    Some(Token {
        shape,
        attitude,
        completed,
        size,
    })
}

fn parse_pos(text: &str) -> Option<(i64, i64)> {
    let (x, y) = text.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
