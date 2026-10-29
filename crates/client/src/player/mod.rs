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
    CurrentDungeon, GameView, Icon, LightSource, MapPosition, SmoothPosition, Token, ViewFacing,
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
                .run_if(not(in_state(GameView::Connecting))),
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
/// Not sent when the position is set from elsewhere (the server's word).
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stepped(pub Step);

/// Travel state, private to this module.
#[derive(Component, Default)]
struct Travel {
    target: Option<DVec2>,
    /// The `MapPosition` this module last set, to notice other changes.
    last: MapPosition,
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
/// walkable cells the client knows of (the server's rule, `shared::movement`).
fn take_steps(
    mut requests: MessageReader<TakeStep>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    mut stepped: MessageWriter<Stepped>,
    mut player: Single<&mut MapPosition, With<Player>>,
) {
    for &TakeStep(step) in requests.read() {
        let from = (player.x, player.y);
        let to = match view.get() {
            GameView::Dungeon => step_on_level(&dungeon.level, from, step),
            _ => Some(step.from(from)),
        };
        if let Some((x, y)) = to {
            **player = MapPosition { x, y };
            stepped.write(Stepped(step));
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
/// the player back, which ends the journey).
fn travel(
    time: Res<Time>,
    view: Res<State<GameView>>,
    mut requests: MessageReader<TravelTo>,
    mut arrived: MessageWriter<Arrived>,
    mut stepped: MessageWriter<Stepped>,
    player: Single<(&mut MapPosition, &mut SmoothPosition, &mut Travel), With<Player>>,
) {
    let (mut tile, mut smooth, mut travel) = player.into_inner();

    if *tile != travel.last {
        travel.target = None;
        travel.last = *tile;
        *smooth = SmoothPosition {
            x: tile.x as f64,
            y: tile.y as f64,
        };
    }

    if let Some(request) = requests.read().last() {
        travel.target = Some(DVec2::new(request.x as f64, request.y as f64));
    }

    let Some(target) = travel.target else {
        return;
    };
    let from = DVec2::new(smooth.x, smooth.y);
    let speed = match view.get() {
        GameView::WorldMap => TRAVEL_SPEED,
        GameView::Dungeon => DUNGEON_TRAVEL_SPEED,
        // Travel doesn't run while connecting.
        GameView::Connecting => return,
    };
    let step = speed * time.delta_secs_f64();
    let now = if from.distance(target) <= step {
        // Arrived: land exactly on the grid point.
        travel.target = None;
        arrived.write(Arrived {
            x: target.x as i64,
            y: target.y as i64,
        });
        target
    } else {
        from + (target - from).normalize() * step
    };

    *smooth = SmoothPosition { x: now.x, y: now.y };
    let nearest = MapPosition {
        x: now.x.round() as i64,
        y: now.y.round() as i64,
    };
    // Normally one step a frame, but a slow frame may cover more.
    for step in steps_between((tile.x, tile.y), (nearest.x, nearest.y)) {
        stepped.write(Stepped(step));
    }
    tile.set_if_neq(nearest);
    travel.last = nearest;
}
