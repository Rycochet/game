//! Rolling moons: how many, how fast and their colours.

use core::f64::consts::TAU;

use super::{
    Body, BodyKind, ColourName, MOON_AXIS_UP, MOON_CHANCES, MOON_FASTEST, MOON_LARGEST,
    MOON_RICHEST, MOON_SLOWEST, MOON_SMALLEST, cross, unit,
};
use crate::rng::Rng;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Moon {
    pub(super) body: Body,
    /// Two unit vectors at right angles, across its orbit.
    pub(super) across: [[f64; 3]; 2],
    /// Hours to go round.
    pub(super) period: f64,
    /// Fraction of an orbit at hour 0.
    pub(super) start: f64,
}

pub(super) fn moon_count(rng: &mut Rng) -> i64 {
    let mut roll = rng.range_u64(0, MOON_CHANCES.iter().sum());
    for (count, &chance) in MOON_CHANCES.iter().enumerate() {
        if roll < chance {
            return count as i64;
        }
        roll -= chance;
    }
    1
}

pub(super) fn new_moon(rng: &mut Rng) -> Moon {
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
pub(super) fn from_hue(hue: f64, richness: f64) -> [f64; 3] {
    let channel = |at: f64| {
        let distance = (hue - at).rem_euclid(6.0);
        let distance = distance.min(6.0 - distance);
        1.0 - richness * (distance - 1.0).clamp(0.0, 1.0)
    };
    [channel(0.0), channel(2.0), channel(4.0)]
}
