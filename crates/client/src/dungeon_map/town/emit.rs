//! Adding lit surfaces to the town's meshes.

use bevy::prelude::*;

use super::super::CELL_SIZE;
use super::super::mesh::{Surface, corner_light, faces_camera, with_ambient};
use super::super::walls::{Layer, Layers, Scene};

pub(in crate::dungeon_map) fn cell_centre(x: i64, y: i64) -> Vec3 {
    Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE)
}

/// The colour for the light at a point, on top of the ambient: from the open
/// cells round its grid corner, or inside a building, its own cell.
pub(super) fn light_at(scene: &Scene, point: Vec3) -> Vec3 {
    let cell = |at: f32| (at / CELL_SIZE).round() as i64;
    let here = corner_light(scene.level, scene.light, point)
        .unwrap_or_else(|| scene.light.at(cell(point.x), cell(point.z)));
    with_ambient(here)
}

/// Adds a four-cornered surface to the town's mesh in a layer. See-through
/// faces always turn towards the camera: the material culls back faces.
pub(in crate::dungeon_map) fn emit(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    positions: [Vec3; 4],
    normal: Vec3,
    uvs: [[f32; 2]; 4],
    colours: [Vec3; 4],
) {
    // Everything of a town's or farm's in view takes the light where it stands.
    let lit = |i: usize| colours[i] * light_at(scene, positions[i]);
    let colours = if layer == Layer::Memory {
        colours
    } else {
        std::array::from_fn(lit)
    };
    emit_as_given(layers, scene, layer, positions, normal, uvs, colours);
}

/// As `emit`, in just these colours, whatever the light.
pub(super) fn emit_as_given(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    positions: [Vec3; 4],
    normal: Vec3,
    uvs: [[f32; 2]; 4],
    colours: [Vec3; 4],
) {
    let town = Surface::Town as usize;
    match layer {
        Layer::Solid => layers.solid[town].free(positions, normal, uvs, colours),
        Layer::Memory => layers.memory[town].free(positions, normal, uvs, colours),
        Layer::SeeThrough => {
            let centre = (positions[0] + positions[2]) / 2.0;
            let front = if faces_camera(centre, normal, scene.camera) {
                normal
            } else {
                -normal
            };
            layers.stairs_through[town].free(positions, front, uvs, colours);
        }
    }
}
