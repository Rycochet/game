//! The world's moons as quests see them: how many, and whether one is in the
//! sky now (`shared::quest::Need::Moon`).

use shared::celestial::ColourName;

use super::ServerWorld;
use shared::world::MINUTES_PER_HOUR;

impl ServerWorld {
    /// How many moons the world has.
    pub(crate) fn moon_count(&self) -> u8 {
        self.celestial.moon_count().min(usize::from(u8::MAX)) as u8
    }

    /// Whether moon `moon` is above the horizon now.
    pub(crate) fn moon_up(&self, moon: u8) -> bool {
        let hours = self.minutes() / MINUTES_PER_HOUR;
        let index = self.celestial.sun_count() + usize::from(moon);
        (self.celestial.at(hours).get(index)).is_some_and(|now| now.is_up())
    }

    /// Moon `moon`'s colour, by name, if it has one.
    pub(crate) fn moon_colour(&self, moon: u8) -> Option<ColourName> {
        let index = self.celestial.sun_count() + usize::from(moon);
        (self.celestial.bodies().get(index)).and_then(|body| body.colour_name)
    }
}
