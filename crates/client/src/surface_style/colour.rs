//! The colour and height a point is drawn with.

use shared::climate::{Biome, ClimateSample};
use shared::surface::SurfaceSample;

use super::{
    BIOME_JITTER, BIOME_SLOPE_SHADE, Band, BiomeColour, PALETTE_STEPS, SHORE_HUE_TURN,
    SHORE_LIGHTNESS, SurfaceKind, SurfaceStyle, TUNDRA_FROM, TUNDRA_FULL, biome_colour,
    hsl_to_rgba,
};

impl SurfaceStyle {
    /// Band and lightness for a point, as the prototype worked them out.
    /// Land is shaded by its slope against `away_height`, the height a step
    /// further from the light, scaled by `slope_strength` (1.0 = prototype). Underwater points
    /// get lighter with height, and with ice as the snow line falls.
    pub fn shade(
        &self,
        sample: SurfaceSample,
        climate: ClimateSample,
        away_height: f64,
        water_level: f64,
        snow_line: f64,
        slope_strength: f64,
    ) -> (Band, f64) {
        let kind = self.kind(sample, climate, water_level, snow_line);
        if kind == SurfaceKind::Land {
            let colour = self.living_colour(climate, sample);
            let slope = (sample.height - away_height) / 64.0;
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
            let slope = (sample.height - away_height) / 64.0;
            let shade = slope * (2000.0 - band.saturation * 2000.0) * slope_strength;
            (64.0 - band.hue / 2.0 - shade) / 100.0
        } else {
            let ice = ((self.ice_start - snow_line) / self.ice_start).clamp(0.0, 1.0);
            (sample.height / 2.0 + 15.0) / 100.0 + ice
        };
        (band, lightness)
    }

    /// The plain colour of a point, as `shade` gives it on level ground: what
    /// a view showing that place close up paints its outside ground with.
    pub fn ground_colour(
        &self,
        sample: SurfaceSample,
        climate: ClimateSample,
        water_level: f64,
        snow_line: f64,
    ) -> [u8; 4] {
        let (band, lightness) =
            self.shade(sample, climate, sample.height, water_level, snow_line, 0.0);
        hsl_to_rgba(band.hue, band.saturation, lightness)
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
