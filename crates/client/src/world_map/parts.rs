//! The scene's parts: its root, what follows the player, terrain, edges, rivers
//! and roads.

use bevy::prelude::*;

/// Everything this view draws (terrain, edge faces, sun), hidden while
/// another view is current.
#[derive(Component)]
pub(super) struct WorldMapScene;

/// Entities that sit at the player's position: terrain and edge faces.
#[derive(Component)]
pub(super) struct FollowsPlayer;

#[derive(Component)]
pub(super) struct Terrain;

#[derive(Component)]
pub(super) struct EdgeFaces;

#[derive(Component)]
pub(super) struct RiverStrips;

#[derive(Component)]
pub(super) struct RiverBanks;

#[derive(Component)]
pub(super) struct RoadStrips;

/// The meshes of the rivers' strips and banks, and the roads.
pub(super) type StripMeshes<'w, 's> = (
    Single<'w, 's, &'static Mesh3d, With<RiverStrips>>,
    Single<'w, 's, &'static Mesh3d, With<RiverBanks>>,
    Single<'w, 's, &'static Mesh3d, With<RoadStrips>>,
);
