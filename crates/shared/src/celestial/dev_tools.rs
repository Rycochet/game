//! Developer tools: changing a body's colour and a sun's strength.

use super::{Celestial, ColourName};

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
