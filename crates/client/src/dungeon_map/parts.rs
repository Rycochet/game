//! The scene's root, what was last drawn, the materials and the meshes' parts.

use bevy::prelude::*;
use shared::dungeon::Sight;
use std::collections::BTreeMap;

use super::mesh::{Surface, WallFace};
use super::outlined::OutlinedMeshes;
use super::sight_lines::SurfaceMaterial;

/// The root of everything this view draws, hidden while another view is
/// current.
#[derive(Component)]
pub(super) struct DungeonScene;

/// What was last drawn, for click picking: the combined sight and the solid
/// wall sides.
#[derive(Resource, Default)]
pub struct Shown {
    pub sight: Option<Sight>,
    pub(super) solid_faces: Vec<WallFace>,
    /// Models' outline meshes last built, for `apply_outlined` to put in place.
    pub(super) outlined: Option<OutlinedMeshes>,
    /// Counts the times sight was worked out again, or the level changed.
    pub(super) looks: u64,
}

/// The materials of the solid meshes (in `Surface::ALL` order), of the
/// remembered ones and of the see-through walls, so their pictures can
/// follow the theme.
#[derive(Resource)]
pub(super) struct SurfaceMaterials {
    pub(super) solid: Vec<Handle<SurfaceMaterial>>,
    pub(super) memory: Vec<Handle<SurfaceMaterial>>,
    pub(super) see_through: Handle<SurfaceMaterial>,
    /// Of stairs in the way of a viewer, in `Surface::ALL` order.
    pub(super) stairs_through: Vec<Handle<SurfaceMaterial>>,
    /// The outside ground's picture as drawn, for `colour_ground` to copy.
    pub(super) ground_picture: Handle<Image>,
    /// Pictures waiting to load so a grey copy can be made for the
    /// remembered surface, by surface index.
    pub(super) to_grey: Vec<(usize, Handle<Image>)>,
    /// Grey copies already made, by the picture they copy.
    pub(super) greys: BTreeMap<AssetId<Image>, Handle<Image>>,
}

/// The mesh of one kind of surface (floors, solid walls...).
#[derive(Component)]
pub(super) struct SolidPart(pub(super) Surface);

/// The mesh of one kind of surface remembered but out of sight.
#[derive(Component)]
pub(super) struct MemoryPart(pub(super) Surface);

/// The mesh of half see-through walls.
#[derive(Component)]
pub(super) struct SeeThroughPart;

/// The mesh of one kind of surface of stairs in the way of a viewer.
#[derive(Component)]
pub(super) struct StairsThroughPart(pub(super) Surface);

/// The mesh of crops standing in fields.
#[derive(Component)]
pub(super) struct CropPart;

/// The mesh of what grows in a herbalist's patch.
#[derive(Component)]
pub(super) struct HerbPart;

/// A real light for tokens, following `owner`'s `LightSource`.
#[derive(Component)]
pub(super) struct TokenLight {
    pub(super) owner: Entity,
}
