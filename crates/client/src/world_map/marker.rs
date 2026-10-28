//! The player marker: a flat white cylinder with a black outline, sitting on
//! the terrain at the player's exact (smooth) position.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use super::{CAMERA_DISTANCE, CAMERA_FOV_DEGREES};

/// Marker diameter in grid units.
pub const DIAMETER: f32 = 16.0;

/// Marker thickness, as a fraction of the diameter.
const THICKNESS: f32 = 0.15;

/// Outline width on screen.
const OUTLINE_PIXELS: f32 = 4.0;

/// Marks the marker's root entity (the disc and its outline are children).
#[derive(Component)]
pub struct Marker;

/// Half the marker's thickness: how far its centre sits above the ground.
pub const HALF_THICKNESS: f32 = DIAMETER * THICKNESS / 2.0;

/// Spawns the marker. `window_height` (in pixels) sets the outline width in
/// grid units, so it is about `OUTLINE_PIXELS` on screen.
pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    window_height: f32,
) {
    // Grid units per pixel at the camera's distance.
    let visible_height = 2.0 * CAMERA_DISTANCE * (CAMERA_FOV_DEGREES.to_radians() / 2.0).tan();
    let outline = OUTLINE_PIXELS * visible_height / window_height;

    // A flat cylinder; the outline is the same shape `outline` bigger all
    // round, drawn from the inside only.
    let radius = DIAMETER / 2.0;
    let disc = |grow: f32| Cylinder::new(radius + grow, 2.0 * (HALF_THICKNESS + grow));

    // Lit, so the sun shades the top and side differently and it reads as a
    // 3D cylinder. The outline stays unlit (always black).
    let white = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.6,
        ..default()
    });
    let black = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        unlit: true,
        // Only the inside faces: shows as a rim round the white shape.
        cull_mode: Some(Face::Front),
        ..default()
    });

    commands
        .spawn((Marker, Transform::default(), Visibility::default()))
        .with_children(|parent| {
            parent.spawn((Mesh3d(meshes.add(disc(0.0))), MeshMaterial3d(white)));
            parent.spawn((
                Mesh3d(meshes.add(disc(outline))),
                MeshMaterial3d(black),
                NotShadowCaster,
            ));
        });
}
