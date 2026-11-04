//! How the moons pull the sea: what the tide follows.

use super::{Celestial, moon_direction};

impl Celestial {
    /// How far the moons pull the sea up (`1`) or down (`-1`) at a time, in
    /// (fractional) game hours: their heights in the sky, from the nadir (`-1`)
    /// to the zenith (`1`), averaged by mass (`size³`), so heavier moons pull
    /// harder. Moons below the horizon pull too. Moons high together make the
    /// highest tides and cancel out when one is up and another down; one whose
    /// orbit never climbs far only ever makes gentle tides. With no moons the
    /// sea stays at mid-tide (`0`).
    pub fn tide_pull(&self, hours: f64) -> f64 {
        let mut pull = 0.0;
        let mut mass = 0.0;
        for moon in &self.moons {
            let weight = moon.body.size * moon.body.size * moon.body.size;
            pull += moon_direction(moon, hours)[1] * weight;
            mass += weight;
        }
        if mass > 0.0 { pull / mass } else { 0.0 }
    }
}
