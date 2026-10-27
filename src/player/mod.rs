//! The player-controlled character. See `docs/context/player.md`.

use bevy::prelude::*;

/// How fast the player moves, in pixels per second.
const PLAYER_SPEED: f32 = 300.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, move_player);
    }
}

/// Marks the entity the keyboard controls.
#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.3, 0.7, 0.9), Vec2::splat(48.0)),
        Transform::default(),
    ));
}

/// Runs every frame: moves the player based on the keys held down.
fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player: Single<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction.x -= 1.0;
    }
    if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction.x += 1.0;
    }
    if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction.y += 1.0;
    }
    if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction.y -= 1.0;
    }

    let delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
    player.translation += delta.extend(0.0);
}
