//! Lightning on a world's map: whether it flashes at a place, by its weather,
//! and where a bolt strikes.

use shared::protocol::Strike;
use shared::rng::Seed;
use shared::weather::LIGHTNING_RAIN;
use shared::wilderness::SCALE;
use shared::world::MINUTES_PER_HOUR;

use super::ServerWorld;

/// A bolt never strikes within this many world map points of a poi, so never
/// in a poi's cutout in the wilderness, nor within this many cells of a World
/// Gate's (its frame and the ground round it).
const POI_CLEAR: i64 = 4;
const GATE_CLEAR: i64 = 4;

/// Spots tried for a bolt before giving up (the flash still happens).
const STRIKE_TRIES: u64 = 8;

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

    /// The lightning area a world map point is in (`WeatherField::lightning_area`).
    pub(crate) fn lightning_area(&self, x: i64, y: i64) -> (i64, i64) {
        self.weather.lightning_area(x, y)
    }

    /// Where a bolt strikes near `near` (in wilderness cells), within
    /// `reach` cells each way, rolled from its lightning area and `salt` (the
    /// second): a few spots are tried, and one too near anyone in `players`
    /// (their cell and how far to keep from it), a poi or a World Gate is
    /// passed over. `None` if every one was.
    pub(crate) fn strike_near(
        &self,
        near: (i64, i64),
        reach: i64,
        salt: i64,
        players: &[(i64, i64, i64)],
    ) -> Option<Strike> {
        let cells = self.width * SCALE;
        let (area_x, area_y) =
            self.lightning_area(near.0.div_euclid(SCALE), near.1.div_euclid(SCALE));
        let at = self
            .params
            .seed
            .derive("strikes")
            .at(&[area_x, area_y, salt]);
        let mut rng = Seed::new(at).rng();
        let mut roll = || rng.range_u64(0, 2 * reach as u64 + 1) as i64 - reach;
        for _ in 0..STRIKE_TRIES {
            let x = (near.0 + roll()).rem_euclid(cells);
            let y = near.1 + roll();
            let near_player = players
                .iter()
                .any(|&(px, py, clear)| apart(px - x, cells).max((py - y).abs()) <= clear);
            if !near_player && !self.near_poi(x, y) {
                let seed = Seed::new(at).derive("shape").at(&[x, y]);
                return Some(Strike { x, y, seed });
            }
        }
        None
    }

    /// Whether a poi is within `POI_CLEAR` points of a cell, or a World Gate's
    /// cell within `GATE_CLEAR` cells.
    fn near_poi(&self, x: i64, y: i64) -> bool {
        let (px, py) = (x.div_euclid(SCALE), y.div_euclid(SCALE));
        let reach = (px - POI_CLEAR, py - POI_CLEAR);
        let to = (px + POI_CLEAR, py + POI_CLEAR);
        let middle = |point: i64| point * SCALE + SCALE / 2;
        !self.pois.in_area(reach, to).is_empty()
            || self.pois.world_gates().gates.iter().any(|gate| {
                let dx = apart(middle(gate.x) - x, self.width * SCALE);
                dx.max((middle(gate.y) - y).abs()) <= GATE_CLEAR
            })
    }
}

/// How far apart two places `dx` apart east to west are, the shorter way
/// round a planet `width` across.
pub(crate) fn apart(dx: i64, width: i64) -> i64 {
    let dx = dx.rem_euclid(width);
    dx.min(width - dx)
}

#[cfg(test)]
mod tests;
