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

/// Height of the cloud sheet, above the highest ground there is.
const CLOUD_HEIGHT: f32 = 170.0;

/// How much of the ground's own colour is added round the player, in the dark.
/// None in full light, where it would only clip.
const PLAYER_GLOW: f32 = 0.35;

/// Quads of precipitation over the shader's `DROP_TILE`, shown in proportion to
/// how hard it is falling. About half are over the window.
const DROPS: u32 = 6000;
