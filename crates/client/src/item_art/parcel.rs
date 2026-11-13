//! A quest's parcel (owner): a cube wrapped in brown paper and tied with
//! string, both ways round, with a knot on top.

use bevy::prelude::*;

use super::Part;

/// How big the parcel is, against the picture's `VIEW_WIDTH` of 2.5.
const SIDE: f32 = 1.4;
/// How thick the string is.
const STRING: f32 = 0.05;

pub(super) fn parcel_parts(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let paper = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.45, 0.28),
        perceptual_roughness: 0.95,
        reflectance: 0.1,
        ..default()
    });
    let string = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.8, 0.66),
        perceptual_roughness: 0.9,
        ..default()
    });
    let around = SIDE + STRING;
    let mut parts = vec![Part {
        mesh: meshes.add(Cuboid::new(SIDE, SIDE, SIDE)),
        material: paper,
        transform: Transform::default(),
    }];
    // One band round it each way, crossing on top and underneath.
    for (wide, deep) in [(STRING, around), (around, STRING)] {
        parts.push(Part {
            mesh: meshes.add(Cuboid::new(wide, around, deep)),
            material: string.clone(),
            transform: Transform::default(),
        });
    }
    parts.push(Part {
        mesh: meshes.add(Sphere::new(STRING * 2.0)),
        material: string,
        transform: Transform::from_xyz(0.0, SIDE / 2.0 + STRING, 0.0),
    });
    parts
}
