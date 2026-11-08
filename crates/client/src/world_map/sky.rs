//! Clouds, the darkening below them, moving water and precipitation.
//!
//! `shared::weather` gives cloud cover and rain at any place and time. Here it
//! is sampled onto coarse textures around the player: two moments of the sky,
//! a second apart, which the shaders fade between, so it changes smoothly
//! however seldom it is worked out. Three shaders read them: the terrain
//! (`assets/shaders/terrain.wesl`), the cloud sheet (`clouds.wesl`) and
//! precipitation (`precipitation.wesl`). All purely visual.

// Split into files (`cargo xtask lint` keeps files small).
mod materials;
mod spawn;
mod update;

pub use materials::{CloudMaterial, PrecipitationMaterial, SkyParams, TerrainMaterial, TerrainSky};
pub use spawn::{SkyAssets, spawn};
pub use update::update;

use super::MAX_HALF;

/// Sky texture points each way, and the grid units between them: low
/// resolution, and light to work out. It covers more than the window, so the
/// player can walk a while before it moves.
const SKY_SIDE: usize = 100;
const SKY_SPACING: i64 = 10;

/// Seconds between the two moments of the sky.
const SKY_INTERVAL: f64 = 1.0;

/// Grid units the player can be from a texture's centre before it is worked
/// out again at once, rather than at the next moment (teleports).
const SKY_REACH: i64 = 180;

/// Grid units the precipitation's pattern repeats over: a copy of `DROP_TILE`
/// in `precipitation.wesl`, for the check below.
const DROP_TILE: f32 = 600.0;

// Checked when compiling: the sky texture still covers the largest window's
// corners when the player is as far from its centre as it allows, and the
// drops' pattern is wider than that window, so neither stretches nor repeats.
const SKY_COVERS: f32 = ((SKY_SIDE as i64 / 2) * SKY_SPACING - SKY_REACH) as f32;
const _: () = assert!(SKY_COVERS >= MAX_HALF * std::f32::consts::SQRT_2);
const _: () = assert!(DROP_TILE / 2.0 >= MAX_HALF * std::f32::consts::SQRT_2);

/// Height of the cloud sheet, above the highest ground there is.
pub(super) const CLOUD_HEIGHT: f32 = 170.0;

/// How much of the ground's own colour is added round the player, in the dark.
/// None in full light, where it would only clip.
const PLAYER_GLOW: f32 = 0.35;

/// Quads of precipitation over the shader's `DROP_TILE`, shown in proportion to
/// how hard it is falling. About half are over the window.
const DROPS: u32 = 6000;
