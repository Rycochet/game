//! A world's seeds and its global settings.
//!
//! Each world has one secret seed, kept by the server. Everything else comes
//! from it one way (hashed, so it can't be worked back): the public terrain
//! seed the client generates the world map from, the secret seeds for
//! pois and sites, and the world's public id.

use crate::celestial::Celestial;
use crate::rng::Seed;
use crate::surface::{Surface, SurfaceGenerator};
use core::f64::consts::PI;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod time;
pub use time::{ClockRate, WorldTime};

/// The world map generator's version (`WorldParams`, `surface`). Bump it only
/// when the generated terrain changes in a way that matters, not for
/// cosmetic changes: existing worlds keep the version they were made with.
pub const WORLD_MAP_VERSION: u32 = 6;

/// The furthest the tide reaches, on the height scale (10 m), for the most
/// tilted planet (owner: realistic tides).
const TIDE_REACH: f64 = 0.1;

/// How often each generator is rolled, in percent (owner): `Desert` rarely,
/// `Continents` most, `Plates` the rest.
const DESERT_PERCENT: u64 = 1;
const CONTINENTS_PERCENT: u64 = 70;

/// A desert world's `climate` (wetness) and `warmth` ranges: dry and warm.
const DESERT_CLIMATE: (f64, f64) = (0.0, 0.1);
const DESERT_WARMTH: (f64, f64) = (0.25, 0.35);
/// How far below the lowest of its sampled ground a desert world's sea is,
/// so only the rare deepest hollows hold water (owner: almost none).
const DESERT_SEA_BELOW: f64 = 0.5;

/// Which generator a world's land is made by.
fn roll_generator(seed: Seed) -> SurfaceGenerator {
    match seed.derive("generator").rng().range_u64(0, 100) {
        roll if roll < DESERT_PERCENT => SurfaceGenerator::Desert,
        roll if roll < DESERT_PERCENT + CONTINENTS_PERCENT => SurfaceGenerator::Continents,
        _ => SurfaceGenerator::Plates,
    }
}

/// The smallest planet's radius in metres (version 4: was 500).
pub const SMALLEST_RADIUS: u64 = 1_500_000;

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

    /// The seed itself: for the server, and for the Hall's gates, whose
    /// planets' seeds are their addresses.
    pub const fn value(self) -> u64 {
        self.0.value()
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
    /// Planet radius in metres, `1_500_000..6_500_000` (Earth is about
    /// 6_378_000): no smaller, so every planet has room for all its World
    /// Gates (`shared::world_gate`).
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
    /// How its surface is made (`shared::surface`).
    pub generator: SurfaceGenerator,
}

/// Real seconds in a game minute, by the server's clock. Clients use it to run
/// smoothly between the server's answers.
pub const GAME_MINUTE_SECONDS: u64 = 1;

/// Game minutes in a game hour. The clock counts minutes; the tide, weather and
/// the sky's rates go by hours.
pub const MINUTES_PER_HOUR: f64 = 60.0;

/// Real seconds in a game hour.
pub const GAME_HOUR_SECONDS: u64 = 60 * GAME_MINUTE_SECONDS;

impl WorldParams {
    /// The world for a terrain seed, with the generator it rolls (or
    /// `HOW_DEBUG_GENERATOR`'s).
    pub fn from_seed(seed: Seed) -> Self {
        let generator = SurfaceGenerator::forced().unwrap_or_else(|| roll_generator(seed));
        Self::with_generator(seed, generator)
    }

    /// The world a terrain seed would be with another generator, as View
    /// Planet shows it: what depends on the generator (a desert's dry, warm
    /// climate and low sea) follows it.
    pub fn with_generator(seed: Seed, generator: SurfaceGenerator) -> Self {
        let radius = seed
            .derive("radius")
            .rng()
            .range_u64(SMALLEST_RADIUS, 6_500_000);
        let desert = generator == SurfaceGenerator::Desert;
        let (climate, warmth) = if desert {
            (DESERT_CLIMATE, DESERT_WARMTH)
        } else {
            ((0.0, 1.0), (-0.1, 0.1))
        };
        let mut world = Self {
            seed,
            radius,
            circumference: (2.0 * PI * radius as f64) as u64,
            year: seed.derive("year").rng().range_u64(100, 365),
            density: seed.derive("density").rng().range_f64(0.5, 1.5),
            tilt: seed.derive("tilt").rng().range_f64(0.0, 1.0),
            sea_level: seed.derive("sealevel").rng().range_f64(20.0, 60.0),
            climate: seed.derive("climate").rng().range_f64(climate.0, climate.1),
            warmth: seed.derive("warmth").rng().range_f64(warmth.0, warmth.1),
            generator,
        };
        if desert {
            // Below nearly all the ground: only the deepest hollows hold water.
            world.sea_level = Surface::new(&world).lowest_sampled() - DESERT_SEA_BELOW;
        }
        world
    }

    /// How far the sea is above `sea_level` at a time (in hours): the moons'
    /// pull (`Celestial::tide_pull`) times the tides' reach, `TIDE_REACH ×
    /// tilt` (at most 10 m). With no moons it stays at `sea_level`.
    pub fn tide(&self, celestial: &Celestial, hour: f64) -> f64 {
        celestial.tide_pull(hour) * self.tide_reach()
    }

    /// How far the tide reaches above or below `sea_level`, on the height scale.
    pub fn tide_reach(&self) -> f64 {
        TIDE_REACH * self.tilt
    }

    /// The highest the water gets (high tide): ground above this is always
    /// dry land.
    pub fn high_water(&self) -> f64 {
        self.sea_level + self.tide_reach()
    }

    /// The water height at a time (in hours), with the moons' tide.
    pub fn water_level(&self, celestial: &Celestial, hour: f64) -> f64 {
        self.sea_level + self.tide(celestial, hour)
    }
}

#[cfg(test)]
mod tests;
