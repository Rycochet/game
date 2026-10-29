//! The world map's features (caves, houses, castles, crypts...) as entities
//! with octagonal tokens and a picture of what they are. Which chunks have
//! one, and where, is what the server has sent (`common::CurrentWorld`).
//!
//! Only features near the player exist as entities, and only while the world
//! map is the current view: they are spawned as the player moves and
//! despawned when they fall out of range or the view changes.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::feature::FeatureKind;
use shared::protocol::SeenFeature;

use super::HALF_SIZE;
use crate::common::{
    Clickable, CurrentWorld, Icon, MapPosition, Portal, SmoothPosition, Token, TokenShape,
    TokenSize,
};
use crate::player::Player;
use crate::view3d::TOKEN_DIAMETER;

/// Features exist this far from the player (each way, east–west and
/// north–south): the corners of the window when it is turned 45°, plus a
/// token's radius.
const RANGE: i64 = (HALF_SIZE as f32 * std::f32::consts::SQRT_2 + TOKEN_DIAMETER / 2.0) as i64 + 1;

/// The tile number of the first `FeatureKind`; the rest follow in order.
const FIRST_TILE: usize = 66;

/// The feature entities that exist, by their position.
#[derive(Resource, Default)]
pub(super) struct SpawnedFeatures {
    entities: BTreeMap<(i64, i64), Entity>,
    /// Whether `entities` matches the player's position: false when the
    /// view has just become current.
    ready: bool,
}

/// The picture shown above a feature's token.
fn icon(kind: FeatureKind) -> String {
    format!("terrain/l2_terrain{:03}.png", FIRST_TILE + kind as usize)
}

/// When the player reaches a new grid point, the server has sent more of the
/// world, or the view has just become current: spawns the features now in range and despawns those that left it.
pub(super) fn spawn_nearby(
    mut commands: Commands,
    world: Res<CurrentWorld>,
    player: Single<Ref<MapPosition>, With<Player>>,
    mut spawned: ResMut<SpawnedFeatures>,
) {
    if spawned.ready && !player.is_changed() && !world.is_changed() {
        return;
    }
    spawned.ready = true;
    let near = world.features.in_area(
        (player.x - RANGE, player.y - RANGE),
        (player.x + RANGE, player.y + RANGE),
    );

    let wanted: BTreeMap<(i64, i64), SeenFeature> =
        near.into_iter().map(|f| ((f.x, f.y), f)).collect();
    spawned.entities.retain(|position, entity| {
        let keep = wanted.contains_key(position);
        if !keep {
            commands.entity(*entity).despawn();
        }
        keep
    });
    for (position, feature) in wanted {
        spawned.entities.entry(position).or_insert_with(|| {
            let at = MapPosition {
                x: feature.x,
                y: feature.y,
            };
            commands
                .spawn((
                    // Clicking it travels there and enters its site.
                    Portal::Feature(feature),
                    Clickable,
                    at,
                    SmoothPosition::from(at),
                    Token {
                        shape: TokenShape::Octagon,
                        // The pictures fill the whole tile, unlike characters.
                        size: TokenSize::Tiny,
                        ..default()
                    },
                    Icon(icon(feature.kind)),
                ))
                .id()
        });
    }
}

/// Leaving the world map: features only exist while it is on screen.
pub(super) fn despawn_all(mut commands: Commands, mut spawned: ResMut<SpawnedFeatures>) {
    spawned.ready = false;
    for entity in std::mem::take(&mut spawned.entities).into_values() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn features_are_shown_only_as_far_as_the_server_sends() {
        const { assert!(RANGE <= shared::protocol::WORLD_MAP_SIGHT) };
    }

    #[test]
    fn icons_cover_the_chosen_tiles() {
        assert_eq!(icon(FeatureKind::ALL[0]), "terrain/l2_terrain066.png");
        assert_eq!(
            icon(*FeatureKind::ALL.last().unwrap()),
            "terrain/l2_terrain099.png"
        );
    }
}
