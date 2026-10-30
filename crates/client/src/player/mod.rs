//! The player-controlled character: keyboard steps and travelling to a
//! clicked point. See `docs/context/client/player.md`.
//!
//! Every step the player takes is announced (`Stepped`), for `connection` to
//! send to the server, which checks it: what is shown here is a prediction
//! the server may put right.

use bevy::math::DVec2;
use std::collections::VecDeque;

use bevy::prelude::*;
use shared::movement::{Step, step_cost, step_on_level, step_on_world_map, world_step_cost};
use shared::pathfinding::{find_path, find_world_path, straighten};

use crate::common::{
    CurrentDungeon, CurrentWorld, GameClock, GameView, Icon, LightSource, MapPosition,
    ServerStatus, SmoothPosition, Token, ViewFacing, Weather,
};

/// While a key is held, how often the player takes another step.
const STEP_REPEAT_SECONDS: f32 = 0.05;

/// How far the player's light reaches in dungeons, in cells.
const LIGHT_RADIUS: u32 = 6;

/// Travel speed towards a clicked point, in grid units per second: world map
/// points, or dungeon cells (which are much bigger on screen).
const TRAVEL_SPEED: f64 = 60.0;
const DUNGEON_TRAVEL_SPEED: f64 = 12.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StepRepeat(Timer::from_seconds(
            STEP_REPEAT_SECONDS,
            TimerMode::Repeating,
        )))
        .add_message::<TravelTo>()
        .add_message::<Arrived>()
        .add_message::<TakeStep>()
        .add_message::<Stepped>()
        .configure_sets(
            Update,
            (PlayerSystems::Input, PlayerSystems::Travel).chain(),
        )
        .add_systems(Startup, spawn_player)
        // Nowhere to move until the server has said which world this is.
        .add_systems(
            Update,
            (
                move_player.in_set(PlayerSystems::Input),
                // After everything in `Input` asked for its steps.
                (take_steps, travel).chain().in_set(PlayerSystems::Travel),
            )
                .run_if(not(in_state(GameView::Connecting)))
                // Nobody to tell while the connection is down.
                .run_if(|status: Res<ServerStatus>| status.is_connected()),
        );
    }
}

/// Ordering for anything that moves the player. Put systems that send
/// `TakeStep` or `TravelTo` in `Input`; systems that draw the player's
/// position (or read `Stepped`) should run `.after(PlayerSystems::Travel)`.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlayerSystems {
    Input,
    Travel,
}

/// Marks the entity the keyboard controls.
#[derive(Component)]
pub struct Player;

/// Ask the player to travel to a grid point, by the quickest way, as far
/// towards it as they can get: in a dungeon, through the cells they know; on
/// the world map, round water and easing off on slopes. A new request replaces
/// the current one, starting from wherever the player is.
#[derive(Message, Clone, Copy, Debug)]
pub struct TravelTo {
    pub x: i64,
    pub y: i64,
}

/// The player has finished travelling to a grid point (sent once, when it
/// gets there). Not sent for keyboard steps or other direct moves, nor for a
/// journey that stopped short of where it was asked to go.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrived {
    pub x: i64,
    pub y: i64,
}

/// Ask the player to take a step (keyboard, demo walk, dev tools): taken
/// unless it leads into a wall in a site.
#[derive(Message, Clone, Copy, Debug)]
pub struct TakeStep(pub Step);

/// The player has taken a step, by `TakeStep` or travelling: for the server.
/// `moved` is how far it took them, which is two cells for a jump over liquid.
/// Not sent when the position is set from elsewhere (the server's word).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stepped {
    pub step: Step,
    pub moved: (i64, i64),
}

impl Stepped {
    /// The step from `from` that ended at `to`.
    fn between(step: Step, from: (i64, i64), to: (i64, i64)) -> Self {
        Self {
            step,
            moved: (to.0 - from.0, to.1 - from.1),
        }
    }
}

/// Travel state, private to this module.
#[derive(Component, Default)]
struct Travel {
    /// The points a journey has still to reach, in order.
    route: VecDeque<(i64, i64)>,
    /// Where the journey was asked to go, to find a new route if the tide cuts
    /// this one.
    goal: (i64, i64),
    /// Whether the route ends where the journey was asked to go.
    route_arrives: bool,
    /// The `MapPosition` this module last set, to notice other changes.
    last: MapPosition,
    /// The cell a jump over liquid is landing on. The player is already there
    /// as far as the game goes, and is shown getting there: until they land
    /// they can't take another step or change direction.
    jump: Option<MapPosition>,
}

#[derive(Resource)]
struct StepRepeat(Timer);

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        MapPosition::default(),
        SmoothPosition::default(),
        Travel::default(),
        // A white circle (no attitude): drawn as a cylinder on the map.
        Token::default(),
        Icon("monsters/humanarcher.png".into()),
        LightSource {
            radius: LIGHT_RADIUS,
            colour: Color::WHITE,
        },
    ));
}

/// Steps one grid unit when a key is pressed, then repeats while it's held.
fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    facing: Res<ViewFacing>,
    mut repeat: ResMut<StepRepeat>,
    mut steps: MessageWriter<TakeStep>,
) {
    // A key tapped and released within one frame is "just pressed" but no
    // longer "pressed", so count either or quick taps get lost.
    let active = |a, b| keys.any_pressed([a, b]) || keys.any_just_pressed([a, b]);
    let mut dx = 0;
    let mut dy = 0;
    if active(KeyCode::ArrowLeft, KeyCode::KeyA) {
        dx -= 1;
    }
    if active(KeyCode::ArrowRight, KeyCode::KeyD) {
        dx += 1;
    }
    if active(KeyCode::ArrowUp, KeyCode::KeyW) {
        dy -= 1;
    }
    if active(KeyCode::ArrowDown, KeyCode::KeyS) {
        dy += 1;
    }

    let just_pressed = keys.any_just_pressed(MOVE_KEYS);
    if just_pressed {
        repeat.0.reset();
    }
    repeat.0.tick(time.delta());

    if (dx != 0 || dy != 0) && (just_pressed || repeat.0.just_finished()) {
        // The keys mean screen directions, so follow the way the map is turned.
        let (dx, dy) = facing.screen_to_grid(dx, dy);
        if let Some(step) = Step::new(dx, dy) {
            steps.write(TakeStep(step));
        }
    }
}

/// Takes the steps asked for this frame, in order: in a dungeon, only onto
/// walkable cells the client knows of, or over liquid to one; on the world map,
/// not into open water (the server's rules, `shared::movement`). Steps are
/// ignored until a jump has landed.
#[allow(clippy::too_many_arguments)]
fn take_steps(
    mut requests: MessageReader<TakeStep>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    mut stepped: MessageWriter<Stepped>,
    player: Single<(&mut MapPosition, &mut Travel), With<Player>>,
) {
    let (mut position, mut travel) = player.into_inner();
    for &TakeStep(step) in requests.read() {
        if travel.jump.is_some() {
            continue;
        }
        let from = (position.x, position.y);
        let to = match view.get() {
            GameView::Dungeon => step_on_level(&dungeon.level, from, step),
            _ => world.as_ref().and_then(|world| {
                step_on_world_map(
                    &world.ground,
                    world.conditions(&clock, &weather),
                    from,
                    step,
                )
            }),
        };
        let Some((x, y)) = to else {
            continue;
        };
        *position = MapPosition { x, y };
        stepped.write(Stepped::between(step, from, (x, y)));
        if (x, y) != step.from(from) {
            // A jump replaces any journey, as a step does, and is shown in
            // `travel` instead of jumping there at once.
            travel.route.clear();
            travel.last = *position;
            travel.jump = Some(*position);
        }
    }
}

const MOVE_KEYS: [KeyCode; 8] = [
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::KeyA,
    KeyCode::KeyD,
    KeyCode::KeyW,
    KeyCode::KeyS,
];

/// Moves along the route found when the journey was asked for, always ending
/// exactly on its last point, announcing each step: at constant speed, except
/// that world map slopes are slower, by what they cost. Any other change to
/// `MapPosition` (a step, the server) cancels travel and jumps there. A step
/// is taken on leaving a cell, and a jump is shown from here too: straight to
/// the cell it lands on, whatever is asked for meanwhile. A journey carries on
/// from that cell's edge; otherwise the jump ends on its centre.
#[allow(clippy::too_many_arguments)]
fn travel(
    time: Res<Time>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    mut requests: MessageReader<TravelTo>,
    mut arrived: MessageWriter<Arrived>,
    mut stepped: MessageWriter<Stepped>,
    player: Single<(&mut MapPosition, &mut SmoothPosition, &mut Travel), With<Player>>,
) {
    let (mut tile, mut smooth, mut travel) = player.into_inner();

    // On the world map, the ground and the conditions now; in a dungeon, `None`.
    let ground = (*view.get() == GameView::WorldMap)
        .then_some(world.as_ref())
        .flatten()
        .map(|world| (&world.ground, world.conditions(&clock, &weather)));

    if *tile != travel.last {
        travel.route.clear();
        travel.jump = None;
        travel.last = *tile;
        *smooth = SmoothPosition {
            x: tile.x as f64,
            y: tile.y as f64,
        };
    }

    if let Some(request) = requests.read().last() {
        let from = (tile.x, tile.y);
        let to = (request.x, request.y);
        let path = match ground {
            Some((ground, conditions)) => find_world_path(ground, conditions, from, to),
            None => {
                // Only through cells the player has seen, so that a route gives
                // nothing away (and memory may be out of date: the server checks).
                let known = |x, y| dungeon.remembered.is_seen(x, y);
                let path = find_path(&dungeon.level, from, to, None, known);
                straighten(&dungeon.level, from, &path, known)
            }
        };
        travel.goal = to;
        travel.route_arrives = path.reached;
        travel.route = path.cells.into();
        if travel.route.is_empty() {
            // Nowhere to go: back to the middle of this cell.
            travel.route.push_back(from);
        }
    }

    let from = DVec2::new(smooth.x, smooth.y);
    let speed = match view.get() {
        GameView::WorldMap => TRAVEL_SPEED,
        GameView::Dungeon => DUNGEON_TRAVEL_SPEED,
        // Travel doesn't run while connecting.
        GameView::Connecting => return,
    };
    let distance = speed * time.delta_secs_f64();

    if let Some(landing) = travel.jump {
        let to = DVec2::new(landing.x as f64, landing.y as f64);
        let now = towards(from, to, distance);
        *smooth = SmoothPosition { x: now.x, y: now.y };
        let over_its_edge = nearest_cell(now) == landing;
        if now == to || (!travel.route.is_empty() && over_its_edge) {
            travel.jump = None;
        }
        return;
    }

    if !travel.route.is_empty() {
        let mut now = from;
        let mut left = distance;
        let mut at = (tile.x, tile.y);
        // Normally part of one step a frame, but a slow frame may cover more.
        while let Some(&next) = travel.route.front() {
            let to = DVec2::new(next.0 as f64, next.1 as f64);
            // How much longer this step takes than on flat ground.
            let slowness = match ground {
                Some((ground, conditions)) if next != at => {
                    let ends = (
                        ground.elevation(at.0, at.1, conditions),
                        ground.elevation(next.0, next.1, conditions),
                    );
                    world_step_cost(at, next, ends) as f64 / step_cost(at, next) as f64
                }
                _ => 1.0,
            };
            let moved = towards(now, to, left / slowness);
            left = (left - now.distance(moved) * slowness).max(0.0);
            now = moved;
            let out_of_cell = (now - DVec2::new(at.0 as f64, at.1 as f64))
                .abs()
                .max_element()
                >= 0.5;
            if next != at && out_of_cell {
                let step = Step::new((next.0 - at.0).signum(), (next.1 - at.1).signum())
                    .expect("the next cell isn't this one");
                // The route was legal when it was found, but cells can change.
                let legal = match ground {
                    Some((ground, conditions)) => step_on_world_map(ground, conditions, at, step),
                    None => step_on_level(&dungeon.level, at, step),
                };
                if legal != Some(next) {
                    // The tide or a thaw has cut the way: find another from here.
                    if let Some((ground, conditions)) = ground {
                        let path = find_world_path(ground, conditions, at, travel.goal);
                        travel.route_arrives = path.reached;
                        travel.route = path.cells.into();
                        if travel.route.is_empty() {
                            travel.route.push_back(at);
                        }
                        continue;
                    }
                    travel.route.clear();
                    now = DVec2::new(at.0 as f64, at.1 as f64);
                    break;
                }
                stepped.write(Stepped::between(step, at, next));
                let jumped = next != step.from(at);
                at = next;
                if jumped {
                    travel.jump = Some(MapPosition { x: at.0, y: at.1 });
                    break;
                }
            }
            if now != to {
                break;
            }
            travel.route.pop_front();
            if travel.route.is_empty() && travel.route_arrives {
                arrived.write(Arrived { x: at.0, y: at.1 });
            }
        }
        *smooth = SmoothPosition { x: now.x, y: now.y };
        let reached = MapPosition { x: at.0, y: at.1 };
        tile.set_if_neq(reached);
        travel.last = reached;
    }
}

/// `distance` along the straight line from one point to another, stopping
/// exactly on it.
fn towards(from: DVec2, to: DVec2, distance: f64) -> DVec2 {
    if from.distance(to) <= distance {
        to
    } else {
        from + (to - from).normalize() * distance
    }
}

/// The grid point a position is nearest: its cell, in a dungeon.
fn nearest_cell(position: DVec2) -> MapPosition {
    MapPosition {
        x: position.x.round() as i64,
        y: position.y.round() as i64,
    }
}
