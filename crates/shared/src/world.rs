//! A world's seeds and its global settings.
//!
//! Each world has one secret seed, kept by the server. Everything else comes
//! from it one way (hashed, so it can't be worked back): the public terrain
//! seed the client generates the world map from, the secret seeds for
//! pois and sites, and the world's public id.

use crate::rng::Seed;
use core::f64::consts::PI;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The world map generator's version (`WorldParams`, `surface`). Bump it only
/// when the generated terrain changes in a way that matters, not for
/// cosmetic changes: existing worlds keep the version they were made with.
pub const WORLD_MAP_VERSION: u32 = 3;

/// What world ids are hashed under. Changing it gives every world a new id.
const WORLD_ID_NAMESPACE: Uuid = Uuid::from_u128(0x6f1d_3a84_9c27_4e5b_b0d6_28e7_41ac_f953);

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

    /// The secret seed world map pois (and so their sites) come from.
    pub fn pois(self) -> Seed {
        self.0.derive("pois")
    }

    /// The world's public id: the same seed is the same world on any server. A
    /// version 5 UUID, a one-way hash of the seed, with no clock or randomness.
    pub fn id(self) -> WorldId {
        WorldId(Uuid::new_v5(
            &WORLD_ID_NAMESPACE,
            &self.0.value().to_le_bytes(),
        ))
    }
}

/// Which world this is, safe to share: it identifies the world without
/// revealing its seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldId(pub Uuid);

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

/// Real seconds in a game minute, by the server's clock. Clients use it to run
/// smoothly between the server's answers.
pub const GAME_MINUTE_SECONDS: u64 = 1;

/// Game minutes in a game hour. The clock counts minutes; the tide, weather and
/// the sky's rates go by hours.
pub const MINUTES_PER_HOUR: f64 = 60.0;

/// Real seconds in a game hour.
pub const GAME_HOUR_SECONDS: u64 = 60 * GAME_MINUTE_SECONDS;

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
mod tests;
