//! Making the scene, and giving terrain and rivers their materials.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::terrain::HeightCache;
use super::{
    EdgeFaces, FollowsPlayer, RiverBanks, RiverStrips, RoadStrips, Terrain, WorldMapScene,
};
use crate::common::GameView;

/// Spawns the scene, hidden unless this is the current view.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_view(
    mut commands: Commands,
    view: Res<State<GameView>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<super::sky::TerrainMaterial>>,
    mut clouds: ResMut<Assets<super::sky::CloudMaterial>>,
    mut precipitation: ResMut<Assets<super::sky::PrecipitationMaterial>>,
) {
    let visibility = if *view.get() == GameView::WorldMap {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    super::sky::spawn(
        &mut commands,
        visibility,
        &mut images,
        &mut meshes,
        &mut terrain_materials,
        &mut clouds,
        &mut precipitation,
    );
    // Everything starts at the origin; `update_terrain` moves it to the player.
    super::lights::spawn(&mut commands);

    // The terrain's chunks are its children (`chunks::update_chunks`).
    commands.spawn((WorldMapScene, Terrain, Transform::default(), visibility));

    commands.spawn((
        WorldMapScene,
        RiverBanks,
        FollowsPlayer,
        // Placeholder until the first update builds the real banks.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        // See-through by vertex alpha, dark whatever the light.
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
        NoFrustumCulling,
        NotShadowCaster,
        Transform::default(),
        visibility,
    ));

    commands.spawn((
        WorldMapScene,
        RiverStrips,
        FollowsPlayer,
        // Placeholder until the first update builds the real strips.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        NoFrustumCulling,
        Transform::default(),
        visibility,
    ));

    commands.spawn((
        WorldMapScene,
        RoadStrips,
        FollowsPlayer,
        // Placeholder until the first update builds the real strips.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        NoFrustumCulling,
        Transform::default(),
        visibility,
    ));

    commands.spawn((
        WorldMapScene,
        EdgeFaces,
        FollowsPlayer,
        // Placeholder until the first update builds the real faces.
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        // Unlit, so the colours match the prototype exactly.
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        NoFrustumCulling,
        Transform::default(),
        visibility,
    ));

    commands.init_resource::<HeightCache>();
    commands.queue(attach_terrain_materials);
}

/// Rivers and roads are drawn with the sky's materials, made in `super::sky::spawn`
/// (the terrain's chunks take theirs as they are made).
/// (`Commands` run in order, so its resource exists by now.)
pub(super) fn attach_terrain_materials(world: &mut World) {
    let assets = world.resource::<super::sky::SkyAssets>();
    let rivers = assets.rivers.clone();
    let mut query = world.query_filtered::<Entity, Or<(With<RiverStrips>, With<RoadStrips>)>>();
    for entity in query.iter(world).collect::<Vec<_>>() {
        world
            .entity_mut(entity)
            .insert(MeshMaterial3d(rivers.clone()));
    }
}
