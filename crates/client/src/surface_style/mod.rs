//! Colours and thresholds for drawing a planet surface, used by every view.
//! Swap the `SurfaceStyle` resource to give other planets a different look.
//! See `docs/context/client/surface_style.md`.

use bevy::prelude::*;
use shared::climate::{Biome, ClimateSample, SNOW_TEMPERATURE};
use shared::surface::SurfaceSample;

pub struct SurfaceStylePlugin;

impl Plugin for SurfaceStylePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SurfaceStyle>();
    }
}

/// A colour as hue (degrees, `0..360`) and saturation (`0..1`). Lightness is
/// worked out per pixel from height and slope.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub hue: f64,
    pub saturation: f64,
}

/// What a surface point shows, which decides its colour band.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceKind {
    /// Land cold enough for snow, and anything above the snow line.
    Snow,
    /// Open land, coloured by its biome.
    Land,
    /// Land just below the water, showing where the tide has reached.
    Shore,
    Water,
}

#[derive(Resource, Clone, Debug)]
pub struct SurfaceStyle {
    pub snow: Band,
    /// Degrees added to biome hues, as much as each biome takes. 0 is green
    /// plants; later chosen per world, with green the most likely.
    pub base_hue_rotation: f64,
    /// How much of the jitter can lift an underwater point into shore.
    /// Smaller means a narrower shore (1.0 was the prototype).
    pub shore_jitter: f64,
    pub water: Band,
    /// Snow line below which water starts to ice over (turning icy blue).
    pub ice_start: f64,
    /// Lightness of frozen water (snow line below the point): flat, no
    /// shading. 0.933 is `#eee`.
    pub ice_lightness: f64,
    /// Colour (hue, saturation, lightness) of the cut-away earth under the
    /// front face.
    pub earth_face: (f64, f64, f64),
}

impl Default for SurfaceStyle {
    /// Earth-like colours from the TypeScript prototype.
    fn default() -> Self {
        Self {
            snow: Band {
                hue: 0.0,
                saturation: 0.0,
            },
            base_hue_rotation: 0.0,
            shore_jitter: 0.5,
            water: Band {
                hue: 232.0,
                saturation: 1.0,
            },
            ice_start: 64.0,
            ice_lightness: 0.933,
            earth_face: (38.0, 1.0, 0.15),
        }
    }
}

impl SurfaceStyle {
    /// Which kind of surface a point shows. `snow_line` is from
    /// `common::Weather` (`NORMAL_SNOW_LINE` normally).
    pub fn kind(
        &self,
        sample: SurfaceSample,
        climate: ClimateSample,
        water_level: f64,
        snow_line: f64,
    ) -> SurfaceKind {
        let SurfaceSample { height, jitter } = sample;
        let cold = climate.temperature + jitter * BIOME_JITTER < SNOW_TEMPERATURE;
        if height + jitter > snow_line || (height > water_level && cold) {
            SurfaceKind::Snow
        } else if height + jitter.abs() * self.shore_jitter > water_level {
            if height < water_level {
                SurfaceKind::Shore
            } else {
                SurfaceKind::Land
            }
        } else {
            SurfaceKind::Water
        }
    }

    /// Whether a river is ice here. Rivers freeze before the land around them
    /// is snow, the more so the higher up: where one meets the sea, it goes
    /// about when the sea's shallows do.
    pub fn river_is_frozen(
        &self,
        temperature: f64,
        ground: f64,
        water_level: f64,
        snow_line: f64,
    ) -> bool {
        let inland = ((ground - water_level) / RIVER_FREEZE_INLAND).clamp(0.0, 1.0);
        temperature < SNOW_TEMPERATURE + RIVER_FREEZE_TEMPERATURE * inland
            || ground + RIVER_FREEZE_AT_SEA + RIVER_FREEZE_HEIGHT * inland > snow_line
    }

    pub fn band(&self, kind: SurfaceKind) -> Band {
        match kind {
            // Land's and shore's colours come from the biome (`living_colour`);
            // the band only matters when asked for by kind alone.
            SurfaceKind::Snow | SurfaceKind::Land | SurfaceKind::Shore => self.snow,
            SurfaceKind::Water => self.water,
        }
    }

    /// Band and lightness for a point, as the prototype worked them out.
    /// Land is shaded by its slope towards the western neighbour (lit from the
    /// east), scaled by `slope_strength` (1.0 = prototype). Underwater points
    /// get lighter with height, and with ice as the snow line falls.
    pub fn shade(
        &self,
        sample: SurfaceSample,
        climate: ClimateSample,
        west_height: f64,
        water_level: f64,
        snow_line: f64,
        slope_strength: f64,
    ) -> (Band, f64) {
        let kind = self.kind(sample, climate, water_level, snow_line);
        if kind == SurfaceKind::Land {
            let colour = self.living_colour(climate, sample);
            let slope = (sample.height - west_height) / 64.0;
            let shade = slope * BIOME_SLOPE_SHADE * slope_strength / 100.0;
            return (
                Band {
                    hue: colour.hue,
                    saturation: colour.saturation,
                },
                colour.lightness - shade,
            );
        }
        if kind == SurfaceKind::Shore {
            let colour = self.living_colour(climate, sample);
            return (
                Band {
                    hue: (colour.hue + SHORE_HUE_TURN).rem_euclid(360.0),
                    saturation: colour.saturation,
                },
                colour.lightness * SHORE_LIGHTNESS,
            );
        }
        let band = self.band(kind);
        let lightness = if self.is_frozen_water(sample, water_level, snow_line) {
            self.ice_lightness
        } else if sample.height > water_level {
            let slope = (sample.height - west_height) / 64.0;
            let shade = slope * (2000.0 - band.saturation * 2000.0) * slope_strength;
            (64.0 - band.hue / 2.0 - shade) / 100.0
        } else {
            let ice = ((self.ice_start - snow_line) / self.ice_start).clamp(0.0, 1.0);
            (sample.height / 2.0 + 15.0) / 100.0 + ice
        };
        (band, lightness)
    }

    /// The colour of open land at a climate: a smooth blend of the biome
    /// colours around it on the diagram, so biomes merge into each other.
    /// `jitter` roughens where they meet. The base hue rotation is applied
    /// last, as much as each biome takes.
    fn living_colour(&self, climate: ClimateSample, sample: SurfaceSample) -> BiomeColour {
        let SurfaceSample { height, jitter } = sample;
        let t = (climate.temperature + jitter * BIOME_JITTER) * PALETTE_STEPS - 0.5;
        let m = (climate.moisture + jitter * BIOME_JITTER) * PALETTE_STEPS - 0.5;
        let (t_cell, m_cell) = (t.floor(), m.floor());
        let (t_mix, m_mix) = (t - t_cell, m - m_cell);
        let at = |t_offset: f64, m_offset: f64| {
            let centre = |cell: f64| ((cell + 0.5) / PALETTE_STEPS).clamp(0.0, 1.0);
            biome_colour(Biome::of(
                centre(t_cell + t_offset),
                centre(m_cell + m_offset),
            ))
        };
        let dry = at(0.0, 0.0).mix(at(1.0, 0.0), t_mix);
        let wet = at(0.0, 1.0).mix(at(1.0, 1.0), t_mix);
        // High ground goes to tundra's colours whatever grows below it.
        let climb = (height + jitter - TUNDRA_FROM) / (TUNDRA_FULL - TUNDRA_FROM);
        let colour = dry
            .mix(wet, m_mix)
            .mix(biome_colour(Biome::Tundra), climb.clamp(0.0, 1.0));
        BiomeColour {
            hue: (colour.hue + self.base_hue_rotation * colour.rotation).rem_euclid(360.0),
            ..colour
        }
    }

    /// Whether a point is frozen water (the same rule as walking on it).
    pub fn is_frozen_water(&self, sample: SurfaceSample, water_level: f64, snow_line: f64) -> bool {
        sample.is_frozen(water_level, snow_line)
    }

    /// The height a point should be drawn at: anything below the water is
    /// drawn at the water surface (coloured by depth by `band`), as the
    /// prototype did. This keeps the shore, which is just below the water by
    /// definition, visible at the waterline.
    pub fn shown_height(&self, sample: SurfaceSample, water_level: f64) -> f64 {
        sample.height.max(water_level)
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

/// How a biome looks. `rotation` is how much of the world's base hue rotation
/// it takes: living colours all of it, mineral ones (sand, bare earth) little,
/// so a purple-leaved world still has sand-coloured deserts.
#[derive(Clone, Copy, Debug, PartialEq)]
struct BiomeColour {
    hue: f64,
    saturation: f64,
    lightness: f64,
    rotation: f64,
}

impl BiomeColour {
    fn mix(self, other: Self, t: f64) -> Self {
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
fn biome_colour(biome: Biome) -> BiomeColour {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_matches_known_colours() {
        assert_eq!(hsl_to_rgba(0.0, 0.0, 0.55), [140, 140, 140, 255]); // #8c8c8c
        assert_eq!(hsl_to_rgba(232.0, 1.0, 0.29), [0, 20, 148, 255]); // ~#001493
        assert_eq!(hsl_to_rgba(0.0, 1.0, 0.5), [255, 0, 0, 255]);
    }

    #[test]
    fn shore_is_the_biome_colour_turned_and_darkened() {
        let style = SurfaceStyle::default();
        let climate = ClimateSample {
            temperature: 0.6,
            moisture: 0.65,
        };
        let shade = |height| {
            let sample = SurfaceSample {
                height,
                jitter: 2.0,
            };
            style.shade(sample, climate, height, 50.0, 128.0, 0.0)
        };
        let (land, land_lightness) = shade(50.5);
        let (shore, shore_lightness) = shade(49.5);
        assert_eq!(shore.hue, land.hue + SHORE_HUE_TURN);
        assert_eq!(shore.saturation, land.saturation);
        assert!((shore_lightness - land_lightness * SHORE_LIGHTNESS).abs() < 1e-9);
    }
}
