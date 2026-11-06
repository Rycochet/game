//! What building a cell's meshes needs to know.

use super::super::wild_ground::Tints;
use bevy::prelude::*;
use shared::dungeon::{Level, Sight};
use shared::site::{BuildingLook, FieldLook};
use std::collections::BTreeSet;

use super::super::mesh::{LightGrid, View};
use super::super::theme::Shape;

/// What is needed to draw any wall, door or window of a level.
pub struct Scene<'a> {
    pub level: &'a Level,
    pub views: &'a [View],
    pub sight: &'a Sight,
    pub light: &'a LightGrid,
    pub camera: Vec3,
    pub shape: Shape,
    /// A town's buildings.
    pub buildings: &'a [BuildingLook],
    /// Which of them (by `id`) have their lights on.
    pub lit_buildings: &'a BTreeSet<u16>,
    /// A farm's fields.
    pub fields: &'a [FieldLook],
    /// Whether the place is a ruin.
    pub ruined: bool,
    /// The wilderness's ground colours, cell by cell (none anywhere else).
    pub tint: Option<&'a Tints>,
}
