//! How each part of a foot piece looks: finishes on metal, dyes on a tabi's cloth.

use bevy::prelude::*;
use shared::armour::feet::FeetForm;
use shared::armour::{Construction, Finish};
use shared::item::Quality;
use shared::material::{Category, Material};

use crate::item_art::armour::torso::{dye_colour, finished};
use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

pub(in crate::item_art) fn material_of(
    stuff: Stuff,
    form: &FeetForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let cloth = || {
        matte(
            dye_colour(form.cloth.unwrap_or_default(), Material::Cotton),
            0.9,
        )
    };
    let main = || match form.material.category() {
        Category::Metal => finished(form.material, form.finish, quality),
        Category::Leather => match form.finish {
            Finish::Lacquered(dye) => matte(dye_colour(dye, form.material), 0.25),
            _ => matte(colour_of(form.material), 0.75),
        },
        _ => matte(colour_of(form.material), 0.4),
    };
    match stuff {
        // Japanese plates and mail are black lacquered iron.
        Stuff::Main | Stuff::Shade if form.japanese() => {
            finished(Material::Iron, Finish::Blackened, quality)
        }
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => cloth(),
        Stuff::Trim => metal_material(form.trim.unwrap_or(Material::Brass), quality),
        Stuff::Rivet => metal_material(
            match form.construction {
                Construction::Leather => Material::Brass,
                _ => Material::Iron,
            },
            quality,
        ),
        _ => main(),
    }
}
