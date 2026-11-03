//! The client's debug API methods (only with the `debug-api` feature): what
//! the game is showing, screenshots, and what the `HOW_DEBUG_` variables do,
//! at any time, without restarting. They go through the same code as the
//! variables (`super`), and ask the server as the game does, so the server's
//! checks still apply. See `docs/context/debug-api.md`.

use std::path::PathBuf;
use std::{fs, path, process};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use server::debug_api::{AddDebugMethod, BrpError, BrpResult, Value, json, params, refused};
use shared::creature::CreatureKind;
use shared::dungeon::Cell;
use shared::item::{Favour, ItemKind, Quality};
use shared::movement::steps_between;
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;
use shared::site::StairsStyle;

use super::{Forced, open_map, token_of, turn_camera};
use crate::common::{
    CurrentDungeon, CurrentWorld, GameView, HerbGrowthShift, InterfaceSize, MapPosition,
    OtherLevels, Preferences, ServerClock, Token,
};
use crate::connection::ToServer;
use crate::debug_menu::{AutoWalk, DebugAction, DebugMenu};
use crate::inventory::InventoryDebug;
use crate::minimap::FullMap;
use crate::player::{Player, TakeStep, TravelTo};
use crate::system_menu::SystemMenu;
use crate::view3d::OrbitCamera;

pub(super) fn add(app: &mut App) {
    app.add_debug_method("client.state", state)
        .add_debug_method("client.screenshot", screenshot)
        // Asking the server.
        .add_debug_method("client.teleport", teleport)
        .add_debug_method("client.enter_site", enter_site)
        .add_debug_method("client.set_cells", set_cells)
        .add_debug_method("client.set_time", set_time)
        .add_debug_method("client.snow_line", snow_line)
        .add_debug_method("client.reveal_pois", reveal_pois)
        .add_debug_method("client.give_item", give_item)
        .add_debug_method("client.set_creature", set_creature)
        // Moving.
        .add_debug_method("client.step", step)
        .add_debug_method("client.travel", travel)
        .add_debug_method("client.walk", walk)
        // How things look.
        .add_debug_method("client.view", view)
        .add_debug_method("client.window_size", window_size)
        .add_debug_method("client.interface", interface)
        .add_debug_method("client.cursor", cursor)
        .add_debug_method("client.stairs_style", stairs_style)
        .add_debug_method("client.fields_stage", fields_stage)
        .add_debug_method("client.herbs_shift", herbs_shift)
        .add_debug_method("client.celestial", celestial)
        .add_debug_method("client.token", token)
        // What is open.
        .add_debug_method("client.map", map)
        .add_debug_method("client.show_floor", show_floor)
        .add_debug_method("client.debug_action", debug_action)
        .add_debug_method("client.debug_menu", debug_menu)
        .add_debug_method("client.system_menu", system_menu)
        .add_debug_method("client.inventory", inventory);
}

/// The answer of a method that only does something.
fn done() -> BrpResult {
    Ok(json!("ok"))
}

/// `client.state`: which client answered (process and the data folder its
/// embedded server saves in, so a caller can tell it is talking to its own
/// game), what it shows, where the player is, the clock, the window and
/// what is open.
#[allow(clippy::too_many_arguments)]
fn state(
    _: In<Option<Value>>,
    view: Res<State<GameView>>,
    player: Query<&MapPosition, With<Player>>,
    dungeon: Res<CurrentDungeon>,
    levels: Res<OtherLevels>,
    clock: Res<ServerClock>,
    (window, ui_scale): (Query<&Window, With<PrimaryWindow>>, Res<UiScale>),
    open: (Res<FullMap>, Res<DebugMenu>, Res<SystemMenu>),
) -> BrpResult {
    let view = *view.get();
    let (map, debug_menu, system_menu) = open;
    Ok(json!({
        "program": "client",
        "pid": process::id(),
        "data": server::default_data_folder(),
        "view": format!("{view:?}"),
        "position": player.single().ok().map(|at| json!({ "x": at.x, "y": at.y })),
        "level": if view == GameView::Dungeon { dungeon.info.as_ref() } else { None },
        "showing_floor": levels.showing(),
        "clock_minutes": clock.minutes,
        "paused": clock.paused,
        "window": window.single().ok().map(|window| json!({
            "width": window.width(),
            "height": window.height(),
            "scale_factor": window.scale_factor(),
        })),
        "ui_scale": ui_scale.0,
        "full_map": map.open,
        "debug_menu": debug_menu.open,
        "system_menu": format!("{:?}", *system_menu),
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

#[derive(Deserialize)]
struct Place {
    x: i64,
    y: i64,
}

/// `client.teleport { "x": 10, "y": -4 }`: on the world map, or onto a
/// walkable cell of the site level (as Teleport Mode, `HOW_DEBUG_POS`).
fn teleport(In(request): In<Option<Value>>, mut to_server: MessageWriter<ToServer>) -> BrpResult {
    let Place { x, y } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
    done()
}

#[derive(Deserialize)]
struct EnterSiteParams {
    /// By name, as `PoiKind` is saved: `"Castle"`, `"RockCave"`, ...
    kind: PoiKind,
    #[serde(default = "one")]
    seed: u64,
    #[serde(default)]
    level: u32,
    #[serde(default)]
    ruin: u8,
}

fn one() -> u64 {
    1
}

/// `client.enter_site { "kind": "Castle", "seed": 1, "level": 0, "ruin": 0 }`:
/// a made-up site where the player stands (`HOW_DEBUG_ENTER`).
fn enter_site(In(request): In<Option<Value>>, mut to_server: MessageWriter<ToServer>) -> BrpResult {
    let EnterSiteParams {
        kind,
        seed,
        level,
        ruin,
    } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevEnterSite {
        kind,
        seed,
        level,
        ruin,
    }));
    done()
}

#[derive(Deserialize)]
struct CellChange {
    dx: i64,
    dy: i64,
    /// By name: `"Wall"`, `"SecretDoor"`, `"Water"`, ...
    cell: Cell,
}

#[derive(Deserialize)]
struct SetCellsParams {
    cells: Vec<CellChange>,
}

/// `client.set_cells { "cells": [{ "dx": 1, "dy": 0, "cell": "Wall" }] }`:
/// changes cells of the site level, from where the player stands
/// (`HOW_DEBUG_CELLS`).
fn set_cells(
    In(request): In<Option<Value>>,
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let SetCellsParams { cells } = params(request)?;
    if *view.get() != GameView::Dungeon {
        return Err(refused("only in a site"));
    }
    for CellChange { dx, dy, cell } in cells {
        let (x, y) = (player.x + dx, player.y + dy);
        to_server.write(ToServer(ClientMessage::DevSetCell { x, y, cell }));
    }
    done()
}

#[derive(Deserialize)]
struct SetTimeParams {
    hours: Option<f64>,
    minutes: Option<f64>,
}

/// `client.set_time { "hours": 12 }` (or `minutes`): moves the world's clock
/// to that long since the world began (`HOW_DEBUG_TIME`). Saved.
fn set_time(In(request): In<Option<Value>>, mut to_server: MessageWriter<ToServer>) -> BrpResult {
    let SetTimeParams { hours, minutes } = params(request)?;
    let minutes = match (hours, minutes) {
        (_, Some(minutes)) => minutes,
        (Some(hours), None) => hours * shared::world::MINUTES_PER_HOUR,
        (None, None) => return Err(refused("give hours or minutes")),
    };
    to_server.write(ToServer(ClientMessage::DevSetTime { minutes }));
    done()
}

#[derive(Deserialize)]
struct SnowLineParams {
    /// 128 is normal, 0 frozen.
    line: u32,
}

/// `client.snow_line { "line": 0 }` (`HOW_DEBUG_SNOW`).
fn snow_line(In(request): In<Option<Value>>, mut to_server: MessageWriter<ToServer>) -> BrpResult {
    let SnowLineParams { line } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevSnowLine(line)));
    done()
}

/// `client.reveal_pois`: every poi, on this connection (`HOW_DEBUG_REVEAL_POIS`).
fn reveal_pois(_: In<Option<Value>>, mut to_server: MessageWriter<ToServer>) -> BrpResult {
    to_server.write(ToServer(ClientMessage::DevRevealPois));
    done()
}

#[derive(Deserialize)]
struct GiveItemParams {
    /// By name: `"Ring"`, `"Backpack"`, ...
    kind: ItemKind,
    #[serde(default)]
    quality: Quality,
    #[serde(default)]
    magical: bool,
    #[serde(default)]
    set: bool,
    /// `"Blessed"` or `"Cursed"`.
    favour: Option<Favour>,
}

/// `client.give_item { "kind": "Ring", "quality": "Rare", "magical": true }`:
/// into the pockets, as Give Item does. Saved.
fn give_item(In(request): In<Option<Value>>, mut to_server: MessageWriter<ToServer>) -> BrpResult {
    let GiveItemParams {
        kind,
        quality,
        magical,
        set,
        favour,
    } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical,
        quality,
        set,
        favour,
    }));
    done()
}

#[derive(Deserialize)]
struct SetCreatureParams {
    /// By name: `"Human"`, ...
    kind: CreatureKind,
}

/// `client.set_creature { "kind": "Human" }`: as Change Creature
/// (`HOW_DEBUG_CREATURE`); what no longer fits is lost.
fn set_creature(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let SetCreatureParams { kind } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevSetCreature(kind)));
    done()
}

#[derive(Deserialize)]
struct Offset {
    #[serde(default)]
    dx: i64,
    #[serde(default)]
    dy: i64,
}

/// `client.step { "dx": 2, "dy": 0 }`: steps that far, as the keys do, so
/// not into walls (`HOW_DEBUG_MOVE`).
fn step(In(request): In<Option<Value>>, mut steps: MessageWriter<TakeStep>) -> BrpResult {
    let Offset { dx, dy } = params(request)?;
    for step in steps_between((0, 0), (dx, dy)) {
        steps.write(TakeStep(step));
    }
    done()
}

/// `client.travel { "x": 5, "y": 3 }`: travels there as if clicked, on
/// either map (`HOW_DEBUG_TRAVEL`).
fn travel(In(request): In<Option<Value>>, mut travel: MessageWriter<TravelTo>) -> BrpResult {
    let Place { x, y } = params(request)?;
    travel.write(TravelTo { x, y });
    done()
}

#[derive(Deserialize)]
struct OnParams {
    #[serde(default = "yes")]
    on: bool,
}

fn yes() -> bool {
    true
}

/// `client.walk { "on": true }`: Auto Walk, one step east per frame
/// (`HOW_DEBUG_WALK`).
fn walk(In(request): In<Option<Value>>, mut walk: ResMut<AutoWalk>) -> BrpResult {
    let OnParams { on } = params(request)?;
    walk.0 = on;
    done()
}

#[derive(Deserialize)]
struct ViewParams {
    /// Degrees: 90 puts the camera in the east looking west.
    yaw: Option<f32>,
    /// Degrees above the horizon.
    pitch: Option<f32>,
    /// Divides the camera's present distance.
    zoom: Option<f32>,
}

/// `client.view { "yaw": 90, "pitch": 30, "zoom": 1.5 }` (`HOW_DEBUG_VIEW`):
/// each only if given. Zoom divides the distance it is at now, so it adds up
/// over calls.
fn view(In(request): In<Option<Value>>, mut cameras: Query<&mut OrbitCamera>) -> BrpResult {
    let ViewParams { yaw, pitch, zoom } = params(request)?;
    for mut camera in &mut cameras {
        turn_camera(&mut camera, yaw, pitch, zoom);
    }
    done()
}

#[derive(Deserialize)]
struct SizeParams {
    width: f32,
    height: f32,
}

/// `client.window_size { "width": 1280, "height": 720 }`, in logical pixels
/// (`HOW_DEBUG_WINDOW_SIZE`).
fn window_size(
    In(request): In<Option<Value>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) -> BrpResult {
    let SizeParams { width, height } = params(request)?;
    window.set_maximized(false);
    window.resolution.set(width, height);
    done()
}

#[derive(Deserialize)]
struct InterfaceParams {
    /// `"auto"`, or a percentage.
    size: Option<Value>,
    /// "Enable touch".
    touch: Option<bool>,
}

/// `client.interface { "size": 200, "touch": false }`: the interface size and
/// "Enable touch", as Settings sets them (and saves them: run with
/// `HOW_PREFERENCES` elsewhere to keep the player's own). `"size": "auto"` for Auto.
fn interface(In(request): In<Option<Value>>, mut preferences: ResMut<Preferences>) -> BrpResult {
    let InterfaceParams { size, touch } = params(request)?;
    if let Some(size) = size {
        preferences.interface_size = match &size {
            Value::String(auto) if auto.eq_ignore_ascii_case("auto") => InterfaceSize::Auto,
            Value::Number(percent) => percent
                .as_u64()
                .and_then(|percent| u16::try_from(percent).ok())
                .map(InterfaceSize::Percent)
                .ok_or_else(|| refused(format!("not a percentage: {percent}")))?,
            other => {
                return Err(refused(format!(
                    "size is \"auto\" or a percentage, not {other}"
                )));
            }
        };
    }
    if let Some(touch) = touch {
        preferences.touch = touch;
    }
    done()
}

#[derive(Deserialize)]
struct CursorParams {
    x: Option<f32>,
    y: Option<f32>,
}

/// `client.cursor { "x": 400, "y": 300 }`: holds the mouse cursor there, in
/// logical pixels, until `client.cursor {}` lets it go (`HOW_DEBUG_CURSOR`).
fn cursor(In(request): In<Option<Value>>, mut forced: ResMut<Forced>) -> BrpResult {
    let CursorParams { x, y } = params(request)?;
    forced.cursor = x.zip(y).map(|(x, y)| Vec2::new(x, y));
    done()
}

#[derive(Deserialize)]
struct StyleParams {
    /// An index into `StairsStyle::ALL` (0 spiral ... 6 knotted rope), or
    /// none to draw stairs as they are.
    style: Option<usize>,
}

/// `client.stairs_style { "style": 2 }` (`HOW_DEBUG_STAIRS`).
fn stairs_style(In(request): In<Option<Value>>, mut forced: ResMut<Forced>) -> BrpResult {
    let StyleParams { style } = params(request)?;
    forced.stairs = match style {
        Some(index) => Some(*StairsStyle::ALL.get(index).ok_or_else(|| {
            refused(format!(
                "styles go from 0 to {}",
                StairsStyle::ALL.len() - 1
            ))
        })?),
        None => None,
    };
    done()
}

#[derive(Deserialize)]
struct StageParams {
    /// A field's `variety`, or none to draw fields as they are.
    stage: Option<u32>,
}

/// `client.fields_stage { "stage": 4 }` (`HOW_DEBUG_FIELDS`).
fn fields_stage(In(request): In<Option<Value>>, mut forced: ResMut<Forced>) -> BrpResult {
    let StageParams { stage } = params(request)?;
    forced.fields = stage;
    done()
}

#[derive(Deserialize)]
struct StagesParams {
    stages: u32,
}

/// `client.herbs_shift { "stages": 3 }`: herbs drawn this many game days on
/// (`HOW_DEBUG_HERBS`).
fn herbs_shift(In(request): In<Option<Value>>, mut shift: ResMut<HerbGrowthShift>) -> BrpResult {
    let StagesParams { stages } = params(request)?;
    shift.0 = stages;
    done()
}

#[derive(Deserialize)]
struct SeedParams {
    seed: u64,
}

/// `client.celestial { "seed": 272 }`: the suns and moons of that terrain
/// seed, not this world's, until restarting (`HOW_DEBUG_CELESTIAL`).
fn celestial(In(request): In<Option<Value>>, mut world: ResMut<CurrentWorld>) -> BrpResult {
    let SeedParams { seed } = params(request)?;
    world.celestial = shared::celestial::Celestial::from_seed(shared::rng::Seed::new(seed));
    done()
}

#[derive(Deserialize)]
struct TokenParams {
    /// Indexes into `TokenShape::ALL`, `Attitude::ALL` and `TokenSize::ALL`.
    shape: usize,
    attitude: usize,
    #[serde(default)]
    completed: bool,
    size: Option<usize>,
}

/// `client.token { "shape": 3, "attitude": 3 }`: the player's token
/// (`HOW_DEBUG_TOKEN`).
fn token(In(request): In<Option<Value>>, mut token: Single<&mut Token, With<Player>>) -> BrpResult {
    let TokenParams {
        shape,
        attitude,
        completed,
        size,
    } = params(request)?;
    **token = token_of(shape, attitude, completed, size)
        .ok_or_else(|| refused("no such shape, attitude or size"))?;
    done()
}

#[derive(Deserialize)]
struct MapParams {
    #[serde(default = "yes")]
    open: bool,
    #[serde(default)]
    dx: i64,
    #[serde(default)]
    dy: i64,
    zoom: Option<f32>,
}

/// `client.map { "dx": 0, "dy": 0, "zoom": 2 }`: opens the full map this far
/// from the player, without the limits on dragging (`HOW_DEBUG_MAP`);
/// `{ "open": false }` closes it.
fn map(
    In(request): In<Option<Value>>,
    player: Single<&MapPosition, With<Player>>,
    mut map: ResMut<FullMap>,
) -> BrpResult {
    let MapParams { open, dx, dy, zoom } = params(request)?;
    if open {
        open_map(&mut map, &player, (dx, dy), zoom);
    } else {
        map.open = false;
    }
    done()
}

#[derive(Deserialize)]
struct FloorParams {
    /// An index among the known floors, from 0; none for the player's own.
    floor: Option<u32>,
}

/// `client.show_floor { "floor": 1 }`: shows that known floor on the full
/// map, which must be open (`HOW_DEBUG_SHOW_FLOOR`).
fn show_floor(In(request): In<Option<Value>>, mut levels: ResMut<OtherLevels>) -> BrpResult {
    let FloorParams { floor } = params(request)?;
    match floor {
        Some(floor) => levels.show(floor),
        None => levels.show_own(),
    }
    Ok(json!({ "showing": levels.showing() }))
}

#[derive(Deserialize)]
struct ActionParams {
    /// As the menu shows it, in any case: `"God Mode"`, `"Truesight"`, ...
    name: String,
}

/// `client.debug_action { "name": "God Mode" }`: as choosing the entry in
/// the debug menu (`HOW_DEBUG_PANEL` for the panels). It isn't checked for
/// being available, but the server still refuses what it doesn't allow.
fn debug_action(
    In(request): In<Option<Value>>,
    mut actions: MessageWriter<DebugAction>,
) -> BrpResult {
    let ActionParams { name } = params(request)?;
    let action = DebugAction::named(&name).ok_or_else(|| {
        let names: Vec<&str> = DebugAction::names().collect();
        refused(format!("no entry {name:?}; there are {}", names.join(", ")))
    })?;
    actions.write(action);
    done()
}

#[derive(Deserialize)]
struct MenuParams {
    #[serde(default = "yes")]
    open: bool,
    #[serde(default)]
    filter: String,
}

/// `client.debug_menu { "filter": "tele" }`: opens it with that typed
/// (`HOW_DEBUG_MENU`); `{ "open": false }` closes it.
fn debug_menu(In(request): In<Option<Value>>, mut menu: ResMut<DebugMenu>) -> BrpResult {
    let MenuParams { open, filter } = params(request)?;
    if open {
        *menu = DebugMenu::open_with(filter);
    } else {
        menu.open = false;
    }
    done()
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SystemMenuShown {
    Closed,
    Menu,
    Quit,
    Settings,
}

#[derive(Deserialize)]
struct SystemMenuParams {
    show: SystemMenuShown,
}

/// `client.system_menu { "show": "settings" }`: `closed`, `menu`, `quit` (its
/// requester) or `settings` (`HOW_DEBUG_SYSTEM_MENU`).
fn system_menu(In(request): In<Option<Value>>, mut menu: ResMut<SystemMenu>) -> BrpResult {
    let SystemMenuParams { show } = params(request)?;
    *menu = match show {
        SystemMenuShown::Closed => SystemMenu::Closed,
        SystemMenuShown::Menu => SystemMenu::Menu,
        SystemMenuShown::Quit => SystemMenu::Confirming,
        SystemMenuShown::Settings => SystemMenu::Settings,
    };
    done()
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum InventoryShown {
    /// The Character panel.
    Pockets,
    /// Every bag in the pockets.
    Bags,
    /// Holds the item at `index` in the pockets.
    Hold,
    /// All but the first of the pockets onto the ground.
    DropSome,
}

#[derive(Deserialize)]
struct InventoryParams {
    show: InventoryShown,
    #[serde(default)]
    index: usize,
}

/// `client.inventory { "show": "pockets" }`: `pockets`, `bags`, `hold` (with
/// `index`) or `drop_some`, as `HOW_DEBUG_INVENTORY` does.
fn inventory(
    In(request): In<Option<Value>>,
    mut debug: MessageWriter<InventoryDebug>,
) -> BrpResult {
    let InventoryParams { show, index } = params(request)?;
    debug.write(match show {
        InventoryShown::Pockets => InventoryDebug::OpenPockets,
        InventoryShown::Bags => InventoryDebug::OpenBags,
        InventoryShown::Hold => InventoryDebug::PickUp(index),
        InventoryShown::DropSome => InventoryDebug::DropSome,
    });
    done()
}
