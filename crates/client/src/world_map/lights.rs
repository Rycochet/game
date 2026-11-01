//! The world map's lights: one `DirectionalLight` per sun and moon (only the
//! brightest casting shadows), the ambient fill, and the parts of the scene that
//! aren't lit by them following along.

use bevy::light::light_consts;
use bevy::prelude::*;
use shared::celestial::{MOST_BODIES, brightest};

use super::{EdgeFaces, WorldMapScene};
use crate::common::SkyNow;

/// A strong light with little ambient fill, so slopes facing away go properly
/// dark. This is what a sun gives flat ground at noon, tuned so overall
/// brightness is close to the prototype's. It was first tuned with the sun low
/// in the east, where flat ground got half of `FULL_DAYLIGHT * 0.2`.
const FLAT_LUX: f32 = light_consts::lux::FULL_DAYLIGHT * 0.2 * 0.498;

/// The ambient fill, day and night: the darkest a night gets. About
/// `common::NIGHT_LIGHT` of what a noon sun gives.
const AMBIENT_BRIGHTNESS: f32 = 60.0;

/// The most light flat ground takes, however many suns are up: above about this,
/// sunlit faces clip towards lime.
const MOST_FLAT_LIGHT: f32 = 1.15;

/// A light for the `.0`th of `Celestial::at`'s bodies.
#[derive(Component)]
pub(super) struct BodyLight(usize);

/// Spawns as many lights as any world can want, hidden until `update` uses them.
pub(super) fn spawn(commands: &mut Commands) {
    for number in 0..MOST_BODIES {
        commands.spawn((
            WorldMapScene,
            BodyLight(number),
            DirectionalLight::default(),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
}

/// Every frame in this view: puts the lights where the suns and moons are.
pub(super) fn update(
    sky: Res<SkyNow>,
    ambient: Option<ResMut<GlobalAmbientLight>>,
    faces: Single<&MeshMaterial3d<StandardMaterial>, With<EdgeFaces>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lights: Query<(
        &BodyLight,
        &mut DirectionalLight,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let strongest_channel = sky.flat.max_element();
    let exposure = (MOST_FLAT_LIGHT / strongest_channel.max(MOST_FLAT_LIGHT)).min(1.0);
    let shadows = brightest(&sky.bodies);
    for (&BodyLight(number), mut light, mut transform, mut visibility) in &mut lights {
        let now = sky.bodies.get(number).filter(|now| now.flat_light() > 0.0);
        let Some(now) = now else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);
        let [east, up, south] = now.direction.map(|v| v as f32);
        // World east is +X and south +Z.
        *transform = Transform::IDENTITY.looking_to(Vec3::new(-east, -up, -south), Vec3::Y);
        let [red, green, blue] = now.light.map(|v| v as f32);
        let strongest = red.max(green).max(blue);
        light.color = Color::linear_rgb(red / strongest, green / strongest, blue / strongest);
        light.illuminance = FLAT_LUX * strongest * exposure;
        let casts = number == shadows;
        if light.shadow_maps_enabled != casts {
            light.shadow_maps_enabled = casts;
        }
    }

    if let Some(mut ambient) = ambient {
        // White under the stars, the light's colour by day.
        let day = strongest_channel.min(1.0);
        let tint = if strongest_channel > 0.0 {
            Vec3::ONE.lerp(sky.flat / strongest_channel, day)
        } else {
            Vec3::ONE
        };
        ambient.color = Color::linear_rgb(tint.x, tint.y, tint.z);
        ambient.brightness = AMBIENT_BRIGHTNESS;
    }

    // The edge faces are unlit, so their colours match the prototype's: they
    // only need dimming, or they would glow at night.
    if let Some(mut material) = materials.get_mut(&faces.0) {
        let lit = sky.lit();
        material.base_color = Color::linear_rgb(lit.x, lit.y, lit.z);
    }
}
