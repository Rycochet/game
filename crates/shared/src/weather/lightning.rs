//! Lightning in the heaviest rain: whether it flashes at a place in a given
//! second. The server asks once a second for each player and tells those it
//! flashes for, so everyone in the same storm sees the same flashes.

use super::{WeatherField, WeatherSample};

/// Rain above this can bring lightning: the top of the heaviest band.
pub const LIGHTNING_RAIN: f64 = 0.85;

/// Below this temperature what falls is sleet or snow, which brings none. The
/// same as the shaders' sleet threshold (`precipitation.wesl`).
pub const LIGHTNING_WARMEST: f64 = 0.3;

/// Places in the same square this many grid points across share their
/// flashes, so players near each other see the same storm.
pub const LIGHTNING_AREA: i64 = 32;

/// The chance in a thousand of a flash in a second, in the worst of a storm
/// (rain 1). It falls to none at `LIGHTNING_RAIN`.
const MOST_PER_MILLE: f64 = 90.0;

impl WeatherField {
    /// Whether lightning flashes during this whole `second` of the server's
    /// clock, at a grid position with this weather and temperature
    /// (`climate`'s, `0..=1`).
    pub fn lightning(
        &self,
        x: i64,
        y: i64,
        weather: WeatherSample,
        temperature: f64,
        second: i64,
    ) -> bool {
        if weather.rain <= LIGHTNING_RAIN || temperature < LIGHTNING_WARMEST {
            return false;
        }
        let storm = (weather.rain - LIGHTNING_RAIN) / (1.0 - LIGHTNING_RAIN);
        let per_mille = (storm.min(1.0) * MOST_PER_MILLE) as u64;
        let (area_x, area_y) = self.lightning_area(x, y);
        self.lightning.at(&[area_x, area_y, second]) % 1000 < per_mille
    }

    /// Which `LIGHTNING_AREA` square a grid position is in, whose places
    /// flash together (`x` wrapped round the planet).
    pub fn lightning_area(&self, x: i64, y: i64) -> (i64, i64) {
        let area_x = x.rem_euclid(self.width as i64).div_euclid(LIGHTNING_AREA);
        (area_x, y.div_euclid(LIGHTNING_AREA))
    }
}

#[cfg(test)]
mod tests;
