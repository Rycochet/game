//! The colour of one grid point on the globe's picture, for each layer.

use shared::surface::SurfaceSample;

use super::layer::{Layer, elevation_tint, moisture_tint, plate_tint, temperature_tint};
use super::planet::Planet;
use crate::surface_style::{SurfaceKind, SurfaceStyle};

/// How much darker trees make open land, when the ground is all woods.
const WOODS_DARKER: f32 = 0.3;
/// What a picture is coloured for: the layer, and the water (the sea at
/// mid-tide).
#[derive(Clone, Copy)]
pub(super) struct Colouring {
    pub(super) layer: Layer,
    pub(super) water: f64,
    /// Only the broad noise layers: finer ones would only speckle a picture
    /// whose pixels are larger than them.
    pub(super) broad: bool,
}

/// A grid point's colour, and its height in metres from the sea (the sea
/// itself flat), for the relief.
pub(super) fn colour_at(
    planet: &Planet,
    style: &SurfaceStyle,
    colouring: Colouring,
    (x, y): (i64, i64),
) -> ([u8; 4], f32) {
    let sample = if colouring.broad {
        SurfaceSample {
            height: planet.surface.broad_height(x, y),
            jitter: 0.0,
        }
    } else {
        planet.surface.sample(x, y)
    };
    let metres = planet.surface.metres(sample.height);
    let colour = match colouring.layer {
        Layer::Surface => surface_colour(planet, style, colouring, (x, y), sample),
        Layer::Elevation => elevation_tint(metres),
        Layer::Plates => plate_tint(planet.surface.plate_at(x, y)),
        Layer::Temperature => {
            temperature_tint(planet.climate.sample(x, y, sample.height).celsius())
        }
        Layer::Moisture => moisture_tint(planet.climate.sample(x, y, sample.height).moisture),
    };
    (colour, metres.max(0) as f32)
}

/// Its colour on level ground, as the world map has it, darker under trees.
fn surface_colour(
    planet: &Planet,
    style: &SurfaceStyle,
    colouring: Colouring,
    (x, y): (i64, i64),
    sample: SurfaceSample,
) -> [u8; 4] {
    let water = colouring.water;
    let climate = planet.climate.sample(x, y, sample.height);
    let mut colour = style.ground_colour(sample, climate, water, planet.surface.floor());
    if style.kind(sample, climate, water) == SurfaceKind::Land {
        let trees = planet.vegetation.cover(x, y, sample.height, climate).trees;
        let shade = 1.0 - WOODS_DARKER * trees.clamp(0.0, 1.0);
        for channel in &mut colour[..3] {
            *channel = (*channel as f32 * shade).round() as u8;
        }
    }
    colour
}
