//! Temperature, moisture and the biome they make, at any world map position.
//!
//! Biomes follow the Whittaker diagram: a place's yearly temperature and
//! moisture decide what grows there. Both come from slow noise wrapped round
//! the same cylinder as the surface, so east and west still join up. Height
//! cools the air, so climbing a hill moves through colder biomes by itself.
//! See `docs/context/shared/climate.md`.

mod planet;

use crate::showcase;
use crate::surface::METRES_PER_UNIT;
use crate::world::WorldParams;
use core::f64::consts::TAU;
use fastnoise_lite::{FastNoiseLite, NoiseType};
pub use planet::{ICE_LATITUDE, SEA_ICE, moisture_belt};
use planet::{Sunlit, polar_limit};

/// Grid units per unit of noise space: climate regions are a few 400-point
/// windows across.
const ZOOM: f64 = 700.0;

/// Noise layers, each double the detail and half the strength.
const OCTAVES: u32 = 3;

/// Simplex noise bunches around 0; this spreads it so the extremes (deserts,
/// rainforests) are reasonably common.
const STRETCH: f64 = 1.5;

/// Temperature at sea level on the equator, with average noise.
const EQUATOR_TEMPERATURE: f64 = 0.75;
/// How much colder the poles are (a quarter of the way round the planet from
/// the equator, `y` = 0). Planets are huge compared with the view, so noise
/// does most of the local work.
const POLE_COOLING: f64 = 0.25;
/// How far noise moves the temperature each way.
const TEMPERATURE_NOISE: f64 = 0.28;
/// Cooling for every `COOLING_HEIGHT` above `COOLING_FROM`. By the height
/// itself rather than above the sea, so the snow is around the same height on
/// every world, however high its sea.
const HEIGHT_COOLING: f64 = 0.40;
const COOLING_HEIGHT: f64 = 50.0;
const COOLING_FROM: f64 = 40.0;
/// How far noise moves the moisture each way.
const MOISTURE_NOISE: f64 = 0.55;
/// How far the world's `climate` moves the moisture each way.
const WORLD_MOISTURE: f64 = 0.25;

/// The temperature scale in degrees Celsius: 0 is this cold, and each whole
/// unit this many degrees warmer, so `SNOW_TEMPERATURE` is about -8 °C and the
/// equator's sea level about 27 °C.
pub const CELSIUS_AT_ZERO: f64 = -10.5;
pub const CELSIUS_PER_UNIT: f64 = 50.0;
/// How much colder the air gets climbing a kilometre: Earth's average.
const LAPSE_RATE: f64 = 6.5;
/// For surfaces in metres, whose climate comes from the planet (see
/// `planet`): how far noise moves the temperature each way, in °C, and how
/// much the belts of rain and the noise move the moisture.
const METRES_TEMPERATURE_NOISE: f64 = 6.0;
const BELT_MOISTURE: f64 = 0.3;
const METRES_MOISTURE_NOISE: f64 = 0.22;

/// Below this temperature, land is covered in snow. With the numbers above,
/// on an average world and at an average place that is around height 90.
pub const SNOW_TEMPERATURE: f64 = 0.05;

/// Temperature and moisture at one place, both `0..=1`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClimateSample {
    pub temperature: f64,
    pub moisture: f64,
}

impl ClimateSample {
    pub fn biome(self) -> Biome {
        Biome::of(self.temperature, self.moisture)
    }

    /// The temperature in degrees Celsius.
    pub fn celsius(self) -> f64 {
        CELSIUS_AT_ZERO + self.temperature * CELSIUS_PER_UNIT
    }
}

/// What grows at a place: the Whittaker diagram's nine biomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    Tundra,
    BorealForest,
    Grassland,
    Woodland,
    SeasonalForest,
    TemperateRainforest,
    Desert,
    Savanna,
    TropicalRainforest,
}

impl Biome {
    pub const ALL: [Biome; 9] = [
        Biome::Tundra,
        Biome::BorealForest,
        Biome::Grassland,
        Biome::Woodland,
        Biome::SeasonalForest,
        Biome::TemperateRainforest,
        Biome::Desert,
        Biome::Savanna,
        Biome::TropicalRainforest,
    ];

    /// The biome for a temperature and moisture.
    pub fn of(temperature: f64, moisture: f64) -> Biome {
        if temperature < 0.2 {
            Biome::Tundra
        } else if temperature < 0.4 {
            if moisture < 0.3 {
                Biome::Tundra
            } else {
                Biome::BorealForest
            }
        } else if temperature < 0.7 {
            if moisture < 0.3 {
                Biome::Grassland
            } else if moisture < 0.5 {
                Biome::Woodland
            } else if moisture < 0.8 {
                Biome::SeasonalForest
            } else {
                Biome::TemperateRainforest
            }
        } else if moisture < 0.3 {
            Biome::Desert
        } else if moisture < 0.6 {
            Biome::Savanna
        } else {
            Biome::TropicalRainforest
        }
    }
}

/// Works out the climate for one world.
pub struct Climate {
    temperature: FastNoiseLite,
    moisture: FastNoiseLite,
    /// Grid units around the equator, where `x` wraps.
    width: i64,
    /// Added everywhere, from the world's `warmth`.
    warmth: f64,
    /// Moisture added everywhere, from the world's `climate`.
    wetness: f64,
    /// The planet's sunlight, for surfaces in metres.
    sunlit: Sunlit,
    /// Whether the world's surface is in metres, so its climate comes from
    /// the planet (`sample_metres`).
    in_metres: bool,
    /// The world's sea level on the height scale, where 0 m is.
    sea_level: f64,
}

impl Climate {
    pub fn new(world: &WorldParams) -> Self {
        let noise = |purpose: &str| {
            let mut noise = FastNoiseLite::with_seed(world.seed.derive(purpose).noise_seed());
            noise.set_noise_type(Some(NoiseType::OpenSimplex2));
            // We scale coordinates ourselves.
            noise.set_frequency(Some(1.0));
            noise
        };
        Self {
            temperature: noise("temperature"),
            moisture: noise("moisture"),
            width: (world.circumference / 1000).max(1) as i64,
            warmth: world.warmth,
            wetness: (world.climate - 0.5) * 2.0 * WORLD_MOISTURE,
            sunlit: Sunlit::new(world),
            in_metres: world.generator.in_metres(),
            sea_level: world.sea_level,
        }
    }

    /// The climate at a position whose ground is at `height` on the height
    /// scale, the way the world's generator has it: from the planet for a
    /// surface in metres (`sample_metres`), cooling with the height itself
    /// otherwise.
    pub fn sample(&self, x: i64, y: i64, height: f64) -> ClimateSample {
        if self.in_metres {
            let metres = ((height - self.sea_level) * METRES_PER_UNIT).round() as i32;
            return self.sample_metres(x, y, metres);
        }
        let climb = (height - COOLING_FROM).max(0.0) / COOLING_HEIGHT;
        self.cooled_by(x, y, HEIGHT_COOLING * climb)
    }

    /// The climate at a position whose ground is `metres` above the sea (the
    /// sea's surface for anything below it), for surfaces worked out in
    /// metres. It comes from the planet: sunlight by latitude for its tilt,
    /// its suns' heat, ice past `ICE_LATITUDE`, belts of rain; then the air
    /// cools at Earth's rate as it climbs from the sea.
    pub fn sample_metres(&self, x: i64, y: i64, metres: i32) -> ClimateSample {
        if showcase::in_plain(y) {
            return showcase::CLIMATE;
        }
        // The poles are a quarter of the way round from the equator.
        let latitude = (y as f64 / (self.width as f64 / 4.0)).clamp(-1.0, 1.0);
        let kilometres = metres.max(0) as f64 / 1000.0;
        let local = self.layered(&self.temperature, x, y);
        let celsius = self.sunlit.sea_level(latitude)
            + self.warmth * CELSIUS_PER_UNIT
            + METRES_TEMPERATURE_NOISE * local
            - LAPSE_RATE * kilometres;
        let celsius = celsius.min(polar_limit(latitude, local));
        let moisture = 0.5
            + self.wetness
            + BELT_MOISTURE * moisture_belt(latitude)
            + METRES_MOISTURE_NOISE * self.layered(&self.moisture, x, y);
        ClimateSample {
            temperature: ((celsius - CELSIUS_AT_ZERO) / CELSIUS_PER_UNIT).clamp(0.0, 1.0),
            moisture: moisture.clamp(0.0, 1.0),
        }
    }

    /// The climate at a position, `cooling` colder for its height.
    fn cooled_by(&self, x: i64, y: i64, cooling: f64) -> ClimateSample {
        if showcase::in_plain(y) {
            return showcase::CLIMATE;
        }
        // Poles are a quarter of the way round from the equator.
        let latitude = (y.abs() as f64 / (self.width as f64 / 4.0)).min(1.0);
        let temperature = EQUATOR_TEMPERATURE + self.warmth - POLE_COOLING * latitude
            + TEMPERATURE_NOISE * self.layered(&self.temperature, x, y)
            - cooling;
        let moisture = 0.5 + self.wetness + MOISTURE_NOISE * self.layered(&self.moisture, x, y);
        ClimateSample {
            temperature: temperature.clamp(0.0, 1.0),
            moisture: moisture.clamp(0.0, 1.0),
        }
    }

    /// Layered noise at a position, roughly evenly spread over `-1..=1`.
    fn layered(&self, noise: &FastNoiseLite, x: i64, y: i64) -> f64 {
        // As in `surface`: on a cylinder, one grid unit east the same noise
        // distance as one south, wrapped first to keep float precision.
        let angle = TAU * x.rem_euclid(self.width) as f64 / self.width as f64;
        let radius = self.width as f64 / (ZOOM * TAU);
        let (nx, ny, nz) = (
            libm::cos(angle) * radius,
            libm::sin(angle) * radius,
            y as f64 / ZOOM,
        );
        let mut total = 0.0;
        for octave in 0..OCTAVES {
            let scale = (1u32 << octave) as f64;
            total += noise.get_noise_3d(nx * scale, ny * scale, nz * scale) as f64 / scale;
        }
        (total * STRETCH).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests;
