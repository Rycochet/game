//! Colours and thresholds for drawing a planet surface, used by every view.
//! Swap the `SurfaceStyle` resource to give other planets a different look.
//! See `docs/context/client/surface_style.md`.

use bevy::prelude::*;
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
    /// Mountain tops, and anything above the snow line.
    Mountain,
    Hill,
    Grass,
    Sand,
    Water,
}

#[derive(Resource, Clone, Debug)]
pub struct SurfaceStyle {
    /// Height (plus jitter) above which ground is mountain (snow-capped). The
    /// snow line can bring this lower, never higher.
    pub mountain_height: f64,
    /// Height (plus jitter) above which ground is hill.
    pub hill_height: f64,
    pub mountain: Band,
    pub hill: Band,
    pub grass: Band,
    /// Land just below the water: beaches.
    pub sand: Band,
    /// Fixed lightness for sand (other bands get theirs from height/slope).
    pub sand_lightness: f64,
    /// How much of the jitter can lift an underwater point into sand.
    /// Smaller means narrower beaches (1.0 was the prototype).
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
            mountain_height: 90.0,
            hill_height: 80.0,
            mountain: Band {
                hue: 0.0,
                saturation: 0.0,
            },
            hill: Band {
                hue: 36.0,
                saturation: 0.4,
            },
            grass: Band {
                hue: 80.0,
                saturation: 0.4,
            },
            sand: Band {
                hue: 57.0,
                saturation: 1.0,
            },
            sand_lightness: 0.5,
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
    pub fn kind(&self, sample: SurfaceSample, water_level: f64, snow_line: f64) -> SurfaceKind {
        let SurfaceSample { height, jitter } = sample;
        if height + jitter > snow_line.min(self.mountain_height) {
            SurfaceKind::Mountain
        } else if height + jitter > self.hill_height {
            SurfaceKind::Hill
        } else if height + jitter.abs() * self.shore_jitter > water_level {
            if height < water_level {
                SurfaceKind::Sand
            } else {
                SurfaceKind::Grass
            }
        } else {
            SurfaceKind::Water
        }
    }

    pub fn band(&self, kind: SurfaceKind) -> Band {
        match kind {
            SurfaceKind::Mountain => self.mountain,
            SurfaceKind::Hill => self.hill,
            SurfaceKind::Grass => self.grass,
            SurfaceKind::Sand => self.sand,
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
        west_height: f64,
        water_level: f64,
        snow_line: f64,
        slope_strength: f64,
    ) -> (Band, f64) {
        let kind = self.kind(sample, water_level, snow_line);
        let band = self.band(kind);
        let lightness = if kind == SurfaceKind::Sand {
            self.sand_lightness
        } else if self.is_frozen_water(sample, water_level, snow_line) {
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

    /// Whether a point is frozen water (the same rule as walking on it).
    pub fn is_frozen_water(&self, sample: SurfaceSample, water_level: f64, snow_line: f64) -> bool {
        sample.is_frozen(water_level, snow_line)
    }

    /// The height a point should be drawn at: anything below the water is
    /// drawn at the water surface (coloured by depth by `band`), as the
    /// prototype did. This keeps shore sand, which is just below the water by
    /// definition, visible at the waterline.
    pub fn shown_height(&self, sample: SurfaceSample, water_level: f64) -> f64 {
        sample.height.max(water_level)
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
}
