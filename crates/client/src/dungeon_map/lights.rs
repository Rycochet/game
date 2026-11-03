//! Real point lights over each light source, for tokens.

use bevy::prelude::*;

use super::{CELL_SIZE, DungeonScene, TOKEN_LIGHT_HEIGHT, TOKEN_LIGHT_LUMENS, TokenLight};
use crate::common::{LightSource, SmoothPosition};

/// Keeps a real point light over each `LightSource`, so tokens near it are
/// lit like the tiles are.
pub(super) fn sync_lights(
    mut commands: Commands,
    scene: Single<Entity, With<DungeonScene>>,
    sources: Query<(Entity, &LightSource, &SmoothPosition)>,
    mut lights: Query<(Entity, &TokenLight, &mut PointLight, &mut Transform)>,
) {
    for (entity, token_light, mut point, mut transform) in &mut lights {
        let Ok((_, source, position)) = sources.get(token_light.owner) else {
            commands.entity(entity).despawn();
            continue;
        };
        point.color = source.colour;
        point.range = (source.radius as f32 + 1.0) * CELL_SIZE;
        transform.translation = Vec3::new(
            position.x as f32 * CELL_SIZE,
            TOKEN_LIGHT_HEIGHT,
            position.y as f32 * CELL_SIZE,
        );
    }
    for (owner, source, _) in &sources {
        if lights.iter().any(|(_, light, _, _)| light.owner == owner) {
            continue;
        }
        commands.entity(*scene).with_child((
            TokenLight { owner },
            PointLight {
                color: source.colour,
                intensity: TOKEN_LIGHT_LUMENS,
                range: (source.radius as f32 + 1.0) * CELL_SIZE,
                ..default()
            },
            // Placed on the next run.
            Transform::default(),
        ));
    }
}
