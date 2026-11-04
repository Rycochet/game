//! The scene drawn into the view's picture: its camera, the globe, a light for
//! each sun and the moons, all on their own render layer.

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::prelude::*;
use shared::celestial::BodyKind;
use std::f32::consts::FRAC_PI_2;

use super::PLANET_LAYER;
use super::moon_look::MoonLook;
use super::sky::moon_distance;
use crate::common::CurrentWorld;

/// The camera's field of view, and how far above the equator it looks from,
/// so the north pole shows.
const FIELD_OF_VIEW: f32 = 30.0;
const CAMERA_PITCH: f32 = 20.0;
/// Room round the furthest moon, or the globe if it has none.
const MARGIN: f32 = 1.15;
/// A moon's radius for each unit of its size (`0.3..1`), in the globe's radii.
const MOON_RADIUS: f32 = 0.2;
/// The light of a sun of strength 1, and the dull light everywhere, so the
/// night side shows a little.
const SUN_LUX: f32 = 4000.0;
const AMBIENT: f32 = 60.0;

/// What `spin` moves.
pub(super) struct Scene {
    /// Everything below; despawning it removes the lot.
    pub(super) root: Entity,
    /// Turns about +Y with the day.
    pub(super) globe: Entity,
    /// A light for each sun, in `Celestial::bodies`' order.
    pub(super) suns: Vec<Entity>,
    /// Each moon, with its number in `Celestial::bodies` and its looks.
    pub(super) moons: Vec<(Entity, usize, MoonLook)>,
}

pub(super) fn spawn_scene(
    commands: &mut Commands,
    world: &CurrentWorld,
    texture: Handle<Image>,
    picture: Handle<Image>,
    assets: (
        &mut Assets<Mesh>,
        &mut Assets<StandardMaterial>,
        &mut Assets<Image>,
    ),
) -> Scene {
    let (meshes, materials, images) = assets;
    let layer = RenderLayers::layer(PLANET_LAYER);
    let root = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();
    let sphere = meshes.add(Sphere::new(1.0).mesh().uv(96, 48));

    // Bevy's sphere has its poles on Z, with its picture's top at +Z: tip it
    // so north is up.
    let surface = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        ..default()
    });
    let globe = commands
        .spawn((Transform::default(), Visibility::default(), ChildOf(root)))
        .with_child((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(surface),
            Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            layer.clone(),
        ))
        .id();

    let mut suns = Vec::new();
    let mut moons = Vec::new();
    let mut furthest: f32 = 1.0;
    let mut moon_number = 0;
    for (number, body) in world.celestial.bodies().iter().enumerate() {
        let [red, green, blue] = body.colour.map(|part| part as f32);
        let colour = Color::linear_rgb(red, green, blue);
        if body.kind == BodyKind::Sun {
            let strength = world.celestial.sun_strength(number) as f32;
            suns.push(
                commands
                    .spawn((
                        DirectionalLight {
                            color: colour,
                            illuminance: SUN_LUX * strength,
                            ..default()
                        },
                        Transform::default(),
                        layer.clone(),
                        ChildOf(root),
                    ))
                    .id(),
            );
            continue;
        }
        let look = MoonLook::of(world.params.seed, moon_number);
        moon_number += 1;
        let radius = MOON_RADIUS * body.size as f32;
        let period = world.celestial.period(number).unwrap_or(24.0);
        furthest = furthest.max(moon_distance(period) + radius);
        let material = materials.add(StandardMaterial {
            base_color: colour,
            base_color_texture: Some(images.add(look.image())),
            perceptual_roughness: 1.0,
            reflectance: 0.1,
            ..default()
        });
        let moon = commands
            .spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(material),
                Transform::from_scale(Vec3::splat(radius)),
                layer.clone(),
                ChildOf(root),
            ))
            .id();
        moons.push((moon, number, look));
    }

    spawn_camera(commands, root, picture, furthest);
    Scene {
        root,
        globe,
        suns,
        moons,
    }
}

/// Far enough back that everything out to `furthest` fits.
fn spawn_camera(commands: &mut Commands, root: Entity, picture: Handle<Image>, furthest: f32) {
    let half_view = (FIELD_OF_VIEW / 2.0).to_radians();
    let distance = MARGIN * furthest / half_view.sin();
    let pitch = CAMERA_PITCH.to_radians();
    let from = Vec3::new(0.0, pitch.sin(), pitch.cos()) * distance;
    commands.spawn((
        Camera3d::default(),
        Camera {
            // Before anything that shows it.
            order: -50,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderTarget::Image(picture.into()),
        Projection::Perspective(PerspectiveProjection {
            fov: FIELD_OF_VIEW.to_radians(),
            near: 0.1,
            far: distance * 3.0,
            ..default()
        }),
        AmbientLight {
            brightness: AMBIENT,
            ..default()
        },
        Tonemapping::None,
        DebandDither::Disabled,
        Transform::from_translation(from).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(PLANET_LAYER),
        ChildOf(root),
    ));
}
