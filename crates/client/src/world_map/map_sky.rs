//! The sky the world map is lit by: the real one (`SkyNow`), or with Midday
//! Sun the main sun alone at noon, whatever the time, so its light never moves.
//! Sites keep the real one.

use bevy::prelude::*;
use shared::celestial::flat_light;

use crate::common::{CurrentWorld, Preferences, SkyNow};

/// Noon for the main sun, which sets the day (`shared::celestial`).
const NOON: f64 = 12.0;

/// The sky the world map is lit by this frame.
#[derive(Resource, Default)]
pub(crate) struct MapSky(pub(super) SkyNow);

/// Every frame in this view: the real sky, or noon's.
pub(super) fn update(
    world: Res<CurrentWorld>,
    sky: Res<SkyNow>,
    preferences: Res<Preferences>,
    mut map: ResMut<MapSky>,
) {
    if !preferences.video.midday_sun {
        map.0.clone_from(&sky);
        return;
    }
    let mut bodies = world.celestial.at(NOON);
    // The main sun comes first.
    bodies.truncate(1);
    map.0.flat = Vec3::from_array(flat_light(&bodies).map(|channel| channel as f32));
    map.0.bodies = bodies;
}

/// The hour the world map's slope shading is lit at.
pub(super) fn shading_hour(midday_sun: bool, hour: f64) -> f64 {
    if midday_sun { NOON } else { hour }
}
