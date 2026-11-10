//! The terrain's chunks as entities: each built once while the window may
//! show any of it, again when its grid points are worked out again (a new
//! hour, weather, style or clearing), and dropped when out of reach. They are
//! children of the `Terrain` entity, so they show and hide with the scene.

use bevy::camera::primitives::Aabb;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::platform::time::Instant;
use bevy::prelude::*;

use super::sky::SkyAssets;
use super::terrain::{CHUNK, TerrainGrid, build_chunk, chunks_needed};
use super::{Terrain, view_square};
use crate::common::{MapPosition, Preferences, ViewStats};
use crate::player::Player;
use crate::view3d::{OrbitCamera, ViewCamera};

/// The chunks on show, by world position (see `chunks_needed`).
#[derive(Resource, Default)]
pub(super) struct TerrainChunks(HashMap<(i64, i64), Entity>);

/// After `update_terrain` has worked out the grid points the window's chunks
/// need: builds the chunks missing or out of date, and drops those out of reach.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_chunks(
    mut commands: Commands,
    player: Single<&MapPosition, With<Player>>,
    orbit: Single<&OrbitCamera, With<ViewCamera>>,
    preferences: Res<Preferences>,
    mut grid: ResMut<TerrainGrid>,
    mut chunks: ResMut<TerrainChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<SkyAssets>,
    terrain: Single<Entity, With<Terrain>>,
    mut stats: ResMut<ViewStats>,
) {
    let start = Instant::now();
    let mut built = 0;
    let square = view_square(&orbit, preferences.graphics.view_distance);
    let needed: HashSet<(i64, i64)> = chunks_needed(square, (player.x, player.y))
        .into_iter()
        .collect();
    let changed = grid.take_changed();
    chunks.0.retain(|chunk, &mut entity| {
        let keep = needed.contains(chunk);
        if !keep {
            commands.entity(entity).despawn();
        }
        keep
    });
    for &chunk in &needed {
        let shown = chunks.0.get(&chunk).copied();
        if shown.is_some() && !changed.contains(&chunk) {
            continue;
        }
        // Not worked out yet (the very first frames): built when it is.
        if !grid.has_chunk(chunk) {
            continue;
        }
        let mesh = Mesh3d(meshes.add(build_chunk(&grid, chunk)));
        built += 1;
        match shown {
            // Its bounds are worked out again for the new mesh.
            Some(entity) => {
                commands.entity(entity).insert(mesh).remove::<Aabb>();
            }
            None => {
                let at = Vec3::new((chunk.0 * CHUNK) as f32, 0.0, (chunk.1 * CHUNK) as f32);
                let entity = commands
                    .spawn((
                        mesh,
                        MeshMaterial3d(assets.terrain.clone()),
                        Transform::from_translation(at),
                        ChildOf(*terrain),
                    ))
                    .id();
                chunks.0.insert(chunk, entity);
            }
        }
    }
    if built > 0 {
        let ms = start.elapsed().as_secs_f32() * 1000.0;
        // After `update_terrain`'s line, replacing what was added before.
        let line = stats
            .summary
            .split(", chunks")
            .next()
            .unwrap_or("")
            .to_owned();
        stats.summary = format!("{line}, chunks {built} in {ms:.1} ms");
    }
}
