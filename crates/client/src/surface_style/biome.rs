//! Each biome's colour, and HSL to RGB.

use shared::climate::Biome;

/// How a biome looks. `rotation` is how much of the world's base hue rotation
/// it takes: living colours all of it, mineral ones (sand, bare earth) little,
/// so a purple-leaved world still has sand-coloured deserts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BiomeColour {
    pub(super) hue: f64,
    pub(super) saturation: f64,
    pub(super) lightness: f64,
    pub(super) rotation: f64,
}

impl BiomeColour {
    pub(super) fn mix(self, other: Self, t: f64) -> Self {
        let mix = |a: f64, b: f64| a + (b - a) * t;
        Self {
            hue: mix(self.hue, other.hue),
            saturation: mix(self.saturation, other.saturation),
            lightness: mix(self.lightness, other.lightness),
            rotation: mix(self.rotation, other.rotation),
        }
    }
}

/// Each biome's colour. `SeasonalForest` is the prototype's grass.
pub(super) fn biome_colour(biome: Biome) -> BiomeColour {
    let (hue, saturation, lightness, rotation) = match biome {
        Biome::Tundra => (45.0, 0.15, 0.38, 0.3),
        Biome::BorealForest => (130.0, 0.30, 0.18, 1.0),
        Biome::Grassland => (48.0, 0.40, 0.38, 0.5),
        Biome::Woodland => (70.0, 0.35, 0.28, 0.8),
        Biome::SeasonalForest => (80.0, 0.40, 0.24, 1.0),
        Biome::TemperateRainforest => (135.0, 0.40, 0.18, 1.0),
        Biome::Desert => (42.0, 0.55, 0.50, 0.0),
        Biome::Savanna => (58.0, 0.45, 0.32, 0.6),
        Biome::TropicalRainforest => (110.0, 0.60, 0.18, 1.0),
    };
    BiomeColour {
        hue,
        saturation,
        lightness,
        rotation,
    }
}

/// Converts hue (`0..360`), saturation and lightness (`0..1`) to sRGB bytes.
/// Out-of-range lightness is clamped, as the browser canvas did.
pub fn hsl_to_rgba(hue: f64, saturation: f64, lightness: f64) -> [u8; 4] {
    let a = saturation * lightness.min(1.0 - lightness);
    let channel = |n: f64| {
        let k = (n + hue / 30.0) % 12.0;
        let value = lightness - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0);
        (value * 255.0).round().clamp(0.0, 255.0) as u8
    };
    [channel(0.0), channel(8.0), channel(4.0), 255]
}
