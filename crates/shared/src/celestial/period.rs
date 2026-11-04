//! How long suns and moons take to go round.

use super::Celestial;

impl Celestial {
    /// Hours one of `bodies` takes to go round once: a sun's day across the
    /// sky, a moon's orbit. `None` past the last body.
    pub fn period(&self, number: usize) -> Option<f64> {
        let suns = self.suns.len();
        if number < suns {
            Some(self.suns[number].period)
        } else {
            self.moons.get(number - suns).map(|moon| moon.period)
        }
    }
}
