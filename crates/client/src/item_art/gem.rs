//! A gem's mesh and how it is drawn. Seven cuts share one builder: an outline, with a
//! flat top above it and a point below. An uncut stone is a lumpy sphere.

use bevy::prelude::*;
use bevy::render::render_resource::Face;
use shared::item::Quality;
use shared::item_form::{Cut, GemForm};
use shared::material::Material;

// Split into files (`cargo xtask lint` keeps files small).
mod facets;
mod outline;
mod uncut;

use facets::{Facets, cut_mesh};
use outline::outline;
use uncut::uncut_mesh;

/// Points round the outline. Even, for the crown's table to take every other one.
const OUTLINE_POINTS: usize = 16;
/// A multiple of 3 too, so that a trillion has its three corners.
const TRILLION_POINTS: usize = 18;
/// How wide the flat top is, against the outline.
const TABLE: f32 = 0.55;
const CROWN_HEIGHT: f32 = 0.3;
pub(super) const PAVILION_DEPTH: f32 = 0.7;
/// How much of the back shows through the front of a cut gem.
const SEE_THROUGH: f32 = 0.4;
/// How much darker the back is than the front.
const BACK_SHADE: f32 = 0.6;

/// One gem's parts, in the order they are drawn.
pub(super) fn gem_parts(
    gem: &GemForm,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<(Handle<Mesh>, Handle<StandardMaterial>)> {
    let colour = gem_colour(gem.material);
    if gem.cut == Cut::Uncut {
        let mesh = meshes.add(uncut_mesh(gem.shape_seed));
        let material = materials.add(StandardMaterial {
            base_color: colour.mix(&Color::srgb(0.45, 0.42, 0.4), 0.3),
            perceptual_roughness: 0.9,
            reflectance: 0.15,
            ..default()
        });
        return vec![(mesh, material)];
    }
    // Upside down on purpose (owner's choice): the softer, broader shine of a low
    // value reads as shinier, so the best gems get it.
    let fineness = 1.0 - quality.unwrap_or_default() as usize as f32 / 6.0;
    let mesh = meshes.add(cut_mesh(gem.cut, 0.15 + 0.25 * fineness));
    let roughness = 0.3 - 0.22 * fineness;
    let back = materials.add(StandardMaterial {
        base_color: colour.darker(1.0 - BACK_SHADE),
        perceptual_roughness: roughness,
        reflectance: 0.5,
        // What is drawn is the inside of the far faces, lit as seen from here.
        cull_mode: Some(Face::Front),
        double_sided: true,
        ..default()
    });
    let front = materials.add(StandardMaterial {
        base_color: colour.with_alpha(1.0 - SEE_THROUGH),
        perceptual_roughness: roughness,
        reflectance: 0.9,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    vec![(mesh.clone(), back), (mesh, front)]
}

fn gem_colour(gem: Material) -> Color {
    match gem {
        Material::Amethyst => Color::srgb(0.55, 0.2, 0.8),
        Material::Aquamarine => Color::srgb(0.4, 0.85, 0.9),
        Material::Citrine => Color::srgb(0.98, 0.72, 0.15),
        Material::Diamond => Color::srgb(0.93, 0.96, 1.0),
        Material::Emerald => Color::srgb(0.05, 0.7, 0.3),
        Material::Garnet => Color::srgb(0.5, 0.05, 0.12),
        // Black, with a little blue in its shine.
        Material::Onyx => Color::srgb(0.04, 0.04, 0.05),
        Material::Peridot => Color::srgb(0.6, 0.8, 0.15),
        Material::Ruby => Color::srgb(0.85, 0.05, 0.18),
        Material::Sapphire => Color::srgb(0.08, 0.2, 0.8),
        Material::Topaz => Color::srgb(0.95, 0.55, 0.3),
        _ => Color::WHITE,
    }
}

#[cfg(test)]
mod tests;
