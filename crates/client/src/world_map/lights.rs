//! The world map's lights: one `DirectionalLight` per sun and moon (only the
//! brightest casting shadows, as the Shadows setting allows), the ambient fill
//! (raised by Brightness), and the parts of the scene that aren't lit by them
//! following along.

use bevy::light::{DirectionalLightShadowMap, light_consts};
use bevy::prelude::*;
use shared::celestial::{BodyKind, BodyNow, MOST_BODIES, brightest};

use super::map_sky::MapSky;
use super::{EdgeFaces, WorldMapScene};
use crate::common::{Lightning, NIGHT_LIGHT, Preferences, Shadows};

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

/// Suns are drawn this much brighter than `shared::celestial` says (owner's
/// choice), on screen only; moons as they are. `MOST_FLAT_LIGHT` still holds,
/// so a lone noon sun gains a little less.
const SUN_BOOST: f32 = 1.2;

/// WebGL 2 takes only one directional light, and Bevy keeps the ones casting no
/// shadows first, so it would drop the brightest: there, only that one is shown.
const ONE_LIGHT: bool = cfg!(target_arch = "wasm32");

/// Lightning's light, white with a little blue, linear RGB.
pub(super) const FLASH_COLOUR: Vec3 = Vec3::new(0.85, 0.9, 1.0);

/// The ambient fill a full flash of lightning adds: several times what a noon
/// sun gives, so everything goes nearly white.
const FLASH_AMBIENT: f32 = 12000.0;

/// How bright unlit things (the edge faces, clouds, rain) are in a full flash.
pub(super) const FLASH_LIT: f32 = 2.5;

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

/// Shadow map sizes for the Shadows setting's Low and High.
const SHADOW_MAP_LOW: usize = 1024;
const SHADOW_MAP_HIGH: usize = 2048;

/// Every frame in this view: puts the lights where the suns and moons are.
#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    sky: Res<MapSky>,
    preferences: Res<Preferences>,
    lightning: Res<Lightning>,
    shadow_map: Option<ResMut<DirectionalLightShadowMap>>,
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
    let sky = &sky.0;
    let graphics = &preferences.graphics;
    if let Some(mut map) = shadow_map {
        let size = match graphics.shadows {
            Shadows::Low => SHADOW_MAP_LOW,
            Shadows::Off | Shadows::High => SHADOW_MAP_HIGH,
        };
        if map.size != size {
            map.size = size;
        }
    }
    let strongest_channel = sky.flat.max_element();
    let shadows = brightest(&sky.bodies);
    let shown = |number: usize| !ONE_LIGHT || number == shadows;
    let mut drawn_flat = Vec3::ZERO;
    for (_, now) in sky.bodies.iter().enumerate().filter(|(n, _)| shown(*n)) {
        let height = now.direction[1].max(0.0) as f32;
        drawn_flat += Vec3::from(now.light.map(|v| v as f32)) * height * boost(now);
    }
    let drawn_strongest = drawn_flat.max_element();
    let exposure = (MOST_FLAT_LIGHT / drawn_strongest.max(MOST_FLAT_LIGHT)).min(1.0);
    for (&BodyLight(number), mut light, mut transform, mut visibility) in &mut lights {
        let now = sky.bodies.get(number).filter(|now| now.flat_light() > 0.0);
        let now = now.filter(|_| shown(number));
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
        light.illuminance = FLAT_LUX * strongest * boost(now) * exposure;
        let casts = number == shadows && graphics.shadows != Shadows::Off;
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
        let tint = tint.lerp(FLASH_COLOUR, lightning.0);
        ambient.color = Color::linear_rgb(tint.x, tint.y, tint.z);
        // Brightness: at least that share of what a noon sun gives.
        let least = AMBIENT_BRIGHTNESS * graphics.least_light() / NIGHT_LIGHT;
        ambient.brightness = AMBIENT_BRIGHTNESS.max(least) + FLASH_AMBIENT * lightning.0;
    }

    // The edge faces are unlit, so their colours match the prototype's: they
    // only need dimming, or they would glow at night.
    if let Some(mut material) = materials.get_mut(&faces.0) {
        let lit = sky.lit().max(Vec3::splat(graphics.least_light()));
        let lit = lit.lerp(FLASH_COLOUR * FLASH_LIT, lightning.0);
        material.base_color = Color::linear_rgb(lit.x, lit.y, lit.z);
    }
}

/// How much brighter this body is drawn than its light: `SUN_BOOST` for suns.
fn boost(now: &BodyNow) -> f32 {
    match now.body.kind {
        BodyKind::Sun => SUN_BOOST,
        BodyKind::Moon => 1.0,
    }
}
