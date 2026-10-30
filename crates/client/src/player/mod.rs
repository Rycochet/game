//! The player-controlled character: keyboard steps and travelling to a
//! clicked point. See `docs/context/client/player.md`.
//!
//! Every step the player takes is announced (`Stepped`), for `connection` to
//! send to the server, which checks it: what is shown here is a prediction
//! the server may put right.

use bevy::math::DVec2;
use bevy::prelude::*;
use shared::movement::{Step, step_on_level, steps_between};

use crate::common::{
    CurrentDungeon, GameView, Icon, LightSource, MapPosition, ServerStatus, SmoothPosition, Token,
    ViewFacing,
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

/// Ask the player to travel to a grid point at constant speed. A new request
/// replaces the current one, starting from wherever the player is.
#[derive(Message, Clone, Copy, Debug)]
pub struct TravelTo {
    pub x: i64,
    pub y: i64,
}

/// The player has finished travelling to a grid point (sent once, when it
/// gets there). Not sent for keyboard steps or other direct moves.
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
    target: Option<DVec2>,
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
/// walkable cells the client knows of, or over liquid to one (the server's
/// rule, `shared::movement`). Steps are ignored until a jump has landed.
fn take_steps(
    mut requests: MessageReader<TakeStep>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
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
            _ => Some(step.from(from)),
        };
        let Some((x, y)) = to else {
            continue;
        };
        *position = MapPosition { x, y };
        stepped.write(Stepped::between(step, from, (x, y)));
        if (x, y) != step.from(from) {
            // A jump replaces any journey, as a step does, and is shown in
            // `travel` instead of jumping there at once.
            travel.target = None;
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

/// Moves towards the requested point at constant speed, always ending exactly
/// on it, announcing each step. Any other change to `MapPosition` (a step,
/// the server) cancels travel and jumps there. Walls don't stop it (TEMPORARY
/// until pathfinding: the server refuses the step into the wall, and puts
/// the player back, which ends the journey). Reaching the edge of a liquid
/// cell starts a jump over it if it can be crossed, and ends the journey there
/// if not. A jump is shown from here too: straight to the cell it lands on,
/// whatever is asked for meanwhile. A journey carries on from that cell's
/// edge; otherwise the jump ends on its centre.
#[allow(clippy::too_many_arguments)]
fn travel(
    time: Res<Time>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut requests: MessageReader<TravelTo>,
    mut arrived: MessageWriter<Arrived>,
    mut stepped: MessageWriter<Stepped>,
    player: Single<(&mut MapPosition, &mut SmoothPosition, &mut Travel), With<Player>>,
) {
    let (mut tile, mut smooth, mut travel) = player.into_inner();

    if *tile != travel.last {
        travel.target = None;
        travel.jump = None;
        travel.last = *tile;
        *smooth = SmoothPosition {
            x: tile.x as f64,
            y: tile.y as f64,
        };
    }

    if let Some(request) = requests.read().last() {
        travel.target = Some(DVec2::new(request.x as f64, request.y as f64));
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
        if now == to || (travel.target.is_some() && over_its_edge) {
            travel.jump = None;
        }
        return;
    }

    let Some(target) = travel.target else {
        return;
    };
    let now = towards(from, target, distance);
    let nearest = nearest_cell(now);
    // Normally one step a frame, but a slow frame may cover more.
    let mut at = (tile.x, tile.y);
    for step in steps_between(at, (nearest.x, nearest.y)) {
        let next = step.from(at);
        let liquid =
            *view.get() == GameView::Dungeon && dungeon.level.cell(next.0, next.1).is_liquid();
        if !liquid {
            stepped.write(Stepped::between(step, at, next));
            at = next;
            continue;
        }
        // The edge of the liquid: the journey can't go on the way it was.
        if let Some(landing) = step_on_level(&dungeon.level, at, step) {
            stepped.write(Stepped::between(step, at, landing));
            at = landing;
            travel.jump = Some(MapPosition {
                x: landing.0,
                y: landing.1,
            });
        } else {
            travel.target = None;
            *smooth = SmoothPosition {
                x: at.0 as f64,
                y: at.1 as f64,
            };
        }
        let stopped_at = MapPosition { x: at.0, y: at.1 };
        tile.set_if_neq(stopped_at);
        travel.last = stopped_at;
        return;
    }

    if now == target {
        // Arrived: exactly on the grid point.
        travel.target = None;
        arrived.write(Arrived {
            x: target.x as i64,
            y: target.y as i64,
        });
    }
    *smooth = SmoothPosition { x: now.x, y: now.y };
    tile.set_if_neq(nearest);
    travel.last = nearest;
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
