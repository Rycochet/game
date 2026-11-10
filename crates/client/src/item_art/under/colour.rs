//! How underclothes look: cloth in its dye (natural silk ivory, not armour's red), a
//! second colour for hose or stays, embroidery and lace, leather soles, hooks.

use bevy::prelude::*;
use shared::armour::Dye;
use shared::item::Quality;
use shared::material::Material;
use shared::underclothes::{Decoration, Garment, Stays, UnderForm};

use crate::item_art::armour::torso::dye_colour;
use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

/// Cloth of this material in this dye.
fn cloth(dye: Dye, material: Material) -> Color {
    match (dye, material) {
        (Dye::Natural, Material::Silk) => Color::srgb(0.94, 0.9, 0.8),
        (Dye::Natural, material) => colour_of(material),
        (dye, material) => dye_colour(dye, material),
    }
}

/// How rough it looks: silk has a sheen, haircloth none.
fn roughness(material: Material) -> f32 {
    match material {
        Material::Silk => 0.45,
        Material::Leather => 0.6,
        _ => 0.95,
    }
}

pub(in crate::item_art) fn material_of(
    stuff: Stuff,
    form: &UnderForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let main = || matte(cloth(form.dye, form.material), roughness(form.material));
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Cloth => {
            let colour = match form.garment {
                Garment::Shirt(shirt) if shirt.stays == Stays::PairOfBodies => {
                    Color::srgb(0.45, 0.36, 0.26)
                }
                Garment::Shirt(_) => dye_colour(Dye::Red, Material::Silk),
                _ => cloth(form.contrast.unwrap_or(Dye::White), form.material),
            };
            matte(colour, roughness(form.material))
        }
        Stuff::Backing => matte(Color::srgb(0.3, 0.2, 0.12), 0.7),
        Stuff::Trim => matte(Color::srgb(0.95, 0.94, 0.9), 0.9),
        Stuff::Field => match form.decoration {
            Decoration::GoldThread => metal_material(Material::Gold, quality),
            _ => matte(Color::srgb(0.12, 0.1, 0.12), 0.8),
        },
        _ => metal_material(Material::Brass, quality),
    }
}
