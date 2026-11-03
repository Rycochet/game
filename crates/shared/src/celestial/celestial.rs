//! A world's suns and moons, rolled from its seed.

use super::{
    LEAST_DAYLIGHT_HOURS, LEAST_LEAN, MOST_DAYLIGHT_HOURS, MOST_LEAN, Moon, Sun, THREE_SUNS,
    TWO_SUNS, moon_count, new_moon, new_sun, rise_of,
};
use crate::rng::Seed;

/// A world's suns and moons.
#[derive(Clone, Debug, PartialEq)]
pub struct Celestial {
    /// Hours each day the main sun is above the horizon.
    pub(super) daylight_hours: f64,
    /// Whether suns rise in the east, rather than the west.
    pub(super) rises_in_east: bool,
    /// How far south of straight overhead suns pass (negative: north).
    pub(super) lean: f64,
    /// The main sun first: it sets the day, and noon is at hour 12.
    pub(super) suns: Vec<Sun>,
    pub(super) moons: Vec<Moon>,
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
}
