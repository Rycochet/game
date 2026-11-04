//! How each part of a neck piece looks: finishes on metal, lacquer on leather and
//! Japanese lames, dyes on padding and lacing.

use bevy::prelude::*;
use shared::armour::neck::NeckForm;
use shared::armour::{Construction, Finish};
use shared::item::Quality;
use shared::material::{Category, Material};

use crate::item_art::armour::torso::{dye_colour, finished};
use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

pub(in crate::item_art) fn material_of(
    stuff: Stuff,
    form: &NeckForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let dye = form.cloth.unwrap_or_default();
    let cloth = || matte(dye_colour(dye, Material::Silk), 0.85);
    let main = || match (form.construction, form.material.category()) {
        (Construction::Padded, _) => matte(dye_colour(dye, form.material), 0.95),
        (_, Category::Metal) => finished(form.material, form.finish, quality),
        (_, _) => match form.finish {
            Finish::Lacquered(dye) => matte(dye_colour(dye, form.material), 0.25),
            _ => matte(colour_of(form.material), 0.7),
        },
    };
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade | Stuff::Etch => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => cloth(),
        Stuff::Trim => metal_material(
            match form.edging {
                shared::armour::neck::Edging::Patterned => Material::Copper,
                _ => form.trim.unwrap_or(Material::Brass),
            },
            quality,
        ),
        Stuff::Rivet => metal_material(
            match form.construction {
                Construction::Leather | Construction::Padded => Material::Brass,
                _ => Material::Iron,
            },
            quality,
        ),
        _ => main(),
    }
}
