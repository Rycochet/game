//! Setting up a drawing off screen: lights, the parts and their black outlines.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use std::f32::consts::TAU;

use super::{
    CAMERA_AT, CAMERA_UP, Drawing, FIRST_LAYER, FRAMES_ALIVE, OUTLINE_BEHIND, OUTLINE_COPIES,
    OUTLINE_PIXELS, Part, PictureLight, SLOTS, VIEW_WIDTH,
};

/// Strongest from the top right, as things are lit in the interface, with enough from
/// elsewhere that no face goes black, and shiny things have something to catch.
pub(super) fn spawn_lights(commands: &mut Commands) {
    let layers = RenderLayers::from_layers(&(FIRST_LAYER..FIRST_LAYER + SLOTS).collect::<Vec<_>>());
    let lights = [
        (Vec3::new(2.0, 3.0, 1.0), 4000.0),
        (Vec3::new(-3.0, 1.5, 1.5), 1500.0),
        (Vec3::new(0.5, 0.6, -3.0), 1000.0),
    ];
    for (from, illuminance) in lights {
        commands.spawn((
            PictureLight,
            DirectionalLight {
                illuminance,
                ..default()
            },
            Transform::from_translation(from).looking_at(Vec3::ZERO, Vec3::Y),
            layers.clone(),
        ));
    }
}

/// A black edge of the same thickness whatever is drawn: each mesh again in black,
/// moved a little each way across the picture, and back so that only what sticks out
/// shows. Going by what the camera sees, it needs nothing of a model's shape.
pub(super) fn outline_copies(
    parts: &[Part],
    pixels: u32,
    black: &Handle<StandardMaterial>,
) -> Vec<Part> {
    let looking = -CAMERA_AT.normalize();
    let right = looking.cross(CAMERA_UP).normalize();
    let up = right.cross(looking);
    let reach = OUTLINE_PIXELS * VIEW_WIDTH / pixels as f32;
    let mut copies = Vec::new();
    let mut last = None;
    for part in parts {
        // A gem is one mesh drawn twice, which needs only one edge.
        let this = (part.mesh.id(), part.transform);
        if last == Some(this) {
            continue;
        }
        last = Some(this);
        for copy in 0..OUTLINE_COPIES {
            let angle = copy as f32 / OUTLINE_COPIES as f32 * TAU;
            let across = (right * angle.cos() + up * angle.sin()) * reach;
            let mut transform = part.transform;
            transform.translation += across + looking * OUTLINE_BEHIND;
            copies.push(Part {
                mesh: part.mesh.clone(),
                material: black.clone(),
                transform,
            });
        }
    }
    copies
}

pub(super) fn spawn_drawing(
    commands: &mut Commands,
    slot: usize,
    picture: Handle<Image>,
    view: Vec2,
    parts: Vec<Part>,
    outline: Vec<Part>,
) {
    let layer = RenderLayers::layer(FIRST_LAYER + slot);
    commands
        .spawn((
            Drawing {
                slot,
                frames_left: FRAMES_ALIVE,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|drawing| {
            drawing.spawn((
                Camera3d::default(),
                Camera {
                    // Before anything that might show it.
                    order: -10 - slot as isize,
                    clear_color: Color::NONE.into(),
                    ..default()
                },
                AmbientLight {
                    brightness: 150.0,
                    ..default()
                },
                RenderTarget::Image(picture.into()),
                Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::Fixed {
                        width: view.x,
                        height: view.y,
                    },
                    ..OrthographicProjection::default_3d()
                }),
                Tonemapping::None,
                Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP),
                layer.clone(),
            ));
            for part in parts.into_iter().chain(outline) {
                drawing.spawn((
                    Mesh3d(part.mesh),
                    MeshMaterial3d(part.material),
                    part.transform,
                    layer.clone(),
                ));
            }
        });
}
