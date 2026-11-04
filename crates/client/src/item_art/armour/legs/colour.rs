//! How each part of a leg piece looks: finishes on metal, dyes on cloth.

use bevy::prelude::*;
use shared::armour::legs::LegsForm;
use shared::armour::{Construction, Finish};
use shared::item::Quality;
use shared::material::{Category, Material};

use crate::item_art::armour::torso::{dye_colour, finished};
use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

pub(super) fn material_of(
    stuff: Stuff,
    form: &LegsForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let cloth = || {
        let base = match form.construction {
            Construction::Padded => form.material,
            _ if form.japanese() => Material::Silk,
            _ => Material::Linen,
        };
        matte(dye_colour(form.cloth.unwrap_or_default(), base), 0.9)
    };
    let main = || match form.material.category() {
        Category::Metal => finished(form.material, form.finish, quality),
        Category::Leather => match form.finish {
            Finish::Lacquered(dye) => matte(dye_colour(dye, form.material), 0.25),
            _ => matte(colour_of(form.material), 0.75),
        },
        Category::Textile => cloth(),
        _ => matte(colour_of(form.material), 0.4),
    };
    match stuff {
        Stuff::Main => main(),
        // Japanese mail is black lacquered iron.
        Stuff::Shade if form.japanese() => finished(Material::Iron, Finish::Blackened, quality),
        // The dizçek's mail between its plates.
        Stuff::Shade if form.construction == Construction::MailAndPlate => {
            finished(form.material, Finish::Bright, quality)
        }
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => cloth(),
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
        _ => main(),
    }
}
