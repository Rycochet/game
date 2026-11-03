//! Global weather: the snow line and water level.

use bevy::prelude::*;

/// Snow line for normal weather: above every height there is, so only the
/// temperature brings snow.
pub const NORMAL_SNOW_LINE: f64 = shared::world::NORMAL_SNOW_LINE as f64;

/// Global weather. `snow_line` is the height above which everything is snow;
/// as it falls towards 0 the water ices over (see `surface_style`).
#[derive(Resource, Debug, PartialEq)]
pub struct Weather {
    pub snow_line: f64,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            snow_line: NORMAL_SNOW_LINE,
        }
    }
}
