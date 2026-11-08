//! Whether lightning flashes at a place on a world's map, by its weather.

use shared::weather::LIGHTNING_RAIN;
use shared::world::MINUTES_PER_HOUR;

use super::ServerWorld;

impl ServerWorld {
    /// Whether lightning flashes at this world map point during this whole
    /// second of the server's clock, worked out as clients work out the rain
    /// they draw there (at sea level's temperature, as `world_map::sky` does).
    pub(crate) fn lightning_at(&self, x: i64, y: i64, second: i64) -> bool {
        let hours = self.minutes() / MINUTES_PER_HOUR;
        let weather = self.weather.sample(x, y, hours);
        if weather.rain <= LIGHTNING_RAIN {
            // Most places: no need to work out the climate.
            return false;
        }
        let temperature = self.climate.sample(x, y, self.params.sea_level);
        self.weather
            .lightning(x, y, weather, temperature.temperature, second)
    }
}
