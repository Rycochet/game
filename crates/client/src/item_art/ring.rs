//! A ring's model: a cross-section swept round a circle, with its settings and gems.
//! It is built standing up, facing the camera, then all tipped back together so that
//! the gems at the top show their faces.

use std::f32::consts::TAU;

use bevy::prelude::*;
use shared::item::Quality;
use shared::item_form::Mount;
use shared::material::Material;

use super::gem::{PAVILION_DEPTH, gem_parts};
use super::{CAMERA_AT, CAMERA_UP, Part, RingLook};

// Split into files (`cargo xtask lint` keeps files small).
mod band;

use band::band_mesh;

/// From the middle of the ring to the middle of its band.
const BAND_RADIUS: f32 = 0.82;
/// How thick the band is, from its middle to its outside.
const BAND_THICKNESS: f32 = 0.05;
/// Steps round the band.
const BAND_STEPS: usize = 72;
/// Steps round a round cross-section.
const ROUND_STEPS: usize = 10;
/// How much it is tipped back, and turned, so it is seen as a ring and not a circle.
const TIP: f32 = 0.75;
const TURN: f32 = 0.35;
/// How many times a twisted band's cross-section turns on its way round.
const TWISTS: f32 = 14.0;
/// How wide a gem is, by how many settings share the band.
const GEM_ALONE: f32 = 0.32;
const GEM_OF_FEW: f32 = 0.21;
const GEM_OF_MANY: f32 = 0.16;
/// How much bigger each size up is drawn.
const SIZE_STEP: f32 = 1.2;
/// How thick a claw is, against its gem.
const CLAW: f32 = 0.09;

/// A ring's parts, all placed.
pub(super) fn ring_parts(
    ring: &RingLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let facing = whole_ring();
    let metal = materials.add(metal_material(ring.metal, quality));
    let mut parts = vec![Part {
        mesh: meshes.add(band_mesh(ring)),
        material: metal.clone(),
        transform: facing,
    }];
    let room = BAND_RADIUS + BAND_THICKNESS;
    let stem = meshes.add(Cylinder::new(1.0, 1.0).mesh());
    for setting in &ring.settings {
        let angle = setting.angle as f32 / 256.0 * TAU;
        let out = Vec3::new(angle.sin(), angle.cos(), 0.0);
        let turned = Quat::from_rotation_arc(Vec3::Y, out);
        let width = setting_width(ring.settings.len()) * size_scale(setting.bigger);
        let sits = room + width * PAVILION_DEPTH * 0.6;
        let centre = out * sits;
        let mut place = |local: Transform, material: Handle<StandardMaterial>| {
            let transform = facing
                * Transform::from_translation(out * room)
                * Transform::from_rotation(turned)
                * local;
            parts.push(Part {
                mesh: stem.clone(),
                material,
                transform,
            });
        };
        let height = sits - room;
        match setting.mount {
            Mount::Bezel => place(
                Transform::from_xyz(0.0, height / 2.0, 0.0).with_scale(Vec3::new(
                    width * 1.05,
                    height + 0.01,
                    width * 1.05,
                )),
                metal.clone(),
            ),
            Mount::Claws => {
                for corner in 0..4 {
                    let around = (corner as f32 + 0.5) / 4.0 * TAU;
                    let at = Vec3::new(around.cos(), 0.0, around.sin());
                    let from = at * width * 0.25;
                    let to = at * width * 0.95 + Vec3::Y * (height + width * 0.12);
                    place(rod(from, to, width * CLAW), metal.clone());
                }
            }
        }
        // Where nothing sits, the dark of the empty seat shows.
        let Some(gem) = &setting.gem else {
            let dark = materials.add(StandardMaterial {
                base_color: Color::srgb(0.05, 0.05, 0.06),
                perceptual_roughness: 1.0,
                ..default()
            });
            place(
                Transform::from_xyz(0.0, height + 0.012, 0.0).with_scale(Vec3::new(
                    width * 0.8,
                    0.01,
                    width * 0.8,
                )),
                dark,
            );
            continue;
        };
        let scale = width * size_scale(gem.bigger);
        let transform = facing
            * Transform::from_translation(centre)
                .with_rotation(turned)
                .with_scale(Vec3::splat(scale));
        for (mesh, material) in gem_parts(&gem.gem, gem.quality, meshes, materials) {
            parts.push(Part {
                mesh,
                material,
                transform,
            });
        }
    }
    parts
}

/// How the whole ring lies, facing the camera then tipped back.
fn whole_ring() -> Transform {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    let tipped = Quat::from_rotation_x(TIP) * Quat::from_rotation_y(TURN);
    Transform::from_rotation(camera.rotation * tipped)
}

fn setting_width(settings: usize) -> f32 {
    match settings {
        0 | 1 => GEM_ALONE,
        2 | 3 => GEM_OF_FEW,
        _ => GEM_OF_MANY,
    }
}

fn size_scale(bigger: i8) -> f32 {
    SIZE_STEP.powi(bigger as i32)
}

/// A unit cylinder placed as a rod between two points.
fn rod(from: Vec3, to: Vec3, radius: f32) -> Transform {
    let along = to - from;
    Transform::from_translation((from + to) / 2.0)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, along.normalize()))
        .with_scale(Vec3::new(radius, along.length(), radius))
}

pub(super) fn metal_material(metal: Material, quality: Option<Quality>) -> StandardMaterial {
    let fineness = quality.unwrap_or_default() as usize as f32 / 6.0;
    StandardMaterial {
        base_color: match metal {
            Material::Adamantine => Color::srgb(0.34, 0.31, 0.42),
            Material::Brass => Color::srgb(0.86, 0.7, 0.36),
            Material::Bronze => Color::srgb(0.72, 0.5, 0.26),
            Material::Copper => Color::srgb(0.85, 0.5, 0.3),
            Material::Iron => Color::srgb(0.48, 0.49, 0.52),
            Material::Mithril => Color::srgb(0.82, 0.88, 0.96),
            Material::Silver => Color::srgb(0.88, 0.89, 0.92),
            Material::Steel => Color::srgb(0.64, 0.66, 0.7),
            _ => Color::srgb(1.0, 0.76, 0.3),
        },
        // Fully metal shows only what it reflects, and there is nothing round it.
        metallic: 0.5,
        perceptual_roughness: 0.5 - 0.3 * fineness,
        reflectance: 0.8,
        ..default()
    }
}

#[cfg(test)]
mod tests;
