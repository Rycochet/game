//! How each part of a torso piece looks: finishes on metal, dyes on cloth.

use bevy::prelude::*;
use shared::armour::torso::{Rivets, TorsoBody, TorsoForm};
use shared::armour::{Construction, Dye, Finish};
use shared::item::Quality;
use shared::material::{Category, Material};

use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

/// What a dye looks like; undyed is the cloth's own colour.
pub(in crate::item_art) fn dye_colour(dye: Dye, cloth: Material) -> Color {
    match dye {
        Dye::Natural => colour_of(cloth),
        Dye::Red => Color::srgb(0.62, 0.1, 0.1),
        Dye::Blue => Color::srgb(0.14, 0.24, 0.55),
        Dye::Green => Color::srgb(0.14, 0.38, 0.18),
        Dye::Black => Color::srgb(0.08, 0.08, 0.09),
        Dye::Brown => Color::srgb(0.4, 0.26, 0.14),
        Dye::White => Color::srgb(0.92, 0.91, 0.88),
        Dye::Yellow => Color::srgb(0.85, 0.7, 0.2),
        Dye::Purple => Color::srgb(0.4, 0.14, 0.45),
    }
}

/// Metal as it is finished.
pub(in crate::item_art) fn finished(
    metal: Material,
    finish: Finish,
    quality: Option<Quality>,
) -> StandardMaterial {
    let tinted = |r, g, b, rough| StandardMaterial {
        base_color: Color::srgb(r, g, b),
        metallic: 0.4,
        perceptual_roughness: rough,
        ..default()
    };
    match finish {
        Finish::Bright | Finish::Etched => metal_material(metal, quality),
        Finish::Blackened => tinted(0.12, 0.12, 0.13, 0.6),
        Finish::Russet => tinted(0.36, 0.2, 0.12, 0.6),
        Finish::Blued => tinted(0.18, 0.22, 0.4, 0.35),
        Finish::Tinned => metal_material(Material::Silver, quality),
        Finish::Gilt => metal_material(Material::Gold, quality),
        Finish::Painted(dye) => matte(dye_colour(dye, Material::Linen), 0.6),
        Finish::Lacquered(dye) => matte(dye_colour(dye, Material::Linen), 0.2),
    }
}

pub(super) fn material_of(
    stuff: Stuff,
    form: &TorsoForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let cloth = || {
        let base = if form.construction == Construction::Padded {
            form.material
        } else {
            Material::Linen
        };
        matte(dye_colour(form.cover.unwrap_or_default(), base), 0.9)
    };
    let main = || match (form.finish, form.material.category()) {
        (_, Category::Metal) => finished(form.material, form.finish, quality),
        (Finish::Lacquered(dye), _) => matte(dye_colour(dye, form.material), 0.25),
        (_, Category::Leather) => matte(colour_of(form.material), 0.75),
        (_, Category::Textile) => cloth(),
        (_, Category::Plant | Category::Wood) => matte(colour_of(form.material), 0.85),
        _ => matte(colour_of(form.material), 0.4),
    };
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        // Lacing on lamellae, lames and slats; cloth elsewhere.
        Stuff::Cloth => match form.lacing {
            Some(dye) => matte(dye_colour(dye, Material::Leather), 0.7),
            None => cloth(),
        },
        // Laminated linen's painted borders take the dye; metal edges are brass.
        Stuff::Trim if form.construction == Construction::Padded => {
            let dye = match form.cover {
                Some(Dye::Natural) | None => Dye::Red,
                Some(dye) => dye,
            };
            matte(dye_colour(dye, Material::Linen), 0.8)
        }
        Stuff::Trim => metal_material(form.trim.unwrap_or(Material::Brass), quality),
        Stuff::Rivet => metal_material(
            match form.body {
                TorsoBody::Leather { .. } => Material::Brass,
                TorsoBody::PlatesInCloth {
                    rivets: Rivets::Tinned,
                    ..
                } => Material::Silver,
                TorsoBody::PlatesInCloth {
                    rivets: Rivets::Gilt,
                    ..
                } => Material::Gold,
                _ => Material::Iron,
            },
            quality,
        ),
        Stuff::Etch => StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.24),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        },
        _ => main(),
    }
}
