//! Building the meshes for everything in sight and remembered.

use bevy::prelude::*;
use shared::dungeon::{Level, Sight};
use shared::protocol::SeenStairs;
use shared::site::{BuildingLook, FieldLook};
use std::collections::BTreeSet;

use super::super::theme::Shape;
use super::super::town;
use super::super::walls::{Layers, Scene};
use super::super::wild_ground::Tints;
use super::{
    Builder, DungeonMeshes, LightGrid, Surface, View, build_cell_at, copy_as_memory, sizes,
};

/// Anything facing up with a normal over `RAISED_NORMAL`, this far over the
/// floor, is a top (of a wall or a roof), which the mask doesn't cut, as sight
/// stops at the wall below it. Match `sight_mask.wesl`.
pub(super) const RAISED: f32 = 1.0;
pub(super) const RAISED_NORMAL: f32 = 0.5;

/// Builds the meshes for everything in `sight` (all the views combined), lit
/// by `light`, and for the rest of `remembered` (which includes what is in
/// sight) in grey, with `stairs` drawn in 3D. Only remembered cells are
/// drawn: cells the client doesn't know may be in sight as wall. `camera` is
/// the camera's position, `shape` how the place's walls are built. Cells in
/// `cut` are in sight only in part, so what is lit there is copied in grey too.
#[allow(clippy::too_many_arguments)]
pub fn build(
    level: &Level,
    views: &[View],
    sight: &Sight,
    remembered: &Sight,
    light: &LightGrid,
    stairs: &[SeenStairs],
    buildings: &[BuildingLook],
    lit_buildings: &BTreeSet<u16>,
    fields: &[FieldLook],
    ruined: bool,
    camera: Vec3,
    shape: Shape,
    cut: &Sight,
    tint: Option<&Tints>,
) -> DungeonMeshes {
    let mut layers = Layers {
        solid: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        memory: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        see_through: Builder::default(),
        stairs_through: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        solid_faces: Vec::new(),
    };
    let scene = Scene {
        level,
        views,
        sight,
        light,
        camera,
        shape,
        buildings,
        lit_buildings,
        fields,
        ruined,
        tint,
    };
    let (min_x, min_y, width, height) = level.bounds();

    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if !remembered.is_seen(x, y) {
                continue;
            }
            let before = sizes(&layers.solid);
            build_cell_at(&mut layers, &scene, stairs, (x, y));
            if cut.is_seen(x, y) {
                copy_as_memory(&mut layers, &before);
            }
        }
    }

    // Buildings are drawn whole, not by the cell, and only the parts in sight
    // are lit, so all of those may be cut.
    let before = sizes(&layers.solid);
    town::build_buildings(&mut layers, &scene);
    copy_as_memory(&mut layers, &before);

    let Layers {
        solid,
        memory,
        see_through,
        stairs_through,
        solid_faces,
    } = layers;
    DungeonMeshes {
        has_solid: solid.iter().map(|b| !b.indices.is_empty()).collect(),
        solid: solid.into_iter().map(Builder::into_mesh).collect(),
        has_memory: memory.iter().map(|b| !b.indices.is_empty()).collect(),
        memory: memory.into_iter().map(Builder::into_mesh).collect(),
        has_see_through: !see_through.indices.is_empty(),
        see_through: see_through.into_mesh(),
        has_stairs_through: stairs_through
            .iter()
            .map(|b| !b.indices.is_empty())
            .collect(),
        stairs_through: stairs_through.into_iter().map(Builder::into_mesh).collect(),
        solid_faces,
    }
}
