//! Names for the colours of suns and moons.

use serde::{Deserialize, Serialize};

use super::PALEST_NAMED;

/// A colour by name, for words such as "under the light of the blue moon".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColourName {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
}

impl ColourName {
    /// The name of a colour, or `None` if it is too near white to have one.
    pub(super) fn of([red, green, blue]: [f64; 3]) -> Option<Self> {
        let most = red.max(green).max(blue);
        let least = red.min(green).min(blue);
        if most <= 0.0 || (most - least) / most < PALEST_NAMED {
            return None;
        }
        let span = most - least;
        let hue = if most == red {
            60.0 * ((green - blue) / span).rem_euclid(6.0)
        } else if most == green {
            60.0 * ((blue - red) / span + 2.0)
        } else {
            60.0 * ((red - green) / span + 4.0)
        };
        Some(match hue {
            h if h < 15.0 => Self::Red,
            h if h < 42.0 => Self::Orange,
            h if h < 70.0 => Self::Yellow,
            h if h < 165.0 => Self::Green,
            h if h < 260.0 => Self::Blue,
            h if h < 340.0 => Self::Violet,
            _ => Self::Red,
        })
    }
}
