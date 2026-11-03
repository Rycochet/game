//! What is built: viewers, wall faces for picking, and the dungeon's meshes.

use bevy::prelude::*;
use shared::dungeon::Sight;

/// What a viewer (the player, or anything sharing its sight) sees from its
/// cell.
pub struct View {
    pub cell: (i64, i64),
    pub sight: Sight,
}

/// A piece of wall drawn solid, kept for click picking: its centre, the way
/// it faces, and half its size along the wall and up it.
#[derive(Clone, Copy, Debug)]
pub struct WallFace {
    pub centre: Vec3,
    pub normal: Vec3,
    pub half_across: f32,
    pub half_up: f32,
}

/// The meshes the dungeon is drawn with.
pub struct DungeonMeshes {
    /// One per `Surface` (in `Surface::ALL` order): floors, wall sides facing
    /// the camera, stairs...
    pub solid: Vec<Mesh>,
    pub has_solid: Vec<bool>,
    /// The same for what is remembered but out of sight.
    pub memory: Vec<Mesh>,
    pub has_memory: Vec<bool>,
    /// Wall sides facing away from the camera (the wall stands between the
    /// camera and what is seen), drawn half see-through.
    pub see_through: Mesh,
    pub has_see_through: bool,
    /// Stairs in the way of a viewer, by surface (in `Surface::ALL` order),
    /// drawn half see-through.
    pub stairs_through: Vec<Mesh>,
    pub has_stairs_through: Vec<bool>,
    /// The solid wall sides, which block clicks.
    pub solid_faces: Vec<WallFace>,
}
