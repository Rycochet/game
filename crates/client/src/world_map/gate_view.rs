//! How a World Gate's token stands on the world map: turned to face the
//! camera whichever way the view is turned, with a white light at its apex
//! that makes it stand out at night.

use bevy::prelude::*;

use crate::view3d::{OrbitCamera, ViewCamera};

/// How far a gate's light reaches, in grid points: about a chunk
/// (`shared::poi::CHUNK_SIZE`).
const LIGHT_RANGE: f32 = 64.0;

/// How bright a gate's light is. Tuned against the night's ambient fill, so the
/// ground round a gate glows at night; the sun drowns it out by day.
const LIGHT_LUMENS: f32 = 100_000_000.0;

/// On the part of a gate's token (its base, model, outline, water and light)
/// that turns to face the camera.
#[derive(Component)]
pub(super) struct GateTurn;

/// Spawns the turning part of a gate's token as a child of its view, with
/// the light `apex` above the view's centre, and returns it for the rest to
/// be put on.
pub(super) fn spawn_turn(commands: &mut Commands, view: Entity, apex: f32) -> Entity {
    let turn = commands
        .spawn((GateTurn, Transform::default(), Visibility::Inherited))
        .with_child((
            PointLight {
                color: Color::WHITE,
                intensity: LIGHT_LUMENS,
                range: LIGHT_RANGE,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, apex, 0.0),
        ))
        .id();
    commands.entity(view).add_child(turn);
    turn
}

/// Every frame, after tokens are placed: turns each gate as far as the camera
/// is turned, so its front always faces the screen.
pub(super) fn face_camera(
    camera: Single<&OrbitCamera, With<ViewCamera>>,
    mut gates: Query<&mut Transform, With<GateTurn>>,
) {
    let rotation = Quat::from_rotation_y(camera.yaw);
    for mut transform in &mut gates {
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
}
