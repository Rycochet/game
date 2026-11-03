//! What each part is drawn in: materials and heraldic tinctures.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::shoulders::{ShouldersForm, ShouldersPiece, Tincture};
use shared::item::Quality;
use shared::material::{Category, Material};

use super::super::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

/// What a tincture looks like.
pub(super) fn tincture(tincture: Tincture, quality: Option<Quality>) -> StandardMaterial {
    match tincture {
        Tincture::Or => metal_material(Material::Gold, quality),
        Tincture::Argent => metal_material(Material::Silver, quality),
        Tincture::Gules => matte(Color::srgb(0.68, 0.1, 0.1), 0.7),
        Tincture::Azure => matte(Color::srgb(0.12, 0.24, 0.62), 0.7),
        Tincture::Vert => matte(Color::srgb(0.1, 0.42, 0.2), 0.7),
        Tincture::Sable => matte(Color::srgb(0.07, 0.07, 0.08), 0.7),
        Tincture::Purpure => matte(Color::srgb(0.42, 0.14, 0.46), 0.7),
    }
}

/// How each part looks, by what it is made of.
pub(super) fn material_of(
    stuff: Stuff,
    form: &ShouldersForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let japanese = form.japanese();
    let main = || match form.material.category() {
        // Japanese lamellae are lacquered black.
        _ if japanese && form.construction != Construction::Mail => {
            matte(Color::srgb(0.1, 0.08, 0.07), 0.3)
        }
        Category::Metal => metal_material(form.material, quality),
        Category::Leather => matte(colour_of(form.material), 0.75),
        Category::Textile => matte(colour_of(form.material), 0.95),
        Category::Wood => matte(colour_of(form.material), 0.8),
        _ => matte(colour_of(form.material), 0.4),
    };
    let heraldry = match form.piece {
        ShouldersPiece::Ailettes { heraldry, .. } => Some(heraldry),
        _ => None,
    };
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => matte(colour_of(form.cloth.unwrap_or(Material::Linen)), 0.9),
        Stuff::Trim => metal_material(form.trim.unwrap_or(Material::Brass), quality),
        Stuff::Rivet => metal_material(
            match form.construction {
                Construction::Leather | Construction::PlatesInCloth => Material::Brass,
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
        Stuff::Bone => matte(Color::srgb(0.86, 0.82, 0.7), 0.6),
        Stuff::Field => heraldry.map_or_else(main, |h| tincture(h.field, quality)),
        Stuff::Charge => heraldry.map_or_else(main, |h| tincture(h.colour, quality)),
    }
}
