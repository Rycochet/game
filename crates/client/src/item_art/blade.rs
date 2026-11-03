//! A weapon's model, built from its rolled form. It is first made in millimetres, with
//! the point up and any edge to the right, then fitted to the cells it takes up. Lengths
//! are squeezed to fit but widths much less, or a pike would be a black line.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::blade::{BladeForm, Build, Decoration, Tips};
use shared::item::Quality;
use shared::material::Material;

use super::ring::metal_material;
use super::{BladeLook, CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

// Split into files (`cargo xtask lint` keeps files small).
mod edge;
mod haft;
mod hilt;
mod model;
mod section;

use edge::add_blade;
use haft::{add_barbs, add_shaft_top, add_side_build, add_side_part, add_tassel, add_tines};
use hilt::{add_guard, add_handle, add_pommel, handle_at};
use model::{Model, STUFFS, Stuff, Triangle};
use section::{across_at, half_width, ricasso_length, section_points, thickness};

/// How many of a picture's pixels are one cell, when drawn for a grid.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round a model, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// How many millimetres across one cell shows at most, whatever its length shows.
const CELL_ACROSS_MM: f32 = 150.0;
/// How far it is turned along its length, so that its faces catch the light differently.
const TURN: f32 = 0.35;
/// Half a handle's width, a little over 3 cm across as its weight assumes.
const HANDLE_RADIUS: f32 = 15.0;
/// Sides of anything round.
const ROUND_SIDES: usize = 8;
/// Steps along a blade: more for one that keeps changing.
const BLADE_STEPS: usize = 28;
const WAVY_STEPS: usize = 56;
/// Steps along a handle.
const HANDLE_STEPS: usize = 12;

/// The whole weapon, in millimetres.
fn build(form: &BladeForm) -> Model {
    let mut model = Model::default();
    add_handle(&mut model, form);
    let on_shaft = form.handle >= 400;
    match form.build {
        Build::Along => {
            match form.tips {
                Tips::TwoTines | Tips::ThreeTines => add_tines(&mut model, form),
                Tips::Single | Tips::Barbed => add_blade(&mut model, form),
            }
            if form.tips == Tips::Barbed {
                add_barbs(&mut model, form);
            }
            if on_shaft {
                add_shaft_top(&mut model, form);
            }
            add_side_part(&mut model, form, form.sides[0], 1.0, 8.0);
            add_side_part(&mut model, form, form.sides[1], -1.0, 8.0);
            add_guard(&mut model, form);
        }
        Build::Side => add_side_build(&mut model, form),
    }
    add_pommel(&mut model, form);
    if form.decoration == Decoration::Tasselled {
        let from = match (on_shaft, form.build) {
            (false, _) => handle_at(form, 1.0),
            (true, Build::Along) => {
                Vec2::new(HANDLE_RADIUS, -(form.handle as f32 * 0.06).max(35.0))
            }
            (true, Build::Side) => Vec2::new(HANDLE_RADIUS, -(form.blade as f32)),
        };
        add_tassel(&mut model, from);
    }
    model
}

/// The model placed to fill these cells, with room for its black edge: its triangles by
/// what they are made of.
fn fitted(look: &BladeLook) -> Vec<(Stuff, Vec<Triangle>)> {
    let model = build(&look.form);
    let turn = Quat::from_rotation_y(TURN);
    let turned = |corner: Vec3| turn * corner;
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for triangle in model.triangles.iter().flatten() {
        for &corner in triangle {
            let corner = turned(corner);
            low = low.min(corner);
            high = high.max(corner);
        }
    }
    let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let size = high - low;
    let along = room(long) / size.y.max(1.0);
    let cell = PIXELS_PER_CELL * world_per_pixel;
    let across = (room(wide) / size.x.max(1.0)).min(cell / CELL_ACROSS_MM);
    let middle = (low + high) / 2.0;
    let scale = Vec3::new(across, along, across);
    let place = |corner: Vec3| (turned(corner) - middle) * scale;
    STUFFS
        .iter()
        .zip(&model.triangles)
        .filter(|(_, triangles)| !triangles.is_empty())
        .map(|(&stuff, triangles)| {
            let triangles = triangles.iter().map(|triangle| triangle.map(place));
            (stuff, triangles.collect())
        })
        .collect()
}

/// A flat-faced mesh of these triangles: they are small, and it shows off the bevels.
fn mesh_of(triangles: &[Triangle]) -> Mesh {
    let mut positions = Vec::with_capacity(triangles.len() * 3);
    let mut normals = Vec::with_capacity(triangles.len() * 3);
    for [a, b, c] in triangles {
        let normal = (*b - *a).cross(*c - *a).normalize_or(Vec3::Z).to_array();
        for corner in [a, b, c] {
            positions.push(corner.to_array());
            normals.push(normal);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
}

fn material_of(stuff: Stuff, form: &BladeForm, quality: Option<Quality>) -> StandardMaterial {
    match stuff {
        Stuff::Blade => metal_material(form.blade_metal, quality),
        Stuff::Fittings => metal_material(form.fittings, quality),
        Stuff::Handle => match form.handle_material {
            Material::Wood => StandardMaterial {
                base_color: Color::srgb(0.46, 0.29, 0.15),
                perceptual_roughness: 0.75,
                ..default()
            },
            Material::Bone => StandardMaterial {
                base_color: Color::srgb(0.86, 0.81, 0.67),
                perceptual_roughness: 0.6,
                ..default()
            },
            metal => metal_material(metal, quality),
        },
        Stuff::Dark => {
            let mut dark = metal_material(form.blade_metal, quality);
            dark.base_color = dark.base_color.darker(0.28);
            dark
        }
        Stuff::Inlay => metal_material(
            if form.blade_metal == Material::Gold {
                Material::Silver
            } else {
                Material::Gold
            },
            quality,
        ),
        Stuff::Cloth => StandardMaterial {
            base_color: Color::srgb(0.62, 0.12, 0.12),
            perceptual_roughness: 0.9,
            ..default()
        },
    }
}

/// A weapon's parts, all placed.
pub(super) fn blade_parts(
    look: &BladeLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    let facing = Transform::from_rotation(camera.rotation);
    fitted(look)
        .into_iter()
        .map(|(stuff, triangles)| Part {
            mesh: meshes.add(mesh_of(&triangles)),
            material: materials.add(material_of(stuff, &look.form, quality)),
            transform: facing,
        })
        .collect()
}

#[cfg(test)]
mod tests;
