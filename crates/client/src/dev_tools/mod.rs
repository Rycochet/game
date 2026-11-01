//! Developer helpers, controlled by environment variables. Native only; does
//! nothing unless a variable is set. See `docs/context/client/dev_tools.md`.
//! Starting positions and states are applied once connected to the server.
//!
//! - `HOW_DEBUG_POS=x,y`: starting grid position on the world map (the embedded
//!   server moves the player there).
//! - `HOW_DEBUG_ENTER=kind[,seed[,level]]`: start inside a made-up site at the
//!   starting position (the embedded server allows it): `kind` is an index
//!   into `PoiKind::ALL`, `seed` its map seed (default 1), `level` the
//!   level (default 0, the way in).
//! - `HOW_DEBUG_CELLS=x,y,kind;...`: on arriving in a site, change cells of the
//!   level, from where the player stands (`kind`: `floor`, `wall`, `door`,
//!   `window`, `water` or `lava`), to look at walls and liquids the generators
//!   don't make.
//! - `HOW_DEBUG_MOVE=dx,dy`: two seconds after starting, step the player this far
//!   (e.g. off the stairs they arrived on, to see them).
//! - `HOW_DEBUG_STAIRS=n`: draw all stairs as this style (an index into
//!   `StairsStyle::ALL`).
//! - `HOW_DEBUG_FIELDS=n`: draw every field as this stage (its `variety`: for crops
//!   0 ploughed, 1 grass, 2 short, 3 tall, 4 ripe).
//! - `HOW_DEBUG_HERBS=n`: draw every herb tile `n` stages (game days) further on.
//! - `HOW_DEBUG_TRAVEL=x,y`: start travelling to a grid position (as if clicked),
//!   once at `HOW_DEBUG_POS` if set, otherwise two seconds after starting.
//! - `HOW_DEBUG_WALK=true`: start with auto-walk on (as if G was pressed).
//! - `HOW_DEBUG_SNOW=n`: start with the snow line at `n` (128 normal, 0 frozen).
//! - `HOW_DEBUG_TIME=n`: once connected, ask the server to move the clock to `n` game
//!   hours since the world began.
//! - `HOW_DEBUG_CELESTIAL=n`: the suns and moons of terrain seed `n`, not this world's.
//! - `HOW_DEBUG_WINDOW_SIZE=w,h`: resize the window (logical pixels) a second after
//!   starting, to check that views handle resizing.
//! - `HOW_DEBUG_TOKEN=shape,attitude[,completed[,size]]`: the player's token, as
//!   numbers (indexes into `TokenShape::ALL` and `Attitude::ALL`), e.g. `3,2`
//!   is a blue hexagon; a third value of `1` makes it completed, a fourth is
//!   an index into `TokenSize::ALL` (`0` tiny ... `5` titan).
//! - `HOW_DEBUG_VIEW=yaw[,pitch[,zoom]]`: start with the map view turned by `yaw`
//!   degrees (positive moves the camera towards the east; 90 looks west, 180
//!   looks south), tilted to `pitch` degrees above the horizon and zoomed in
//!   (the camera distance is divided by `zoom`; 1.6667 is the wheel's
//!   closest).
//! - `HOW_DEBUG_MAP=dx,dy`: two seconds after starting, open the full map, moved
//!   this many grid points from the player (`0,0` to centre it on them).
//! - `HOW_DEBUG_MAP_ZOOM=n`: the full map opened by `HOW_DEBUG_MAP` at this zoom.
//! - `HOW_DEBUG_PANEL=1`: open the Show Widgets panel; `suns`: Suns and Moons;
//!   `generate`: Generate POI's submenu.
//! - `HOW_DEBUG_MENU=text`: start with the debug menu open, filtered by `text`.
//! - `HOW_DEBUG_SYSTEM_MENU=menu`: start with the system menu open; `quit`: at
//!   its requester; `settings`: at its settings.
//! - `HOW_DEBUG_FIRST_FLOOR=n`: number floors from this (0 or 1), whatever the
//!   locale says.
//! - `HOW_DEBUG_SHOW_FLOOR=n`: three seconds after starting, look at this known
//!   floor (an index, from 0) on the maps.
//! - `HOW_DEBUG_CURSOR=x,y`: hold the mouse cursor here (logical pixels), to look
//!   at what hovering shows.
//! - `HOW_DEBUG_SCREENSHOT=path.png`: save a screenshot after `HOW_DEBUG_SCREENSHOT_DELAY`
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
use crate::system_menu::SystemMenu;
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

        if let Some((x, y)) = env::var("HOW_DEBUG_POS")
            .ok()
            .and_then(|pos| parse_pos(&pos))
        {
            app.add_systems(
                OnExit(GameView::Connecting),
                (move |mut to_server: MessageWriter<ToServer>| {
                    to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
                })
                .in_set(Start::Place),
            );
        }

        if server::option_on("HOW_DEBUG_LOADING_HOLD", false) {
            app.insert_resource(crate::loading_screen::HoldLoadingScreen(true));
        }

        if server::option_on("HOW_DEBUG_REVEAL_POIS", false) {
            app.add_systems(
                OnExit(GameView::Connecting),
                (|mut to_server: MessageWriter<ToServer>| {
                    to_server.write(ToServer(ClientMessage::DevRevealPois));
                })
                .in_set(Start::Enter),
            );
        }

        if let Ok(enter) = env::var("HOW_DEBUG_ENTER") {
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
                    // Where the player is (after `HOW_DEBUG_POS`), as its way out.
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

        if let Ok(cells) = env::var("HOW_DEBUG_CELLS") {
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

        if let Some(style) = env::var("HOW_DEBUG_STAIRS")
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

        if let Some(variety) = env::var("HOW_DEBUG_FIELDS")
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

        if let Some(stages) = env::var("HOW_DEBUG_HERBS")
            .ok()
            .and_then(|n| n.trim().parse::<u32>().ok())
        {
            app.insert_resource(crate::common::HerbGrowthShift(stages));
        }

        if let Ok(panel) = env::var("HOW_DEBUG_PANEL") {
            let action = if panel == "suns" {
                DebugAction::SunsAndMoons
            } else if panel == "generate" {
                DebugAction::GeneratePoi
            } else {
                DebugAction::ShowWidgets
            };
            app.add_systems(
                Update,
                move |mut actions: MessageWriter<DebugAction>, mut sent: Local<bool>| {
                    if !*sent {
                        *sent = true;
                        actions.write(action);
                    }
                },
            );
        }

        if let Ok(filter) = env::var("HOW_DEBUG_MENU") {
            app.insert_resource(DebugMenu::open_with(filter));
        }

        match env::var("HOW_DEBUG_SYSTEM_MENU").as_deref() {
            Ok("menu") => app.insert_resource(SystemMenu::Menu),
            Ok("quit") => app.insert_resource(SystemMenu::Confirming),
            Ok("settings") => app.insert_resource(SystemMenu::Settings),
            _ => app,
        };

        if let Some((x, y)) = env::var("HOW_DEBUG_CURSOR")
            .ok()
            .and_then(|pos| parse_pos(&pos))
        {
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

        if let Some(first) = env::var("HOW_DEBUG_FIRST_FLOOR")
            .ok()
            .and_then(|n| n.parse().ok())
        {
            app.insert_resource(FirstFloor(first));
        }

        if let Some(floor) = env::var("HOW_DEBUG_SHOW_FLOOR")
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

        if let Some((dx, dy)) = env::var("HOW_DEBUG_MAP")
            .ok()
            .and_then(|pos| parse_pos(&pos))
        {
            let zoom = env::var("HOW_DEBUG_MAP_ZOOM")
                .ok()
                .and_then(|n| n.parse().ok());
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

        if let Some((dx, dy)) = env::var("HOW_DEBUG_MOVE")
            .ok()
            .and_then(|pos| parse_pos(&pos))
        {
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

        if let Some((x, y)) = env::var("HOW_DEBUG_TRAVEL")
            .ok()
            .and_then(|pos| parse_pos(&pos))
        {
            let start = env::var("HOW_DEBUG_POS")
                .ok()
                .and_then(|pos| parse_pos(&pos));
            app.add_systems(
                Update,
                (move |time: Res<Time>,
                       mut done: Local<bool>,
                       player: Single<&MapPosition, With<Player>>,
                       mut travel: MessageWriter<TravelTo>| {
                    // Not before the server has moved the player to `HOW_DEBUG_POS`
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

        if server::option_on("HOW_DEBUG_WALK", false) {
            app.insert_resource(AutoWalk(true));
        }

        if let Some((width, height)) = env::var("HOW_DEBUG_WINDOW_SIZE")
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

        if let Some(hours) = env::var("HOW_DEBUG_TIME")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
        {
            // Asked of the server once there is one to ask.
            app.add_systems(
                Update,
                move |view: Res<State<GameView>>,
                      mut done: Local<bool>,
                      mut to_server: MessageWriter<ToServer>| {
                    if !*done && *view.get() != GameView::Connecting {
                        *done = true;
                        to_server.write(ToServer(ClientMessage::DevSetTime {
                            minutes: hours * shared::world::MINUTES_PER_HOUR,
                        }));
                    }
                },
            );
        }

        if let Some(seed) = env::var("HOW_DEBUG_CELESTIAL")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            app.add_systems(
                OnExit(GameView::Connecting),
                move |mut world: ResMut<crate::common::CurrentWorld>| {
                    let seed = shared::rng::Seed::new(seed);
                    world.celestial = shared::celestial::Celestial::from_seed(seed);
                },
            );
        }

        if let Some(snow_line) = env::var("HOW_DEBUG_SNOW").ok().and_then(|v| v.parse().ok()) {
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

        if let Some(new_token) = env::var("HOW_DEBUG_TOKEN")
            .ok()
            .and_then(|t| parse_token(&t))
        {
            app.add_systems(
                OnExit(GameView::Connecting),
                move |mut token: Single<&mut Token, With<Player>>| {
                    **token = new_token;
                },
            );
        }

        if let Ok(view) = env::var("HOW_DEBUG_VIEW") {
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
}

#[derive(Resource)]
struct ScreenshotPlan {
    path: String,
    at: f32,
    taken: bool,
}

fn screenshot_then_quit(
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

/// `dx,dy,kind`, as in `HOW_DEBUG_CELLS`.
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
        "covered" => Cell::CoveredGround,
        _ => return None,
    };
    Some((dx, dy, cell))
}

fn parse_pos(text: &str) -> Option<(i64, i64)> {
    let (x, y) = text.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
