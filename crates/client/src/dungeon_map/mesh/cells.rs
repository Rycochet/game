//! What stands at a remembered cell, and flat open cells or stairs.

use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::protocol::SeenStairs;

use super::super::CELL_SIZE;
use super::super::field;
use super::super::holes::build_hole;
use super::super::liquid::build_cell;
use super::super::oasis::build_grass_or_tree;
use super::super::stairs::{build_stairs, hides_viewer};
use super::super::town;
use super::super::walls::{Layers, Scene, build_opening, build_wall};
use super::{MEMORY_SHADE, Surface, corner_light, layer_sizes, tint_since, with_ambient};

/// Whatever stands at a remembered cell.
pub(super) fn build_cell_at(
    layers: &mut Layers,
    scene: &Scene,
    stairs: &[SeenStairs],
    (x, y): (i64, i64),
) {
    let level = scene.level;
    let cell = level.cell(x, y);
    if cell.is_wall() {
        // A town's buildings are drawn whole, in `build`.
        if level.room(x, y).is_none() {
            let before = layer_sizes(layers);
            build_wall(layers, scene, (x, y));
            // Only sent to truesight: otherwise it arrives as a wall.
            if cell == Cell::SecretDoor {
                tint_since(layers, &before);
            }
        }
    } else if cell.is_road() || cell == Cell::Garden {
        town::build_flat(layers, scene, (x, y));
        town::build_hedges(layers, scene, (x, y));
    } else if cell.is_opening() {
        build_opening(layers, scene, (x, y));
    } else if cell == Cell::Hole {
        build_hole(layers, scene, (x, y));
    } else if cell == Cell::Field && field::build_field(layers, scene, (x, y)) {
        // Drawn: anything else of a field is the land's own ground.
    } else if build_grass_or_tree(layers, scene, (x, y)) {
        // An oasis's grass, and any tree on it.
    } else {
        let found = stairs.iter().find(|s| (s.x, s.y) == (x, y));
        build_open_cell(layers, scene, (x, y), found);
        field::build_fence(layers, scene, (x, y));
    }
}

/// A flat open cell, or stairs: lit if in sight, otherwise as memory.
pub(super) fn build_open_cell(
    layers: &mut Layers,
    scene: &Scene,
    (x, y): (i64, i64),
    stairs: Option<&SeenStairs>,
) {
    let Scene { level, light, .. } = *scene;
    if level.cell(x, y).is_liquid() {
        build_cell(layers, scene, (x, y));
        return;
    }
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    let surface = Surface::of_flat(level.cell(x, y));
    if !scene.sight.is_seen(x, y) {
        let grey = Vec3::splat(MEMORY_SHADE);
        match stairs {
            Some(found) => build_stairs(&mut layers.memory, None, found, surface, grey, &|_| grey),
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
            let through = hides_viewer(found, scene.views, scene.camera)
                .then_some(&mut layers.stairs_through[..]);
            build_stairs(
                &mut layers.solid,
                through,
                found,
                surface,
                lit,
                &floor_colour,
            );
        }
        None => {
            layers.solid[surface as usize].face(centre, Vec3::Y, CELL_SIZE / 2.0, floor_light);
        }
    }
}
