//! The player-controlled character: keyboard steps and travelling to a
//! clicked point. See `docs/context/client/player.md`.

use bevy::math::DVec2;
use bevy::prelude::*;

use crate::common::{Icon, Token};

/// While a key is held, how often the player takes another step.
const STEP_REPEAT_SECONDS: f32 = 0.05;

/// Travel speed towards a clicked point, in grid units per second.
const TRAVEL_SPEED: f64 = 60.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StepRepeat(Timer::from_seconds(
            STEP_REPEAT_SECONDS,
            TimerMode::Repeating,
        )))
        .add_message::<TravelTo>()
        .configure_sets(
            Update,
            (PlayerSystems::Input, PlayerSystems::Travel).chain(),
        )
        .add_systems(Startup, spawn_player)
        .add_systems(Update, move_player.in_set(PlayerSystems::Input))
        .add_systems(Update, travel.in_set(PlayerSystems::Travel));
    }
}

/// Ordering for anything that moves the player. Put systems that change
/// `MapPosition` or send `TravelTo` in `Input`; systems that draw the player's
/// position should run `.after(PlayerSystems::Travel)`.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlayerSystems {
    Input,
    Travel,
}

/// Marks the entity the keyboard controls.
#[derive(Component)]
pub struct Player;

/// Position on the world grid (1 unit is about 1 km). `y` increases southwards
/// (towards the viewer on the world map). While travelling, this is the
/// nearest grid point to `SmoothPosition`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapPosition {
    pub x: i64,
    pub y: i64,
}

/// The exact position, fractional while travelling. Equal to `MapPosition`
/// whenever the player is standing still. Read it for smooth drawing; change
/// `MapPosition` or send `TravelTo` to move.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct SmoothPosition {
    pub x: f64,
    pub y: f64,
}

/// Ask the player to travel to a grid point at constant speed. A new request
/// replaces the current one, starting from wherever the player is.
#[derive(Message, Clone, Copy, Debug)]
pub struct TravelTo {
    pub x: i64,
    pub y: i64,
}

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
    ));
}

/// Steps one grid unit when a key is pressed, then repeats while it's held.
fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut repeat: ResMut<StepRepeat>,
    mut player: Single<&mut MapPosition, With<Player>>,
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
        player.x += dx;
        player.y += dy;
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
/// on it. A direct change to `MapPosition` (keyboard, demo walk, dev tools)
/// cancels any travel and jumps there.
fn travel(
    time: Res<Time>,
    mut requests: MessageReader<TravelTo>,
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
    let step = TRAVEL_SPEED * time.delta_secs_f64();
    let now = if from.distance(target) <= step {
        // Arrived: land exactly on the grid point.
        travel.target = None;
        target
    } else {
        from + (target - from).normalize() * step
    };

    *smooth = SmoothPosition { x: now.x, y: now.y };
    let nearest = MapPosition {
        x: now.x.round() as i64,
        y: now.y.round() as i64,
    };
    tile.set_if_neq(nearest);
    travel.last = nearest;
}
