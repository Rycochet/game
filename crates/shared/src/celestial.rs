//! A world's suns and moons, and where they are at any time. Only the terrain
//! seed and the server's clock go in, so the client and server agree on them.
//!
//! They only make light: nothing here draws anything. Directions are seen
//! from the ground, in `[east, up, south]`. See `docs/context/shared/celestial.md`.

// Split into files (`cargo xtask lint` keeps files small).
mod body;
#[allow(clippy::module_inception)]
mod celestial;
mod colour_name;
mod dev_tools;
mod light;
mod moon;
mod moon_path;
mod now;
mod sun;
mod vector;

pub use body::{Body, BodyKind, BodyNow, MoonPhase};
pub use celestial::Celestial;
pub use colour_name::ColourName;
pub use light::{brightest, colour_along, flat_light};
use moon::{Moon, from_hue, moon_count, new_moon};
use moon_path::moon_at;
use now::{fade, rise_of};
use sun::{Sun, new_sun, sun_colour};
use vector::{cross, dot, scaled, unit};

/// Hours in a day, by the main sun.
const DAY_HOURS: f64 = 24.0;

/// Hours of the main sun above the horizon: `MOST` is not reached.
const LEAST_DAYLIGHT_HOURS: f64 = 12.0;
const MOST_DAYLIGHT_HOURS: f64 = 16.0;

/// How far the suns' path leans north or south of straight overhead, as the
/// tangents of 15 and 35 degrees: so noon's light isn't straight down.
const LEAST_LEAN: f64 = 0.268;
const MOST_LEAN: f64 = 0.700;

/// Chances of a second and third sun (out of 1).
const THREE_SUNS: f64 = 0.001;
const TWO_SUNS: f64 = 0.05;

/// Chances of 0 to 5 moons (out of 100).
const MOON_CHANCES: [u64; 6] = [4, 40, 25, 15, 10, 6];

/// The most suns and moons a world can have.
pub const MOST_BODIES: usize = 3 + MOON_CHANCES.len() - 1;

/// A body's light fades in over this height above the horizon (the sine of
/// about 15 degrees, roughly an hour of the sun's climb), so it never comes on
/// suddenly.
const FADE_HEIGHT: f64 = 0.26;

/// The orange end of the suns' colours is this much slower than the blue end,
/// in fractions of a day either side of the main sun.
const COMPANION_SPEED_SPREAD: f64 = 0.5;

/// Companions' strength compared with the main sun, by brightness.
const COMPANION_DIMMEST: f64 = 0.3;
const COMPANION_BRIGHTEST: f64 = 0.7;

/// The least difference between two suns' colours (see `colour_distance`),
/// and draws to find it before settling for the far end of the range.
const LEAST_COLOUR_DISTANCE: f64 = 0.2;
const COLOUR_DRAWS: u32 = 32;

/// Sizes: suns grow from the blue end to the orange end. A moon's size also
/// decides how much light it reflects, by its square.
const SUN_SMALLEST: f64 = 0.5;
const SUN_LARGEST: f64 = 2.0;
const MOON_SMALLEST: f64 = 0.3;
const MOON_LARGEST: f64 = 1.0;

/// What a full moon of size 1 reflects of a sun's light.
const MOONLIGHT: f64 = 0.2;

/// Moon orbits, in hours.
const MOON_FASTEST: f64 = 7.0;
const MOON_SLOWEST: f64 = 90.0;

/// How near a moon's orbit may be to lying flat: the sine of its axis's
/// height, so the moon always climbs at least 20 degrees rather than creeping
/// round the horizon.
const MOON_AXIS_UP: f64 = 0.94;

/// Game hours ahead to look, to tell whether a moon is waxing.
const WAXING_LOOK_AHEAD: f64 = 0.1;

/// How much of a moon's face counts as new or full, and how far from half counts
/// as a quarter.
const NEW_MOON_BELOW: f64 = 0.03;
const QUARTER_WITHIN: f64 = 0.03;

/// How strong a moon's colour can be: how far one channel can be below 1.
const MOON_RICHEST: f64 = 0.55;

/// Colours paler than this have no name (see `ColourName`).
const PALEST_NAMED: f64 = 0.12;

/// The suns' colours along their temperature: blue, white, yellow, orange.
/// Linear RGB. They pass through white as a direct blend of blue and yellow
/// would pass through green.
const SUN_COLOURS: [(f64, [f64; 3]); 4] = [
    (0.0, [0.55, 0.70, 1.0]),
    (0.4, [1.0, 1.0, 1.0]),
    (0.7, [1.0, 0.92, 0.6]),
    (1.0, [1.0, 0.68, 0.4]),
];

#[cfg(test)]
mod tests;
