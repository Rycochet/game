//! The climate of surfaces worked out in metres (`Continents`, `Plates`), from
//! the planet itself: how much sunlight each latitude gets in a year for the
//! world's tilt, how much its suns give, and ice past a fixed latitude.

use core::f64::consts::FRAC_PI_2;

use crate::celestial::{BodyKind, Celestial};
use crate::world::WorldParams;

/// The world's `tilt` (`0..1`) as an angle between these, in degrees: chosen
/// so ice comes naturally somewhere near `ICE_LATITUDE`. Earth's is 23.4.
const LEAST_TILT: f64 = 15.0;
const MOST_TILT: f64 = 35.0;
/// Temperature at sea level for average sunlight, in °C, and how much more a
/// latitude gets for each unit more sunlight than average. Fitted to Earth's
/// tilt: about 29 °C at the equator and -14 °C at the poles (heat carried by
/// winds and seas flattens what sunlight alone would make).
const AVERAGE: f64 = 15.0;
const PER_SUNLIGHT: f64 = 60.0;
/// Past this latitude, in degrees, there is always ice. Its edge wanders
/// this far either way with the local temperature, so it isn't a straight
/// line.
pub const ICE_LATITUDE: f64 = 80.0;
const ICE_WANDER: f64 = 3.0;
/// The warmest it may be at the poles, in °C, and how much warmer that limit
/// is for each degree of latitude nearer the equator: -8 °C (snow) at
/// `ICE_LATITUDE`, no limit to speak of by 60°.
const POLE_LIMIT: f64 = -20.0;
const LIMIT_RISE: f64 = 1.2;
/// How cold it is past `ICE_LATITUDE`.
const ICE_CAP: f64 = -40.0;
/// A companion sun's heat compared with a main sun of the same size.
const COMPANION_HEAT: f64 = 0.5;
/// How much the suns' heat moves temperatures, measured from absolute zero:
/// four times Earth's sun is about 25 °C warmer, a quarter about 23 °C colder.
const HEAT_POWER: f64 = 0.06;
const ABSOLUTE_ZERO: f64 = -273.15;
/// Below this yearly average at sea level, in °C, the sea is frozen.
pub const SEA_ICE: f64 = -6.0;

/// What a planet's sunlight is like.
pub(super) struct Sunlit {
    /// The tilt's part in how sunlight spreads from the equator to the poles
    /// over a year (the second Legendre term: -0.48 for Earth's tilt).
    spread: f64,
    /// How much the suns warm the planet compared with Earth's sun.
    heat: f64,
}

impl Sunlit {
    pub(super) fn new(world: &WorldParams) -> Self {
        let tilt = (LEAST_TILT + (MOST_TILT - LEAST_TILT) * world.tilt).to_radians();
        let cos = libm::cos(tilt);
        let heat = Celestial::from_seed(world.seed)
            .bodies()
            .iter()
            .filter(|body| body.kind == BodyKind::Sun)
            .enumerate()
            .map(|(number, sun)| {
                let share = if number == 0 { 1.0 } else { COMPANION_HEAT };
                share * sun.size * sun.size
            })
            .sum::<f64>()
            .max(0.01);
        Self {
            spread: -0.625 * (3.0 * cos * cos - 1.0) / 2.0,
            heat,
        }
    }

    /// The yearly average at sea level, in °C, at `latitude` (`-1..=1`, the
    /// poles at either end), before local differences.
    pub(super) fn sea_level(&self, latitude: f64) -> f64 {
        let sine = libm::sin(latitude * FRAC_PI_2);
        let sunlight = 1.0 + self.spread * (3.0 * sine * sine - 1.0) / 2.0;
        let earthlike = AVERAGE + PER_SUNLIGHT * (sunlight - 1.0);
        (earthlike - ABSOLUTE_ZERO) * libm::pow(self.heat, HEAT_POWER) + ABSOLUTE_ZERO
    }
}

/// The warmest it may be at `latitude` (`-1..=1`), in °C: always ice past
/// `ICE_LATITUDE` (moved by `local`, the temperature noise `-1..=1`, warmer
/// further), and limited approaching it.
pub(super) fn polar_limit(latitude: f64, local: f64) -> f64 {
    let degrees = latitude.abs() * 90.0 - ICE_WANDER * local;
    if degrees >= ICE_LATITUDE {
        ICE_CAP
    } else {
        POLE_LIMIT + LIMIT_RISE * (90.0 - degrees)
    }
}

/// The belts of rain round a planet, `-1..=1` by `latitude` (`-1..=1`): wet at
/// the equator where the air rises, dry about 30° out where it sinks (where
/// deserts are), wet again about 60°, dry at the poles.
pub fn moisture_belt(latitude: f64) -> f64 {
    libm::cos(6.0 * latitude * FRAC_PI_2)
}
