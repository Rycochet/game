//! Setting up a page off screen: lights, its two cameras, and each picture's model
//! and the square its edge is drawn on.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;

use super::{Batch, CAMERA_AT, CAMERA_UP, PAGES, Part, PictureLight, VIEW_WIDTH, layers};

/// World units to a pixel of the page. Models are scaled down to this, so that a
/// whole page of them stays well in front of its camera.
const PAGE_UNIT: f32 = 1.0 / 1024.0;
/// How far back from the page the models' camera is.
const CAMERA_BACK: f32 = 10.0;

/// Strongest from the top right, as things are lit in the interface, with enough from
/// elsewhere that no face goes black, and shiny things have something to catch.
pub(super) fn spawn_lights(commands: &mut Commands) {
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
            RenderLayers::from_layers(&(0..PAGES).map(|slot| layers(slot).0).collect::<Vec<_>>()),
        ));
    }
}

/// Where the models' camera is, which every model is seen from as a picture of its
/// own would be.
fn models_view() -> Transform {
    Transform::from_translation(CAMERA_AT.normalize() * CAMERA_BACK)
        .looking_at(Vec3::ZERO, CAMERA_UP)
}

/// The batch on its entity, with its cameras.
pub(super) fn spawn_batch(commands: &mut Commands, entity: Entity, batch: Batch) {
    let size = batch.size.as_vec2();
    let (models_layer, edge_layer) = layers(batch.slot);
    // Before anything that might show them, the models before their edges.
    let order = -40 + 2 * batch.slot as isize;
    let camera = |order| Camera {
        order,
        clear_color: Color::NONE.into(),
        ..default()
    };
    let models = commands
        .spawn((
            Camera3d::default(),
            camera(order),
            AmbientLight {
                brightness: 150.0,
                ..default()
            },
            RenderTarget::Image(batch.models.clone().into()),
            Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: CAMERA_BACK * 2.0,
                scaling_mode: ScalingMode::Fixed {
                    width: size.x * PAGE_UNIT,
                    height: size.y * PAGE_UNIT,
                },
                ..OrthographicProjection::default_3d()
            }),
            Tonemapping::None,
            DebandDither::Disabled,
            models_view(),
            RenderLayers::layer(models_layer),
        ))
        .id();
    // In the page's pixels, looking at its middle.
    let edges = commands
        .spawn((
            Camera3d::default(),
            camera(order + 1),
            RenderTarget::Image(batch.page.clone().into()),
            Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 2.0,
                scaling_mode: ScalingMode::Fixed {
                    width: size.x,
                    height: size.y,
                },
                ..OrthographicProjection::default_3d()
            }),
            Tonemapping::None,
            DebandDither::Disabled,
            // Whole pixels: an edge is there or not.
            Msaa::Off,
            Transform::from_xyz(0.0, 0.0, 1.0).looking_to(Vec3::NEG_Z, Vec3::Y),
            RenderLayers::layer(edge_layer),
        ))
        .id();
    commands
        .entity(entity)
        .insert((batch, Transform::default(), Visibility::default()))
        .add_children(&[models, edges]);
}

/// A picture's model, fitted to its place on the page, and the square its edge is
/// drawn on. A model fills `VIEW_WIDTH` along its longest side, as a picture of its
/// own would.
pub(super) fn spawn_cell(
    commands: &mut Commands,
    batch: (Entity, &Batch),
    rect: URect,
    parts: Vec<Part>,
    edge: MeshMaterial3d<super::EdgeMaterial>,
) {
    let (entity, batch) = batch;
    let (models_layer, edge_layer) = layers(batch.slot);
    let half = batch.size.as_vec2() / 2.0;
    let middle = rect.as_rect().center();
    // From the page's middle, with y up.
    let across = Vec2::new(middle.x - half.x, half.y - middle.y);
    let view = models_view();
    let at = (view.right() * across.x + view.up() * across.y) * PAGE_UNIT;
    let longest = rect.width().max(rect.height()) as f32;
    let scale = longest * PAGE_UNIT / VIEW_WIDTH;
    commands
        .spawn((
            Transform::from_translation(at).with_scale(Vec3::splat(scale)),
            Visibility::default(),
            ChildOf(entity),
        ))
        .with_children(|cell| {
            for part in parts {
                cell.spawn((
                    Mesh3d(part.mesh),
                    MeshMaterial3d(part.material),
                    part.transform,
                    RenderLayers::layer(models_layer),
                ));
            }
        });
    let size = rect.size().as_vec2();
    commands.spawn((
        Mesh3d(batch.quad.clone()),
        edge,
        Transform::from_translation(across.extend(0.0)).with_scale(size.extend(1.0)),
        RenderLayers::layer(edge_layer),
        ChildOf(entity),
    ));
}
