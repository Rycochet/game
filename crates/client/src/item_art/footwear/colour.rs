//! How each part of a pair of footwear looks: the upper in its dye, the sole, a
//! second colour, fur, thread and beads, metal fittings.

use bevy::prelude::*;
use shared::armour::Dye;
use shared::footwear::{Decoration, FootwearForm};
use shared::item::Quality;
use shared::material::{Category, Material};

use crate::item_art::armour::torso::dye_colour;
use crate::item_art::armour::{Stuff, colour_of, darker, matte};
use crate::item_art::ring::metal_material;

/// How rough a material looks: polished leather shines a little, cloth and fibre not.
fn roughness(material: Material) -> f32 {
    match material {
        Material::Leather | Material::BuffLeather => 0.55,
        Material::Wood => 0.6,
        material if material.category() == Category::Textile => 0.95,
        _ => 0.85,
    }
}

pub(super) fn material_of(
    stuff: Stuff,
    form: &FootwearForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let upper = || matte(dye_colour(form.dye, form.upper), roughness(form.upper));
    match stuff {
        Stuff::Main => upper(),
        Stuff::Shade => darker(upper()),
        Stuff::Backing => {
            let colour = match form.sole {
                // Leather soles are left undyed and darken with wear.
                Material::Leather => Color::srgb(0.3, 0.2, 0.12),
                sole if sole == form.upper => dye_colour(form.dye, sole),
                sole => colour_of(sole),
            };
            matte(colour, roughness(form.sole))
        }
        Stuff::Cloth => {
            let second = form.contrast.unwrap_or(Dye::White);
            matte(dye_colour(second, Material::Leather), 0.6)
        }
        Stuff::Charge => matte(colour_of(Material::Fur), 0.95),
        Stuff::Field => match form.decoration {
            Decoration::Embroidery => matte(Color::srgb(0.82, 0.66, 0.26), 0.4),
            _ => matte(Color::srgb(0.25, 0.62, 0.66), 0.3),
        },
        Stuff::Etch => matte(Color::srgb(0.7, 0.08, 0.14), 0.15),
        Stuff::Bone => matte(
            dye_colour(
                match form.dye {
                    Dye::Red => Dye::Black,
                    _ => Dye::Red,
                },
                Material::Wool,
            ),
            0.95,
        ),
        Stuff::Rivet => metal_material(Material::Brass, quality),
        _ => metal_material(Material::Iron, quality),
    }
}
