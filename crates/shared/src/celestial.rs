//! A world's suns and moons, and where they are at any time. Only the terrain
//! seed and the server's clock go in, so the client and server agree on them.
//!
//! They only make light: nothing here draws anything. Directions are seen
//! from the ground, in `[east, up, south]`. See `docs/context/shared/celestial.md`.

use crate::rng::{Rng, Seed};
use core::f64::consts::{PI, TAU};

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

/// A sun or a moon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyKind {
    Sun,
    Moon,
}

/// A colour by name, for words such as "under the light of the blue moon".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColourName {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
}

impl ColourName {
    /// The name of a colour, or `None` if it is too near white to have one.
    fn of([red, green, blue]: [f64; 3]) -> Option<Self> {
        let most = red.max(green).max(blue);
        let least = red.min(green).min(blue);
        if most <= 0.0 || (most - least) / most < PALEST_NAMED {
            return None;
        }
        let span = most - least;
        let hue = if most == red {
            60.0 * ((green - blue) / span).rem_euclid(6.0)
        } else if most == green {
            60.0 * ((blue - red) / span + 2.0)
        } else {
            60.0 * ((red - green) / span + 4.0)
        };
        Some(match hue {
            h if h < 15.0 => Self::Red,
            h if h < 42.0 => Self::Orange,
            h if h < 70.0 => Self::Yellow,
            h if h < 165.0 => Self::Green,
            h if h < 260.0 => Self::Blue,
            h if h < 340.0 => Self::Violet,
            _ => Self::Red,
        })
    }
}

/// Where a moon is in its phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

/// One of a world's suns or moons, whatever the time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub kind: BodyKind,
    /// Its colour, linear RGB with the strongest channel at 1.
    pub colour: [f64; 3],
    /// Its colour's name, `None` if it is too near white to have one.
    pub colour_name: Option<ColourName>,
    /// How big it is: suns about `0.5..2`, moons `0.3..1`.
    pub size: f64,
}

/// One of a world's suns or moons at one time. `Celestial::at` gives these in
/// the order of `Celestial::bodies`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyNow {
    pub body: Body,
    /// A unit vector towards it, `[east, up, south]`.
    pub direction: [f64; 3],
    /// How far a moon's face we see is lit, `0..=1`. Suns are always 1.
    pub lit: f64,
    /// Whether a moon's lit part is growing. Never for suns.
    pub waxing: bool,
    /// How far along its way across the sky it is, whichever way that goes on the
    /// world: negative before its highest point, positive after, always rising.
    pub along: f64,
    /// The light it gives now, linear RGB, faded near the horizon: zero below
    /// it. Noon's main sun gives its colour on flat ground, so more than 1 here.
    pub light: [f64; 3],
}

impl BodyNow {
    /// Whether it is above the horizon.
    pub fn is_up(&self) -> bool {
        self.direction[1] > 0.0
    }

    /// A moon's phase; `None` for a sun.
    pub fn phase(&self) -> Option<MoonPhase> {
        if self.body.kind == BodyKind::Sun {
            return None;
        }
        Some(if self.lit < NEW_MOON_BELOW {
            MoonPhase::New
        } else if self.lit > 1.0 - NEW_MOON_BELOW {
            MoonPhase::Full
        } else {
            let quarter = (self.lit - 0.5).abs() < QUARTER_WITHIN;
            match (self.waxing, quarter, self.lit < 0.5) {
                (true, true, _) => MoonPhase::FirstQuarter,
                (false, true, _) => MoonPhase::LastQuarter,
                (true, _, true) => MoonPhase::WaxingCrescent,
                (true, _, false) => MoonPhase::WaxingGibbous,
                (false, _, false) => MoonPhase::WaningGibbous,
                (false, _, true) => MoonPhase::WaningCrescent,
            }
        })
    }

    /// The strongest channel of its light on flat ground.
    pub fn flat_light(&self) -> f64 {
        let [red, green, blue] = self.light;
        red.max(green).max(blue) * self.direction[1].max(0.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Sun {
    body: Body,
    strength: f64,
    /// Hours between one of its noons and the next.
    period: f64,
    /// An hour at which it is highest.
    noon: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct Moon {
    body: Body,
    /// Two unit vectors at right angles, across its orbit.
    across: [[f64; 3]; 2],
    /// Hours to go round.
    period: f64,
    /// Fraction of an orbit at hour 0.
    start: f64,
}

/// A world's suns and moons.
#[derive(Clone, Debug, PartialEq)]
pub struct Celestial {
    /// Hours each day the main sun is above the horizon.
    daylight_hours: f64,
    /// Whether suns rise in the east, rather than the west.
    rises_in_east: bool,
    /// How far south of straight overhead suns pass (negative: north).
    lean: f64,
    /// The main sun first: it sets the day, and noon is at hour 12.
    suns: Vec<Sun>,
    moons: Vec<Moon>,
}

impl Celestial {
    /// From a world's terrain seed: public, so clients work this out themselves.
    pub fn from_seed(seed: Seed) -> Self {
        let seed = seed.derive("celestial");
        let mut rng = seed.derive("sky").rng();
        let daylight_hours = rng.range_f64(LEAST_DAYLIGHT_HOURS, MOST_DAYLIGHT_HOURS);
        let rises_in_east = rng.next_f64() < 0.5;
        let lean =
            rng.range_f64(LEAST_LEAN, MOST_LEAN) * if rng.next_f64() < 0.5 { 1.0 } else { -1.0 };

        let chance = rng.next_f64();
        let count = if chance < THREE_SUNS {
            3
        } else if chance < THREE_SUNS + TWO_SUNS {
            2
        } else {
            1
        };
        let mut suns: Vec<Sun> = Vec::new();
        let main_temperature = rng.next_f64();
        for number in 0..count {
            suns.push(new_sun(
                &mut Seed::new(seed.derive("sun").at(&[number])).rng(),
                number,
                main_temperature,
                &suns,
            ));
        }
        // The main sun's strength puts its colour on flat ground at noon, so
        // the lean doesn't dim a world.
        let rise = rise_of(daylight_hours);
        let peak = 1.0 + rise;
        suns[0].strength = (peak * peak + lean * lean).sqrt() / peak;

        let mut moon_rng = seed.derive("moons").rng();
        let moons = (0..moon_count(&mut moon_rng))
            .map(|number| new_moon(&mut Seed::new(seed.derive("moon").at(&[number])).rng()))
            .collect();

        Self {
            daylight_hours,
            rises_in_east,
            lean,
            suns,
            moons,
        }
    }

    /// Every sun, then every moon.
    pub fn bodies(&self) -> Vec<Body> {
        let suns = self.suns.iter().map(|sun| sun.body);
        suns.chain(self.moons.iter().map(|moon| moon.body))
            .collect()
    }

    /// Where every sun and moon is at a time, in (fractional) game hours, and
    /// the light it gives. Everything that wants to know about them should
    /// come through here, so what is said matches what is seen.
    pub fn at(&self, hours: f64) -> Vec<BodyNow> {
        let suns_at = |hours: f64| -> Vec<BodyNow> {
            self.suns
                .iter()
                .map(|sun| self.sun_at(sun, hours))
                .collect()
        };
        let suns = suns_at(hours);
        let suns_later = suns_at(hours + WAXING_LOOK_AHEAD);
        let moons: Vec<BodyNow> = self
            .moons
            .iter()
            .map(|moon| moon_at(moon, &suns, &suns_later, hours))
            .collect();
        suns.into_iter().chain(moons).collect()
    }

    fn sun_at(&self, sun: &Sun, hours: f64) -> BodyNow {
        let angle = TAU * (hours - sun.noon) / sun.period;
        let across = libm::sin(angle);
        let east = if self.rises_in_east { -across } else { across };
        // A quarter turn of a sine over the time it is up, as how far east it is
        // comes back a little before a long day's end.
        let from_noon = (angle + PI).rem_euclid(TAU) - PI;
        let half_day = PI * self.daylight_hours / DAY_HOURS;
        let along = libm::sin((from_noon / half_day).clamp(-1.0, 1.0) * PI / 2.0);
        let up = libm::cos(angle) + rise_of(self.daylight_hours);
        let direction = unit([east, up, self.lean]);
        BodyNow {
            body: sun.body,
            direction,
            lit: 1.0,
            waxing: false,
            along,
            light: scaled(sun.body.colour, sun.strength * fade(direction)),
        }
    }
}

impl Celestial {
    /// Developer tools: changes the colour of one of `bodies`.
    pub fn set_colour(&mut self, number: usize, colour: [f64; 3]) {
        let suns = self.suns.len();
        let body = if number < suns {
            self.suns.get_mut(number).map(|sun| &mut sun.body)
        } else {
            self.moons.get_mut(number - suns).map(|moon| &mut moon.body)
        };
        if let Some(body) = body {
            body.colour = colour;
            body.colour_name = ColourName::of(colour);
        }
    }

    /// How strong one of `bodies` is, if a sun: the main sun gives its colour at
    /// noon at about 1.
    pub fn sun_strength(&self, number: usize) -> f64 {
        self.suns.get(number).map_or(0.0, |sun| sun.strength)
    }

    /// Developer tools: changes how strong one of `bodies` is, if a sun.
    pub fn set_sun_strength(&mut self, number: usize, strength: f64) {
        if let Some(sun) = self.suns.get_mut(number) {
            sun.strength = strength;
        }
    }
}

/// A colour for this kind of body, `0..=1` along every one it can pick: suns
/// from blue to orange, moons round the hues. Generated bodies can be paler.
pub fn colour_along(kind: BodyKind, along: f64) -> [f64; 3] {
    match kind {
        BodyKind::Sun => sun_colour(along, 0.0),
        BodyKind::Moon => from_hue(along * 6.0, MOON_RICHEST),
    }
}

/// The light on flat ground from all of these together, linear RGB.
pub fn flat_light(bodies: &[BodyNow]) -> [f64; 3] {
    let mut total = [0.0; 3];
    for now in bodies {
        let height = now.direction[1].max(0.0);
        for (channel, light) in total.iter_mut().zip(now.light) {
            *channel += light * height;
        }
    }
    total
}

/// Which of these lights flat ground the most: the first (the main sun) if
/// nothing does.
pub fn brightest(bodies: &[BodyNow]) -> usize {
    let mut best = 0;
    for (i, now) in bodies.iter().enumerate() {
        if now.flat_light() > bodies[best].flat_light() {
            best = i;
        }
    }
    best
}

/// How far above its horizon-crossing level a sun's path is raised for the
/// day's length: 0 for half a day of light, more for longer days.
fn rise_of(daylight_hours: f64) -> f64 {
    -libm::cos(PI * daylight_hours / DAY_HOURS)
}

fn fade(direction: [f64; 3]) -> f64 {
    let t = (direction[1] / FADE_HEIGHT).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn new_sun(rng: &mut Rng, number: i64, main_temperature: f64, others: &[Sun]) -> Sun {
    let mut temperature = if number == 0 {
        main_temperature
    } else {
        rng.next_f64()
    };
    let mut brightness = rng.next_f64();
    let different = |colour: [f64; 3]| {
        others
            .iter()
            .all(|other| colour_distance(other.body.colour, colour) >= LEAST_COLOUR_DISTANCE)
    };
    for _ in 0..COLOUR_DRAWS {
        if different(sun_colour(temperature, brightness)) {
            break;
        }
        temperature = rng.next_f64();
        brightness = rng.next_f64();
    }
    // Very unlikely: take whichever pure end is furthest from the others.
    if !different(sun_colour(temperature, brightness)) {
        brightness = 0.0;
        temperature = if others[0].body.colour[0] < 0.8 {
            1.0
        } else {
            0.0
        };
    }
    let colour = sun_colour(temperature, brightness);
    let main = number == 0;
    Sun {
        body: Body {
            kind: BodyKind::Sun,
            colour,
            colour_name: ColourName::of(colour),
            size: SUN_SMALLEST
                + (SUN_LARGEST - SUN_SMALLEST) * (temperature * 0.8 + rng.next_f64() * 0.2),
        },
        strength: COMPANION_DIMMEST + (COMPANION_BRIGHTEST - COMPANION_DIMMEST) * brightness,
        period: if main {
            DAY_HOURS
        } else {
            DAY_HOURS * (1.0 + (temperature - main_temperature) * COMPANION_SPEED_SPREAD)
        },
        noon: if main {
            DAY_HOURS / 2.0
        } else {
            rng.next_f64() * DAY_HOURS
        },
    }
}

/// A sun's colour: blue to orange by temperature, `0..=1`, drawn towards white
/// by brightness, `0..=1`.
fn sun_colour(temperature: f64, brightness: f64) -> [f64; 3] {
    let mut colour = SUN_COLOURS[0].1;
    for pair in SUN_COLOURS.windows(2) {
        let ((from_at, from), (to_at, to)) = (pair[0], pair[1]);
        if temperature >= from_at {
            let t = ((temperature - from_at) / (to_at - from_at)).min(1.0);
            colour = [0, 1, 2].map(|i| from[i] + (to[i] - from[i]) * t);
        }
    }
    colour.map(|channel| channel + (1.0 - channel) * brightness)
}

fn colour_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn moon_count(rng: &mut Rng) -> i64 {
    let mut roll = rng.range_u64(0, MOON_CHANCES.iter().sum());
    for (count, &chance) in MOON_CHANCES.iter().enumerate() {
        if roll < chance {
            return count as i64;
        }
        roll -= chance;
    }
    1
}

fn new_moon(rng: &mut Rng) -> Moon {
    // Any axis, so any direction round the sky, but not lying flat.
    let up = rng.range_f64(-MOON_AXIS_UP, MOON_AXIS_UP);
    let turn = rng.next_f64() * TAU;
    let flat = (1.0 - up * up).sqrt();
    let axis = [flat * libm::cos(turn), up, flat * libm::sin(turn)];
    let first = unit(cross(axis, [0.0, 1.0, 0.0]));

    let hue = rng.next_f64() * 6.0;
    let richness = rng.next_f64() * MOON_RICHEST;
    let colour = from_hue(hue, richness);
    Moon {
        body: Body {
            kind: BodyKind::Moon,
            colour,
            colour_name: ColourName::of(colour),
            size: rng.range_f64(MOON_SMALLEST, MOON_LARGEST),
        },
        across: [first, cross(axis, first)],
        period: rng.range_f64(MOON_FASTEST, MOON_SLOWEST),
        start: rng.next_f64(),
    }
}

/// A colour at `hue` (`0..6`: red, yellow, green, cyan, blue, magenta) whose
/// weakest channel is `richness` below 1.
fn from_hue(hue: f64, richness: f64) -> [f64; 3] {
    let channel = |at: f64| {
        let distance = (hue - at).rem_euclid(6.0);
        let distance = distance.min(6.0 - distance);
        1.0 - richness * (distance - 1.0).clamp(0.0, 1.0)
    };
    [channel(0.0), channel(2.0), channel(4.0)]
}

/// The way a moon is going at the top of its orbit.
fn moon_way_over(moon: &Moon) -> [f64; 3] {
    let axis = unit(cross(moon.across[0], moon.across[1]));
    let up = axis[1];
    let top = unit([-axis[0] * up, 1.0 - up * up, -axis[2] * up]);
    cross(axis, top)
}

fn moon_direction(moon: &Moon, hours: f64) -> [f64; 3] {
    let angle = TAU * (hours / moon.period + moon.start);
    let (sine, cosine) = (libm::sin(angle), libm::cos(angle));
    let [a, b] = moon.across;
    [0, 1, 2].map(|i| a[i] * cosine + b[i] * sine)
}

/// How far a moon's face is lit: by the sun lighting most of it.
fn lit_by(direction: [f64; 3], suns: &[BodyNow]) -> f64 {
    suns.iter()
        .map(|sun| (1.0 - dot(direction, sun.direction)) / 2.0)
        .fold(0.0, f64::max)
}

fn moon_at(moon: &Moon, suns: &[BodyNow], suns_later: &[BodyNow], hours: f64) -> BodyNow {
    let direction = moon_direction(moon, hours);
    let later = moon_direction(moon, hours + WAXING_LOOK_AHEAD);

    // Each sun lights the side of the moon facing it: all of what we see when
    // the sun is behind us, none when it is behind the moon. Suns light it
    // whether or not they are above our horizon.
    let reflected = moon.body.size * moon.body.size * MOONLIGHT * fade(direction);
    let mut light = [0.0; 3];
    let lit = lit_by(direction, suns);
    for sun in suns {
        let facing = (1.0 - dot(direction, sun.direction)) / 2.0;
        for (i, channel) in light.iter_mut().enumerate() {
            *channel += sun.body.colour[i] * moon.body.colour[i] * facing * reflected;
        }
    }
    BodyNow {
        body: moon.body,
        direction,
        lit,
        waxing: lit_by(later, suns_later) > lit,
        along: dot(direction, moon_way_over(moon)),
        light,
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    scaled(v, 1.0 / dot(v, v).sqrt())
}

fn scaled(v: [f64; 3], by: f64) -> [f64; 3] {
    v.map(|channel| channel * by)
}

#[cfg(test)]
mod tests;
