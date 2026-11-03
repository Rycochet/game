//! Colours and thresholds for drawing a planet surface, used by every view.
//! Swap the `SurfaceStyle` resource to give other planets a different look.
//! See `docs/context/client/surface_style.md`.

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod band;
mod biome;
mod colour;
mod style;

pub use band::{Band, SurfaceKind};
pub use biome::hsl_to_rgba;
use biome::{BiomeColour, biome_colour};
pub use style::SurfaceStyle;

pub struct SurfaceStylePlugin;

impl Plugin for SurfaceStylePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SurfaceStyle>();
    }
}

/// How finely the Whittaker diagram is cut for blending: each cell takes one
/// biome's colour, and colours blend between neighbouring cells' centres.
const PALETTE_STEPS: f64 = 10.0;

/// How much warmer than snow, and how far below the snow line, rivers freeze
/// once they are `RIVER_FREEZE_INLAND` above the water. At the water, only
/// `RIVER_FREEZE_AT_SEA` below the snow line, about when the shallows freeze.
const RIVER_FREEZE_TEMPERATURE: f64 = 0.07;
const RIVER_FREEZE_HEIGHT: f64 = 8.0;
const RIVER_FREEZE_INLAND: f64 = 16.0;
const RIVER_FREEZE_AT_SEA: f64 = 4.0;

/// Heights where land starts going to tundra's colours, and is all tundra.
/// Snow is around 90.
const TUNDRA_FROM: f64 = 76.0;
const TUNDRA_FULL: f64 = 84.0;

/// The shore is the biome's colour turned this many degrees round the colour
/// wheel (the way red goes to yellow), and this much of its lightness, so it
/// marks the tide in every biome and with any base hue.
const SHORE_HUE_TURN: f64 = 30.0;
const SHORE_LIGHTNESS: f64 = 0.8;

/// How far a unit of surface jitter (`-8..8`) moves a point on the diagram.
const BIOME_JITTER: f64 = 0.008;

/// Slope shading on open land, the same in every biome (the prototype's grass
/// strength), so relief reads equally everywhere.
const BIOME_SLOPE_SHADE: f64 = 1200.0;

#[cfg(test)]
mod tests;
