//! A world's global settings, all generated from its seed.

use crate::rng::Seed;
use core::f64::consts::PI;

/// Settings that shape a whole world. Ported from the TypeScript prototype's
/// `IWorld`; the ranges are the same.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldParams {
    pub seed: Seed,
    /// Planet radius in metres, `500..6_500_000` (Earth is about 6_378_000).
    pub radius: u64,
    /// Distance around the equator in metres.
    pub circumference: u64,
    /// Days in a year, `100..365`.
    pub year: u64,
    /// How common ores are (and later gravity), `0.5..1.5`.
    pub density: f64,
    /// Axial tilt: strength of seasons and tides, `0.0..1.0`.
    pub tilt: f64,
    /// Height of the sea on the surface height scale (see `surface`), `20..60`.
    pub sea_level: f64,
    /// How wet the planet is, `0.0..1.0`.
    pub climate: f64,
}

impl WorldParams {
    pub fn from_seed(seed: Seed) -> Self {
        let radius = seed.derive("radius").rng().range_u64(500, 6_500_000);
        Self {
            seed,
            radius,
            circumference: (2.0 * PI * radius as f64) as u64,
            year: seed.derive("year").rng().range_u64(100, 365),
            density: seed.derive("density").rng().range_f64(0.5, 1.5),
            tilt: seed.derive("tilt").rng().range_f64(0.0, 1.0),
            sea_level: seed.derive("sealevel").rng().range_f64(20.0, 60.0),
            climate: seed.derive("climate").rng().range_f64(0.0, 1.0),
        }
    }

    /// How far the sea is above `sea_level` at a time of day (in hours).
    /// Two tides a day; a tilt of 0.25 gives 1 unit.
    pub fn tide(&self, hour: f64) -> f64 {
        libm::cos(hour * PI / 6.0) * 4.0 * self.tilt
    }

    /// The highest the water gets (high tide): ground above this is always
    /// dry land.
    pub fn high_water(&self) -> f64 {
        self.sea_level + 4.0 * self.tilt
    }

    /// The water height at a time of day.
    pub fn water_level(&self, hour: f64) -> f64 {
        self.sea_level + self.tide(hour)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_world() {
        let world = WorldParams::from_seed(Seed::new(12345));
        assert_eq!(format!("{world:?}"), GOLDEN_WORLD);
        assert_eq!(world.water_level(2.0), GOLDEN_WATER_AT_2);
    }

    const GOLDEN_WORLD: &str = "WorldParams { seed: Seed(12345), radius: 4239010, circumference: 26634485, year: 152, density: 1.2471431745178077, tilt: 0.38362863605768616, sea_level: 50.762015892761944, climate: 0.19909579997428872 }";
    const GOLDEN_WATER_AT_2: f64 = 51.52927316487732;
}
