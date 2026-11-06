//! The planet's surface style: which kind each point shows, its band, frozen rivers.

use bevy::prelude::*;
use shared::climate::{ClimateSample, SNOW_TEMPERATURE};
use shared::surface::SurfaceSample;

use super::{BIOME_JITTER, Band, RIVER_FREEZE_INLAND, RIVER_FREEZE_TEMPERATURE, SurfaceKind};

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
    /// Lightness of frozen water (`SurfaceSample::is_frozen`): flat, no
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
            ice_lightness: 0.933,
            earth_face: (38.0, 1.0, 0.15),
        }
    }
}

impl SurfaceStyle {
    /// Which kind of surface a point shows: land colder than snow is snow.
    pub fn kind(
        &self,
        sample: SurfaceSample,
        climate: ClimateSample,
        water_level: f64,
    ) -> SurfaceKind {
        let SurfaceSample { height, jitter } = sample;
        let cold = climate.temperature + jitter * BIOME_JITTER < SNOW_TEMPERATURE;
        if height > water_level && cold {
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
    /// is snow, the more so the higher up.
    pub fn river_is_frozen(&self, temperature: f64, ground: f64, water_level: f64) -> bool {
        let inland = ((ground - water_level) / RIVER_FREEZE_INLAND).clamp(0.0, 1.0);
        temperature < SNOW_TEMPERATURE + RIVER_FREEZE_TEMPERATURE * inland
    }

    pub fn band(&self, kind: SurfaceKind) -> Band {
        match kind {
            // Land's and shore's colours come from the biome (`living_colour`);
            // the band only matters when asked for by kind alone.
            SurfaceKind::Snow | SurfaceKind::Land | SurfaceKind::Shore => self.snow,
            SurfaceKind::Water => self.water,
        }
    }
}
