//! What stands at a remembered cell, and flat open cells or stairs.

use crate::dungeon_map::liquid::LooksLiquid;
use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::protocol::SeenStairs;

use super::super::CELL_SIZE;
use super::super::field;
use super::super::holes::build_hole;
use super::super::liquid::build_cell;
use super::super::oasis::{build_grass_or_tree, build_tree};
use super::super::rampart;
use super::super::stairs::{build_stairs, hides_viewer};
use super::super::town;
use super::super::walls::{Layers, Scene, build_opening, build_secret_door, build_wall};
use super::{MEMORY_SHADE, Surface, corner_light, with_ambient};

/// Whatever stands at a remembered cell.
pub(super) fn build_cell_at(
    layers: &mut Layers,
    scene: &Scene,
    stairs: &[SeenStairs],
    (x, y): (i64, i64),
) {
    let level = scene.level;
    let cell = level.cell(x, y);
    if rampart::build_rampart(layers, scene, (x, y)) {
        // A wall round a site, and its gates (which draw their own floor).
        if level.is_wall(x, y) {
            build_ground(layers, scene, (x, y));
        }
    } else if level.is_wall(x, y) {
        // A town's buildings are drawn whole, in `build`; seen from the
        // wilderness, which isn't told them, as walls.
        if level.room(x, y).is_none() || scene.buildings.is_empty() {
            build_wall(layers, scene, (x, y));
            // Only sent to truesight: otherwise it arrives as a wall.
            build_secret_door(layers, scene, (x, y));
        }
    } else if cell.is_road() || cell == Cell::Garden {
        town::build_flat(layers, scene, (x, y));
        town::build_hedges(layers, scene, (x, y));
    } else if level.is_opening(x, y) {
        build_opening(layers, scene, (x, y));
    } else if cell == Cell::Hole {
        build_hole(layers, scene, (x, y));
    } else if cell == Cell::Field && field::build_field(layers, scene, (x, y)) {
        // Drawn: anything else of a field is the land's own ground.
    } else if scene.tint.is_some() && matches!(cell, Cell::Grass | Cell::Tree | Cell::WorldGate) {
        // The wilderness: its own ground, and any tree on it.
        build_open_cell(layers, scene, (x, y), None);
        if cell == Cell::Tree {
            build_tree(layers, scene, (x, y));
        }
    } else if build_grass_or_tree(layers, scene, (x, y)) {
        // An oasis's grass, and any tree on it.
    } else {
        let found = stairs.iter().find(|s| (s.x, s.y) == (x, y));
        build_open_cell(layers, scene, (x, y), found);
        field::build_fence(layers, scene, (x, y));
    }
}

/// The land's own ground over a whole cell, as under a wall round a site.
fn build_ground(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    let half = CELL_SIZE / 2.0;
    let ground = Surface::Ground as usize;
    if !scene.sight.is_seen(x, y) {
        let grey = Vec3::splat(MEMORY_SHADE);
        layers.memory[ground].quad(centre, Vec3::Y, Vec3::NEG_Z, (half, half), |_| grey);
        return;
    }
    let Scene { level, light, .. } = *scene;
    let tint = scene.tint.map_or(Vec3::ONE, |tints| tints.at(x, y, false));
    let lit = |p| with_ambient(corner_light(level, light, p).unwrap_or(light.at(x, y))) * tint;
    layers.solid[ground].quad(centre, Vec3::Y, Vec3::NEG_Z, (half, half), lit);
}

/// A flat open cell, or stairs: lit if in sight, otherwise as memory.
pub(super) fn build_open_cell(
    layers: &mut Layers,
    scene: &Scene,
    (x, y): (i64, i64),
    stairs: Option<&SeenStairs>,
) {
    let Scene { level, light, .. } = *scene;
    if level.cell(x, y).looks_liquid() {
        build_cell(layers, scene, (x, y));
        return;
    }
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    // A World Gate's floor in the wilderness is the land's own ground.
    let surface = match level.cell(x, y) {
        Cell::WorldGate if scene.tint.is_some() => Surface::Ground,
        cell => Surface::of_flat(cell),
    };
    if !scene.sight.is_seen(x, y) {
        let grey = Vec3::splat(MEMORY_SHADE);
        match stairs {
            Some(found) => build_stairs(
                &mut layers.memory,
                None,
                &mut layers.outlined,
                found,
                surface,
                grey,
                &|_| grey,
            ),
            None => {
                let half = CELL_SIZE / 2.0;
                layers.memory[surface as usize].quad(
                    centre,
                    Vec3::Y,
                    Vec3::NEG_Z,
                    (half, half),
                    |_| grey,
                );
            }
        }
        return;
    }
    let floor_light = |p| corner_light(level, light, p).unwrap_or(light.at(x, y));
    match stairs {
        Some(found) => {
            let lit = with_ambient(light.at(x, y));
            let floor_colour = |p| with_ambient(floor_light(p));
            // Measured from their ground, on raised land.
            let camera = scene.camera - Vec3::Y * scene.ground(x, y);
            let through =
                hides_viewer(found, scene.views, camera).then_some(&mut layers.stairs_through[..]);
            build_stairs(
                &mut layers.solid,
                through,
                &mut layers.outlined,
                found,
                surface,
                lit,
                &floor_colour,
            );
        }
        None => {
            let cell = level.cell(x, y);
            let tint = scene
                .tint
                .filter(|_| surface == Surface::Ground)
                .map_or(Vec3::ONE, |tints| tints.at(x, y, cell == Cell::Grass));
            // After the ambient light, which would wash it out at night.
            let lit = |p| with_ambient(floor_light(p)) * tint;
            let half = CELL_SIZE / 2.0;
            layers.solid[surface as usize].quad(centre, Vec3::Y, Vec3::NEG_Z, (half, half), lit);
        }
    }
}
