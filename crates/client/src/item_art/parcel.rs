//! A quest's parcel (owner): a cube wrapped in brown paper and tied with
//! string, both ways round, with a knot on top; and its keepsakes' one
//! picture (owner: one for all for now), a bundle of cloth tied at the neck.

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

/// A keepsake: a round bundle of faded cloth, gathered and tied at the top.
pub(super) fn keepsake_parts(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let cloth = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.42, 0.5),
        perceptual_roughness: 0.95,
        reflectance: 0.08,
        ..default()
    });
    let string = materials.add(StandardMaterial {
        base_color: Color::srgb(0.86, 0.8, 0.66),
        perceptual_roughness: 0.9,
        ..default()
    });
    vec![
        Part {
            mesh: meshes.add(Sphere::new(0.8)),
            material: cloth.clone(),
            transform: Transform::from_xyz(0.0, -0.25, 0.0).with_scale(Vec3::new(1.0, 0.8, 1.0)),
        },
        // The gathered neck, and the cloth's ends above the tie.
        Part {
            mesh: meshes.add(Cylinder::new(0.22, 0.4)),
            material: cloth.clone(),
            transform: Transform::from_xyz(0.0, 0.55, 0.0),
        },
        Part {
            mesh: meshes.add(Cone::new(0.42, 0.35)),
            material: cloth,
            transform: Transform::from_xyz(0.0, 0.85, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        },
        Part {
            mesh: meshes.add(Torus::new(0.2, 0.27)),
            material: string,
            transform: Transform::from_xyz(0.0, 0.6, 0.0),
        },
    ]
}
