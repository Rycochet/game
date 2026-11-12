//! Every building of the town, and which are seen or in the way.

use bevy::prelude::*;
use shared::dungeon::RoomId;
use shared::site::BuildingLook;
use std::collections::BTreeSet;

use super::super::mesh::Marks;
use super::super::walls::{Layers, Scene};
use super::super::{CELL_SIZE, WALL_HEIGHT};
use super::{ROOF_RISE, build_cell, cell_centre};

/// Every building of the town.
pub fn build_buildings(layers: &mut Layers, scene: &Scene) {
    let hidden = in_the_way(scene);
    for building in scene.buildings {
        let id = RoomId(building.id);
        let seen = building_seen(scene, building, id);
        let hides = hidden.contains(&building.id);
        let marks = Marks::of(layers);
        for rect in &building.rects {
            for y in rect.y..rect.y + rect.height {
                for x in rect.x..rect.x + rect.width {
                    build_cell(layers, scene, building, (x, y), id, seen, hides);
                }
            }
        }
        // On a cutout's flat ground, all at one height.
        marks.drape(layers, |at| scene.surface(at));
    }
}

/// The buildings on the camera's line to a viewer, at any height a building
/// reaches.
pub(super) fn in_the_way(scene: &Scene) -> BTreeSet<u16> {
    let mut found = BTreeSet::new();
    let top = WALL_HEIGHT + ROOF_RISE;
    for view in scene.views {
        let target = cell_centre(view.cell.0, view.cell.1);
        // Measured from the viewer's ground, on raised land.
        let camera = scene.camera - Vec3::Y * scene.ground(view.cell.0, view.cell.1);
        if camera.y <= top {
            continue;
        }
        // From where the line comes down through the roofs to the ground.
        let from = target + (camera - target) * (top / camera.y);
        let steps = ((from - target).length() / (CELL_SIZE / 4.0))
            .ceil()
            .max(1.0) as i32;
        for step in 0..=steps {
            let at = target.lerp(from, step as f32 / steps as f32);
            let cell = (
                (at.x / CELL_SIZE).round() as i64,
                (at.z / CELL_SIZE).round() as i64,
            );
            if let Some(room) = scene.level.room(cell.0, cell.1) {
                found.insert(room.0);
            }
        }
    }
    found
}

/// Whether anyone can see any part of a building.
pub(super) fn building_seen(scene: &Scene, building: &BuildingLook, id: RoomId) -> bool {
    let level = scene.level;
    building.rects.iter().any(|rect| {
        (rect.y..rect.y + rect.height).any(|y| {
            (rect.x..rect.x + rect.width).any(|x| {
                let edge = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                    .iter()
                    .any(|(dx, dy)| level.room(x + dx, y + dy) != Some(id));
                edge && scene.sight.is_seen(x, y)
            })
        })
    })
}
