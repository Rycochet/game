//! How each part of a head piece looks: finishes on metal, dyes on cloth, lacing,
//! crests and plumes, gilt ornaments.

use bevy::prelude::*;
use shared::armour::head::HeadForm;
use shared::armour::{Dye, Finish};
use shared::item::Quality;
use shared::material::{Category, Material};

use crate::item_art::armour::torso::{dye_colour, finished};
use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

/// Bone and horn.
const BONE: Color = Color::srgb(0.86, 0.8, 0.66);

pub(in crate::item_art) fn material_of(
    stuff: Stuff,
    form: &HeadForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let cloth = |dye: Option<Dye>| {
        let base = match form.material.category() {
            Category::Textile => form.material,
            _ => Material::Silk,
        };
        matte(dye_colour(dye.unwrap_or_default(), base), 0.9)
    };
    let main = || match form.material.category() {
        Category::Metal => finished(form.material, form.finish, quality),
        Category::Leather => match form.finish {
            Finish::Lacquered(dye) => matte(dye_colour(dye, form.material), 0.25),
            _ => matte(colour_of(form.material), 0.75),
        },
        Category::Textile => cloth(form.cover),
        _ => matte(colour_of(form.material), 0.6),
    };
    let gold = || metal_material(Material::Gold, quality);
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Cloth if form.lacing.is_some() => cloth(form.lacing),
        Stuff::Cloth => cloth(form.cover),
        Stuff::Field => matte(
            dye_colour(form.cover.unwrap_or(Dye::Red), Material::Wool),
            0.95,
        ),
        Stuff::Trim if form.finish == Finish::Gilt => gold(),
        Stuff::Trim => metal_material(Material::Brass, quality),
        Stuff::Charge => gold(),
        Stuff::Rivet => metal_material(Material::Iron, quality),
        Stuff::Bone => matte(BONE, 0.5),
        Stuff::Etch => StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.24),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        },
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
    }
}
