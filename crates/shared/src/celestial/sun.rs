//! Rolling suns and their colours.

use super::{
    Body, BodyKind, COLOUR_DRAWS, COMPANION_BRIGHTEST, COMPANION_DIMMEST, COMPANION_SPEED_SPREAD,
    ColourName, DAY_HOURS, LEAST_COLOUR_DISTANCE, SUN_COLOURS, SUN_LARGEST, SUN_SMALLEST,
};
use crate::rng::Rng;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Sun {
    pub(super) body: Body,
    pub(super) strength: f64,
    /// Hours between one of its noons and the next.
    pub(super) period: f64,
    /// An hour at which it is highest.
    pub(super) noon: f64,
}

pub(super) fn new_sun(rng: &mut Rng, number: i64, main_temperature: f64, others: &[Sun]) -> Sun {
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
pub(super) fn sun_colour(temperature: f64, brightness: f64) -> [f64; 3] {
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

pub(super) fn colour_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}
