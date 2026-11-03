//! The client's debug API methods (only with the `debug-api` feature): what
//! the game is showing, and screenshots, without restarting it. The rest of
//! the dev tools follow as methods too. See `docs/context/debug-api.md`.

use std::path::PathBuf;
use std::{fs, path, process};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use server::debug_api::{AddDebugMethod, BrpError, BrpResult, Value, json, params};

use crate::common::{CurrentDungeon, GameView, MapPosition, ServerClock};
use crate::player::Player;

pub(super) fn add(app: &mut App) {
    app.add_debug_method("client.state", state)
        .add_debug_method("client.screenshot", screenshot);
}

/// `client.state`: which client answered (process and the data folder its
/// embedded server saves in, so a caller can tell it is talking to its own
/// game), what it shows, where the player is, the clock and the window.
fn state(
    _: In<Option<Value>>,
    view: Res<State<GameView>>,
    player: Query<&MapPosition, With<Player>>,
    dungeon: Res<CurrentDungeon>,
    clock: Res<ServerClock>,
    window: Query<&Window, With<PrimaryWindow>>,
) -> BrpResult {
    let view = *view.get();
    Ok(json!({
        "program": "client",
        "pid": process::id(),
        "data": server::default_data_folder(),
        "view": format!("{view:?}"),
        "position": player.single().ok().map(|at| json!({ "x": at.x, "y": at.y })),
        "level": if view == GameView::Dungeon { dungeon.info.as_ref() } else { None },
        "clock_minutes": clock.minutes,
        "paused": clock.paused,
        "window": window.single().ok().map(|window| json!({
            "width": window.width(),
            "height": window.height(),
        })),
    }))
}

#[derive(Deserialize)]
struct ScreenshotParams {
    /// Where to save it (`.png`); relative to the game's working folder.
    path: PathBuf,
}

/// `client.screenshot { "path": "target/shots/a.png" }`: saves the window as
/// it is at the end of this frame. Answers at once with the full path; the
/// file follows a frame or two later (`cargo xtask api` waits for files it is
/// told of in `wait_for_file`). The game carries on.
fn screenshot(In(request): In<Option<Value>>, mut commands: Commands) -> BrpResult {
    let ScreenshotParams { path } = params(request)?;
    let path = path::absolute(&path).map_err(BrpError::internal)?;
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(BrpError::internal)?;
    }
    // Any older copy goes first, so waiting for the file waits for this one.
    let _ = fs::remove_file(&path);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path.clone()));
    Ok(json!({ "path": path, "wait_for_file": path }))
}
