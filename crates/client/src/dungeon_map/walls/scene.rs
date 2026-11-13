//! What building a cell's meshes needs to know.

use super::super::CELL_SIZE;
use super::super::wild_ground::Tints;
use crate::common::DrawnGround;
use bevy::prelude::*;
use shared::dungeon::{Level, Sight};
use shared::site::{BuildingLook, FieldLook, WallLook};
use std::collections::{BTreeMap, BTreeSet};

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
    /// Walls round the level's buildings (`rampart`).
    pub walls: &'a [WallLook],
    /// Whether the place is a ruin.
    pub ruined: bool,
    /// The wilderness's ground colours, cell by cell (none anywhere else).
    pub tint: Option<&'a Tints>,
    /// The ground's shape (none in tests: flat).
    pub heights: Option<&'a DrawnGround>,
    /// Door leaves part way through a swing, by cell (`door_swing`); the rest
    /// are drawn as they hang.
    pub leaves: &'a BTreeMap<(i64, i64), super::LeafPose>,
}

impl Scene<'_> {
    /// How high a cell's middle is drawn, in world units: 0 on flat ground.
    pub fn ground(&self, x: i64, y: i64) -> f32 {
        self.heights
            .map_or(0.0, |heights| heights.drawn(x, y) * CELL_SIZE)
    }

    /// How high the ground is under a place, in world units.
    pub fn surface(&self, at: Vec3) -> f32 {
        self.heights.map_or(0.0, |heights| {
            heights.drawn_between(at.x / CELL_SIZE, at.z / CELL_SIZE) * CELL_SIZE
        })
    }

    /// A cell's corners' heights in world units: north-west, north-east,
    /// south-west, south-east.
    pub fn corners(&self, x: i64, y: i64) -> [f32; 4] {
        let corner = |cx, cy| {
            self.heights
                .map_or(0.0, |heights| heights.corner(cx, cy) * CELL_SIZE)
        };
        [
            corner(x, y),
            corner(x + 1, y),
            corner(x, y + 1),
            corner(x + 1, y + 1),
        ]
    }
}
