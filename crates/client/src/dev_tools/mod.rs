//! Developer helpers, controlled by environment variables. Native only; does
//! nothing unless a variable is set. See `docs/context/client/dev_tools.md`.
//! Starting positions and states are applied once connected to the server.
//!
//! - `GAME_POS=x,y`: starting grid position on the world map (the embedded
//!   server moves the player there).
//! - `GAME_ENTER=kind[,seed[,level]]`: start inside a made-up site at the
//!   starting position (the embedded server allows it): `kind` is an index
//!   into `PoiKind::ALL`, `seed` its map seed (default 1), `level` the
//!   level (default 0, the way in).
//! - `GAME_CELLS=x,y,kind;...`: on arriving in a site, change cells of the
//!   level, from where the player stands (`kind`: `floor`, `wall`, `door`,
//!   `window`, `water` or `lava`), to look at walls and liquids the generators
//!   don't make.
//! - `GAME_MOVE=dx,dy`: two seconds after starting, step the player this far
//!   (e.g. off the stairs they arrived on, to see them).
//! - `GAME_STAIRS=n`: draw all stairs as this style (an index into
//!   `StairsStyle::ALL`).
//! - `GAME_FIELDS=n`: draw every field as this stage (its `variety`: for crops
//!   0 ploughed, 1 grass, 2 short, 3 tall, 4 ripe).
//! - `GAME_HERBS=n`: draw every herb tile `n` stages (game days) further on.
//! - `GAME_TRAVEL=x,y`: start travelling to a grid position (as if clicked),
//!   once at `GAME_POS` if set, otherwise two seconds after starting.
//! - `GAME_WALK=1`: start with auto-walk on (as if G was pressed).
//! - `GAME_SNOW=n`: start with the snow line at `n` (128 normal, 0 frozen).
//! - `GAME_SKY_HOUR=n`: draw the weather this many game hours ahead.
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
//! - `GAME_MAP=dx,dy`: two seconds after starting, open the full map, moved
//!   this many grid points from the player (`0,0` to centre it on them).
//! - `GAME_MAP_ZOOM=n`: the full map opened by `GAME_MAP` at this zoom.
//! - `GAME_PANEL=1`: open the Show Widgets panel.
//! - `GAME_DEBUG_MENU=text`: start with the debug menu open, filtered by `text`.
//! - `GAME_FIRST_FLOOR=n`: number floors from this (0 or 1), whatever the
//!   locale says.
//! - `GAME_SHOW_FLOOR=n`: three seconds after starting, look at this known
//!   floor (an index, from 0) on the maps.
//! - `GAME_CURSOR=x,y`: hold the mouse cursor here (logical pixels), to look
//!   at what hovering shows.
//! - `GAME_SCREENSHOT=path.png`: save a screenshot after `GAME_SCREENSHOT_DELAY`
//!   seconds (default 5), then quit. Lets tools check visuals without a person.

use std::env;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::PrimaryWindow;

use shared::dungeon::Cell;
use shared::movement::steps_between;
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;
use shared::site::StairsStyle;

use crate::common::{
    Attitude, CurrentDungeon, FirstFloor, GameView, MapPosition, OtherLevels, Token, TokenShape,
    TokenSize,
};
use crate::connection::ToServer;
use crate::debug_menu::{AutoWalk, DebugAction, DebugMenu};
use crate::minimap::FullMap;
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

        if env::var("GAME_REVEAL_POIS").is_ok() {
            app.add_systems(
                OnExit(GameView::Connecting),
                (|mut to_server: MessageWriter<ToServer>| {
                    to_server.write(ToServer(ClientMessage::DevRevealPois));
                })
                .in_set(Start::Enter),
            );
        }

        if let Ok(enter) = env::var("GAME_ENTER") {
            let numbers: Vec<u64> = enter
                .split(',')
                .filter_map(|n| n.trim().parse().ok())
                .collect();
            let kind = numbers
                .first()
                .and_then(|&k| PoiKind::ALL.get(k as usize).copied())
                .unwrap_or(PoiKind::Castle);
            let seed = numbers.get(1).copied().unwrap_or(1);
            let level = numbers.get(2).copied().unwrap_or(0) as u32;
            let ruin = numbers.get(3).copied().unwrap_or(0) as u8;
            app.add_systems(
                OnExit(GameView::Connecting),
                (move |mut to_server: MessageWriter<ToServer>| {
                    // Where the player is (after `GAME_POS`), as its way out.
                    to_server.write(ToServer(ClientMessage::DevEnterSite {
                        kind,
                        seed,
                        level,
                        ruin,
                    }));
                })
                .in_set(Start::Enter),
            );
        }

        if let Ok(cells) = env::var("GAME_CELLS") {
            let cells: Vec<(i64, i64, Cell)> = cells.split(';').filter_map(parse_cell).collect();
            app.add_systems(
                Update,
                move |view: Res<State<GameView>>,
                      mut done: Local<bool>,
                      player: Single<&MapPosition, With<Player>>,
                      mut to_server: MessageWriter<ToServer>| {
                    if !*done && view.get() == &GameView::Dungeon {
                        *done = true;
                        for &(dx, dy, cell) in &cells {
                            let (x, y) = (player.x + dx, player.y + dy);
                            to_server.write(ToServer(ClientMessage::DevSetCell { x, y, cell }));
                        }
                    }
                },
            );
        }

        if let Some(style) = env::var("GAME_STAIRS")
            .ok()
            .and_then(|n| n.trim().parse::<usize>().ok())
            .and_then(|n| StairsStyle::ALL.get(n).copied())
        {
            app.add_systems(Update, move |mut dungeon: ResMut<CurrentDungeon>| {
                if dungeon.stairs.iter().any(|stairs| stairs.style != style) {
                    for stairs in &mut dungeon.stairs {
                        stairs.style = style;
                    }
                }
            });
        }

        if let Some(variety) = env::var("GAME_FIELDS")
            .ok()
            .and_then(|n| n.trim().parse::<u32>().ok())
        {
            app.add_systems(Update, move |mut dungeon: ResMut<CurrentDungeon>| {
                if dungeon.fields.iter().any(|f| f.variety != variety) {
                    for field in &mut dungeon.fields {
                        field.variety = variety;
                    }
                }
            });
        }

        if let Some(stages) = env::var("GAME_HERBS")
            .ok()
            .and_then(|n| n.trim().parse::<u32>().ok())
        {
            app.insert_resource(crate::common::HerbGrowthShift(stages));
        }

        if env::var("GAME_PANEL").is_ok() {
            app.add_systems(
                Update,
                |mut actions: MessageWriter<DebugAction>, mut sent: Local<bool>| {
                    if !*sent {
                        *sent = true;
                        actions.write(DebugAction::ShowWidgets);
                    }
                },
            );
        }

        if let Ok(filter) = env::var("GAME_DEBUG_MENU") {
            app.insert_resource(DebugMenu::open_with(filter));
        }

        if let Some((x, y)) = env::var("GAME_CURSOR").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(
                Update,
                move |mut window: Single<&mut Window, With<PrimaryWindow>>| {
                    let at = Vec2::new(x as f32, y as f32);
                    if window.cursor_position() != Some(at) {
                        window.set_cursor_position(Some(at));
                    }
                },
            );
        }

        if let Some(first) = env::var("GAME_FIRST_FLOOR")
            .ok()
            .and_then(|n| n.parse().ok())
        {
            app.insert_resource(FirstFloor(first));
        }

        if let Some(floor) = env::var("GAME_SHOW_FLOOR")
            .ok()
            .and_then(|n| n.parse().ok())
        {
            app.add_systems(
                Update,
                move |time: Res<Time>, mut done: Local<bool>, mut levels: ResMut<OtherLevels>| {
                    if !*done && time.elapsed_secs() > 3.0 {
                        *done = true;
                        levels.show(floor);
                    }
                },
            );
        }

        if let Some((dx, dy)) = env::var("GAME_MAP").ok().and_then(|pos| parse_pos(&pos)) {
            let zoom = env::var("GAME_MAP_ZOOM").ok().and_then(|n| n.parse().ok());
            app.add_systems(
                Update,
                move |time: Res<Time>,
                      mut done: Local<bool>,
                      player: Single<&MapPosition, With<Player>>,
                      mut map: ResMut<FullMap>| {
                    if !*done && time.elapsed_secs() > 2.0 {
                        *done = true;
                        map.open = true;
                        map.centre = Some(((player.x + dx) as f64, (player.y + dy) as f64));
                        if let Some(zoom) = zoom {
                            map.zoom = zoom;
                        }
                    }
                },
            );
        }

        if let Some((dx, dy)) = env::var("GAME_MOVE").ok().and_then(|pos| parse_pos(&pos)) {
            app.add_systems(
                Update,
                (move |time: Res<Time>,
                       mut done: Local<bool>,
                       mut steps: MessageWriter<TakeStep>| {
                    if !*done && time.elapsed_secs() > 2.0 {
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
                (move |time: Res<Time>,
                       mut done: Local<bool>,
                       player: Single<&MapPosition, With<Player>>,
                       mut travel: MessageWriter<TravelTo>| {
                    // Not before the server has moved the player to `GAME_POS`
                    // or back into their site: that would end the journey.
                    let ready = match start {
                        Some((sx, sy)) => (player.x, player.y) == (sx, sy),
                        None => time.elapsed_secs() > 2.0,
                    };
                    if !*done && ready {
                        *done = true;
                        travel.write(TravelTo { x, y });
                    }
                })
                .in_set(PlayerSystems::Input)
                .run_if(not(in_state(GameView::Connecting))),
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

        if let Some(hours) = env::var("GAME_SKY_HOUR").ok().and_then(|v| v.parse().ok()) {
            app.insert_resource(crate::world_map::sky::SkyHourOffset(hours));
        }

        if let Some(snow_line) = env::var("GAME_SNOW").ok().and_then(|v| v.parse().ok()) {
            // Asked of the server once there is one to ask.
            app.add_systems(
                Update,
                move |view: Res<State<GameView>>,
                      mut done: Local<bool>,
                      mut to_server: MessageWriter<ToServer>| {
                    if !*done && *view.get() != GameView::Connecting {
                        *done = true;
                        to_server.write(ToServer(ClientMessage::DevSnowLine(snow_line)));
                    }
                },
            );
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
        ..default()
    })
}

/// `dx,dy,kind`, as in `GAME_CELLS`.
fn parse_cell(text: &str) -> Option<(i64, i64, Cell)> {
    let (position, kind) = text.rsplit_once(',')?;
    let (dx, dy) = parse_pos(position)?;
    let cell = match kind.trim() {
        "floor" => Cell::Floor,
        "wall" => Cell::Wall,
        "door" => Cell::Door,
        "window" => Cell::Window,
        "water" => Cell::Water,
        "lava" => Cell::Lava,
        "hole" => Cell::Hole,
        "ground" => Cell::Ground,
        _ => return None,
    };
    Some((dx, dy, cell))
}

fn parse_pos(text: &str) -> Option<(i64, i64)> {
    let (x, y) = text.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
