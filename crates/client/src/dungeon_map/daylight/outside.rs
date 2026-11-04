//! The light outside at the site's place, and what follows it.

use bevy::prelude::*;
use shared::celestial::{brightest, flat_light};

use super::super::mesh::with_ambient;
use super::super::sight_lines::SurfaceMaterial;
use super::super::{CropPart, HerbPart, Surface, SurfaceMaterials, TOKEN_AMBIENT_BRIGHTNESS};
use super::{CLOUD_SHADOW, OPEN_AMBIENT_BRIGHTNESS};

/// How much darker remembered places are with Remembered Places set to Dim.
const MEMORY_DIM: f32 = 0.5;
use crate::common::{
    CurrentDungeon, CurrentWorld, GameClock, MapPosition, Preferences, Remembered, WorldReturn,
};
use crate::player::Player;

/// The light outside at the site's place.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::dungeon_map) struct OutsideLight {
    /// On open ground, linear RGB from 0 (only the dull ambient light) to 1.
    pub light: Vec3,
    /// The way the brightest light is going across the grid, longer the lower
    /// it is: 1 on the horizon, 0 straight overhead.
    pub travel: Vec2,
}

impl OutsideLight {
    /// How much to multiply what is out in the open and unlit by.
    pub fn lit(&self) -> Vec3 {
        with_ambient(self.light)
    }
}

/// Keeps the outside light up to date, for a new hour or anything changing the
/// world (the debug menu's panel), or arriving somewhere else.
pub(in crate::dungeon_map) fn update_outside(
    world: Option<Res<CurrentWorld>>,
    clock: Res<GameClock>,
    place: Res<WorldReturn>,
    mut outside: ResMut<OutsideLight>,
) {
    let Some(world) = world else {
        return;
    };
    if !(world.is_changed() || clock.is_changed() || place.is_changed()) {
        return;
    }
    let hours = f64::from(clock.hour);
    let bodies = world.celestial.at(hours);
    let cover = world.weather.sample(place.0.x, place.0.y, hours).cover;
    let flat = flat_light(&bodies).map(|channel| (channel * (1.0 - CLOUD_SHADOW * cover)) as f32);
    let [east, _, south] = bodies[brightest(&bodies)].direction;
    outside.set_if_neq(OutsideLight {
        light: Vec3::from_array(flat).min(Vec3::ONE),
        travel: -Vec2::new(east as f32, south as f32),
    });
}

/// What isn't baked into meshes' colours follows the light: remembered open
/// ground, which would otherwise be brighter than what is in sight at night, and
/// crops and plants. Remembered places are darker still with Remembered
/// Places set to Dim.
#[allow(clippy::type_complexity)]
pub(in crate::dungeon_map) fn tint_outside(
    preferences: Res<Preferences>,
    outside: Res<OutsideLight>,
    surfaces: Res<SurfaceMaterials>,
    mut materials: ResMut<Assets<SurfaceMaterial>>,
    plants: Query<&MeshMaterial3d<SurfaceMaterial>, Or<(With<CropPart>, With<HerbPart>)>>,
    mut dimmed: Local<f32>,
) {
    let dim = match preferences.video.remembered {
        Remembered::Dim => MEMORY_DIM,
        Remembered::Show | Remembered::Hide => 1.0,
    };
    if !outside.is_changed() && !(preferences.is_changed() && *dimmed != dim) {
        return;
    }
    *dimmed = dim;
    let lit = outside.lit();
    let tint = |material: &mut StandardMaterial, by: Vec3| {
        let alpha = material.base_color.alpha();
        material.base_color = Color::linear_rgba(by.x, by.y, by.z, alpha);
    };
    for surface in Surface::ALL {
        let outside = matches!(surface, Surface::Ground | Surface::Town);
        let by = if outside { lit.max_element() } else { 1.0 };
        if let Some(mut material) = materials.get_mut(&surfaces.memory[surface as usize]) {
            tint(&mut material.base, Vec3::splat(by * dim));
        }
    }
    for plant in &plants {
        if let Some(mut material) = materials.get_mut(&plant.0) {
            tint(&mut material.base, lit);
        }
    }
}

/// Tokens have more ambient light with the player out in the open or in a town,
/// following the light there, and the usual dim fill otherwise; with Brightness,
/// at least that share of what full daylight gives.
pub(in crate::dungeon_map) fn token_ambient(
    preferences: Res<Preferences>,
    outside: Res<OutsideLight>,
    dungeon: Res<CurrentDungeon>,
    player: Single<&MapPosition, With<Player>>,
    ambient: Option<ResMut<GlobalAmbientLight>>,
) {
    let Some(mut ambient) = ambient else {
        return;
    };
    let open = dungeon.is_town() || dungeon.level.cell(player.x, player.y).is_outside();
    let strongest = outside.light.max_element();
    let (brightness, tint) = if open && strongest > 0.0 {
        let extra = (OPEN_AMBIENT_BRIGHTNESS - TOKEN_AMBIENT_BRIGHTNESS) * strongest;
        (TOKEN_AMBIENT_BRIGHTNESS + extra, outside.light / strongest)
    } else {
        (TOKEN_AMBIENT_BRIGHTNESS, Vec3::ONE)
    };
    let brightness = brightness.max(OPEN_AMBIENT_BRIGHTNESS * preferences.video.least_light());
    let color = Color::linear_rgb(tint.x, tint.y, tint.z);
    if ambient.brightness != brightness || ambient.color != color {
        ambient.brightness = brightness;
        ambient.color = color;
    }
}
