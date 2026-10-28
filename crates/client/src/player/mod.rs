//! The player-controlled character. See `docs/context/client/player.md`.

use bevy::prelude::*;

/// While a key is held, how often the player takes another step.
const STEP_REPEAT_SECONDS: f32 = 0.05;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StepRepeat(Timer::from_seconds(
            STEP_REPEAT_SECONDS,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, spawn_player)
        .add_systems(Update, move_player);
    }
}

/// Marks the entity the keyboard controls.
#[derive(Component)]
pub struct Player;

/// Position on the world grid (1 unit is about 1 km). `y` increases southwards
/// (towards the viewer on the world map).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapPosition {
    pub x: i64,
    pub y: i64,
}

#[derive(Resource)]
struct StepRepeat(Timer);

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, MapPosition::default()));
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
