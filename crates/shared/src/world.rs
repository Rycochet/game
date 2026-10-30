//! A world's seeds and its global settings.
//!
//! Each world has one secret seed, kept by the server. Everything else comes
//! from it one way (hashed, so it can't be worked back): the public terrain
//! seed the client generates the world map from, the secret seeds for
//! features and sites, and the world's public id.

use crate::rng::Seed;
use core::f64::consts::PI;
use serde::{Deserialize, Serialize};

/// The world map generator's version (`WorldParams`, `surface`). Bump it only
/// when the generated terrain changes in a way that matters, not for
/// cosmetic changes: existing worlds keep the version they were made with.
pub const WORLD_MAP_VERSION: u32 = 1;

/// A world's secret seed. Only the server has it (and saves it); never send
/// it to a client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSeed(Seed);

impl WorldSeed {
    pub const fn new(value: u64) -> Self {
        Self(Seed::new(value))
    }

    /// The public seed the world map terrain comes from (`WorldParams`).
    pub fn terrain(self) -> Seed {
        self.0.derive("terrain")
    }

    /// The secret seed world map features (and so their sites) come from.
    pub fn features(self) -> Seed {
        self.0.derive("features")
    }

    /// The world's public id: the same seed is the same world on any server.
    pub fn id(self) -> WorldId {
        WorldId(self.0.derive("id").value())
    }
}

/// Which world this is, safe to share: it identifies the world without
/// revealing its seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldId(pub u64);

/// Settings that shape a whole world. Ported from the TypeScript prototype's
/// `IWorld`; the ranges are the same.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldParams {
    /// The public terrain seed (`WorldSeed::terrain`) these came from.
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
    /// How much warmer than usual the planet is on average, `-0.1..0.1` of
    /// `shared::climate`'s temperature scale.
    pub warmth: f64,
}

/// The snow line in normal weather: above every height there is, so nothing is
/// frozen.
pub const NORMAL_SNOW_LINE: u32 = 128;

/// Real seconds in a game hour, by the server's clock. Clients use it to run
/// smoothly between the server's hours.
pub const GAME_HOUR_SECONDS: u64 = 60;

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
            warmth: seed.derive("warmth").rng().range_f64(-0.1, 0.1),
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

    // Golden values: if these change, every world gets a new terrain and id.
    #[test]
    fn golden_world_seed() {
        let seed = WorldSeed::new(12345);
        assert_eq!(seed.terrain().value(), GOLDEN_TERRAIN_SEED);
        assert_eq!(seed.id(), WorldId(GOLDEN_WORLD_ID));
        // Each purpose gets its own seed, none of them the secret one.
        let derived = [seed.terrain(), seed.features(), Seed::new(seed.id().0)];
        assert!(derived.iter().all(|&d| d != seed.0));
        assert!(derived[0] != derived[1] && derived[1] != derived[2]);
    }

    const GOLDEN_TERRAIN_SEED: u64 = 15274154617967521790;
    const GOLDEN_WORLD_ID: u64 = 13205059784687644427;

    const GOLDEN_WORLD: &str = "WorldParams { seed: Seed(12345), radius: 4239010, circumference: 26634485, year: 152, density: 1.2471431745178077, tilt: 0.38362863605768616, sea_level: 50.762015892761944, climate: 0.19909579997428872, warmth: -0.015898052726041195 }";
    const GOLDEN_WATER_AT_2: f64 = 51.52927316487732;
}
