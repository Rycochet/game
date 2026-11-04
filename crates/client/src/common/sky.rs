//! The suns and moons as they are this frame.

use bevy::prelude::*;
use shared::celestial::{BodyNow, flat_light};

use super::{CurrentWorld, NIGHT_LIGHT, ServerClock};

/// The suns and moons as they are this frame (`shared::celestial`).
#[derive(Resource, Default, Clone)]
pub struct SkyNow {
    /// In the order of `Celestial::bodies`.
    pub bodies: Vec<BodyNow>,
    /// The light they give flat ground together, linear RGB.
    pub flat: Vec3,
}

impl SkyNow {
    /// How much to multiply what isn't really lit by, to match what is.
    pub fn lit(&self) -> Vec3 {
        Vec3::splat(NIGHT_LIGHT) + self.flat.min(Vec3::ONE) * (1.0 - NIGHT_LIGHT)
    }
}

pub(super) fn update_sky(
    world: Res<CurrentWorld>,
    clock: Res<ServerClock>,
    mut sky: ResMut<SkyNow>,
) {
    sky.bodies = world.celestial.at(clock.hours());
    sky.flat = Vec3::from_array(flat_light(&sky.bodies).map(|channel| channel as f32));
}
