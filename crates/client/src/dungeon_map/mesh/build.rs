//! Building the meshes for everything in sight and remembered.

use bevy::prelude::*;
use shared::dungeon::{Level, Sight};
use shared::protocol::SeenStairs;
use shared::site::{BuildingLook, FieldLook, WallLook};
use std::collections::{BTreeMap, BTreeSet};

use super::super::liquid::LooksLiquid;
use super::super::outlined::Outlined;
use super::super::theme::Shape;
use super::super::town;
use super::super::walls::{Layers, LeafPose, Scene};
use super::super::wild_ground::Wild;
use super::cutaway::{cut_height, settle_in_the_way};
use super::flow::flow_liquid;
use super::marks::Marks;
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
#[cfg(test)]
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
    wild: Option<Wild>,
) -> DungeonMeshes {
    let looks = (buildings, lit_buildings, fields, &[][..], ruined);
    let no_swings = BTreeMap::new();
    let seen = (views, sight, remembered, cut);
    build_posed(
        level,
        seen,
        light,
        stairs,
        looks,
        (camera, shape),
        wild,
        &no_swings,
    )
}

/// `build`, with door leaves part way through swinging drawn as `leaves` says.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn build_posed(
    level: &Level,
    (views, sight, remembered, cut): (&[View], &Sight, &Sight, &Sight),
    light: &LightGrid,
    stairs: &[SeenStairs],
    (buildings, lit_buildings, fields, walls, ruined): (
        &[BuildingLook],
        &BTreeSet<u16>,
        &[FieldLook],
        &[WallLook],
        bool,
    ),
    (camera, shape): (Vec3, Shape),
    wild: Option<Wild>,
    leaves: &BTreeMap<(i64, i64), LeafPose>,
) -> DungeonMeshes {
    let mut layers = Layers {
        solid: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        memory: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        see_through: Builder::default(),
        stairs_through: Surface::ALL.iter().map(|_| Builder::default()).collect(),
        solid_faces: Vec::new(),
        outlined: Outlined::default(),
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
        walls,
        ruined,
        tint: wild.and_then(|wild| wild.tints),
        heights: wild.map(|wild| wild.heights),
        leaves,
    };
    let (min_x, min_y, width, height) = level.bounds();
    // The first viewer is the player.
    let player = views.first().map(|view| view.cell);

    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if !remembered.is_seen(x, y) {
                continue;
            }
            let before = sizes(&layers.solid);
            let marks = Marks::of(&layers);
            let through = (
                layers.see_through.positions.len(),
                layers.see_through.indices.len(),
            );
            build_cell_at(&mut layers, &scene, stairs, (x, y));
            if level.is_wall(x, y) || level.is_opening(x, y) {
                // Measured from its ground, on sloping land.
                let camera = scene.camera - Vec3::Y * scene.ground(x, y);
                let cut = player.and_then(|player| cut_height(player, (x, y)));
                settle_in_the_way(&mut layers, &marks, through, camera, cut);
            }
            marks.drape(&mut layers, |at| scene.surface(at));
            if level.cell(x, y).looks_liquid() {
                flow_liquid(&mut layers, &marks, &scene, (x, y));
            }
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
        outlined,
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
        outlined: outlined.into_meshes(),
    }
}
