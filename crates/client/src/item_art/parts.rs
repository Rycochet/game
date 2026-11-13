//! The parts of a look's model, from whichever builder makes that kind of thing.

use bevy::prelude::*;

use super::{Look, Part};

pub(super) fn parts_of(
    look: Look,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    match look {
        Look::Gem { gem, quality } => super::gem::gem_parts(&gem, quality, meshes, materials)
            .into_iter()
            .map(|(mesh, material)| Part {
                mesh,
                material,
                transform: Transform::default(),
            })
            .collect(),
        Look::Ring { ring, quality } => super::ring::ring_parts(&ring, quality, meshes, materials),
        Look::Blade { blade, quality } => {
            super::blade::blade_parts(&blade, quality, meshes, materials)
        }
        Look::Body {
            layout,
            height,
            colours,
            hair,
        } => super::body::body_parts(&layout, height, colours, hair, meshes, materials),
        Look::Character {
            kind,
            worn,
            colours,
            hair,
        } => super::character::character_parts(kind, &worn, colours, hair, None, meshes, materials),
        Look::Portrait(character) => match *character {
            Look::Character {
                kind,
                worn,
                colours,
                hair,
            } => {
                let standing = Some(super::character::portrait_scale(kind));
                super::character::character_parts(
                    kind, &worn, colours, hair, standing, meshes, materials,
                )
            }
            _ => Vec::new(),
        },
        Look::Morph(morph) => super::body::morph_parts(&morph, meshes, materials),
        Look::Footwear { footwear, quality } => {
            super::footwear::footwear_parts(&footwear, quality, meshes, materials)
        }
        Look::Parcel => super::parcel::parcel_parts(meshes, materials),
        Look::Under { under, quality } => {
            super::under::under_parts(&under, quality, meshes, materials)
        }
        look => super::armour::armour_parts(look, meshes, materials),
    }
}
