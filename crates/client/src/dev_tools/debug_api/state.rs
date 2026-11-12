//! `client.state` and `client.screenshot`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use server::debug_api::{BrpError, BrpResult, Value, json, params};
use std::path::PathBuf;
use std::{fs, path, process};

use super::super::FakeCursor;
use crate::common::{
    CurrentDungeon, Facing, GameView, MapPosition, OtherLevels, PointerPosition, ServerClock,
    SpotPosition, Target,
};
use crate::debug_menu::DebugMenu;
use crate::minimap::FullMap;
use crate::player::Player;
use crate::system_menu::SystemMenu;

/// The answer of a method that only does something.
pub(super) fn done() -> BrpResult {
    Ok(json!("ok"))
}

/// `client.state`: which client answered (process and the data folder its
/// embedded server saves in, so a caller can tell it is talking to its own
/// game), what it shows, where the player is and whom they target, the clock,
/// the window, the pointer and what is open.
#[allow(clippy::too_many_arguments)]
pub(super) fn state(
    _: In<Option<Value>>,
    view: Res<State<GameView>>,
    player: Query<(&MapPosition, &SpotPosition, &Facing), With<Player>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    clock: Res<ServerClock>,
    (window, ui_scale): (Query<&Window, With<PrimaryWindow>>, Res<UiScale>),
    open: (Res<FullMap>, Res<DebugMenu>, Res<SystemMenu>),
    target: Res<Target>,
    pointer: (Res<PointerPosition>, Res<FakeCursor>),
) -> BrpResult {
    let view = *view.get();
    let (map, debug_menu, system_menu) = open;
    Ok(json!({
        "program": "client",
        "pid": process::id(),
        "data": server::default_data_folder(),
        "view": format!("{view:?}"),
        "position": player.single().ok().map(|(at, _, _)| json!({ "x": at.x, "y": at.y })),
        // On a level or in the Hall, exactly, in units (240 a cell).
        "spot": player.single().ok().map(|(_, at, _)| json!({ "x": at.0.x, "y": at.0.y })),
        // Which way they face (`shared::facing::Heading`: 0 east, growing clockwise).
        "facing": player.single().ok().map(|(_, _, facing)| facing.0.0),
        "level": if view == GameView::Dungeon { dungeon.info.as_ref() } else { None },
        "showing_floor": levels.showing(),
        "target": target.0,
        "clock_minutes": clock.minutes,
        "paused": clock.paused,
        "window": window.single().ok().map(|window| json!({
            "width": window.width(),
            "height": window.height(),
            "scale_factor": window.scale_factor(),
        })),
        "ui_scale": ui_scale.0,
        // Where the game's pointer is (logical pixels), and whether dev tools
        // hold it in place of the real mouse.
        "pointer": json!({
            "at": pointer.0.0.map(|at| [at.x, at.y]),
            "fake": pointer.1.at.is_some(),
        }),
        "full_map": map.open,
        "debug_menu": debug_menu.open,
        "system_menu": format!("{:?}", *system_menu),
    }))
}

#[derive(Deserialize)]
pub(super) struct ScreenshotParams {
    /// Where to save it (`.png`); relative to the game's working folder.
    path: PathBuf,
}

/// `client.screenshot { "path": "target/shots/a.png" }`: saves the window as
/// it is at the end of this frame. Answers at once with the full path; the
/// file follows a frame or two later (`cargo xtask api` waits for files it is
/// told of in `wait_for_file`). The game carries on. The image is in physical
/// pixels: `logical_per_pixel` turns a point on it into what `client.click`
/// takes (a viewer that shrinks it to show it changes that again, so aim by
/// widget id where there is one).
pub(super) fn screenshot(
    In(request): In<Option<Value>>,
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
) -> BrpResult {
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
    let logical_per_pixel = window
        .single()
        .map_or(1.0, |window| 1.0 / window.scale_factor());
    Ok(json!({
        "path": path,
        "wait_for_file": path,
        "logical_per_pixel": logical_per_pixel,
    }))
}
