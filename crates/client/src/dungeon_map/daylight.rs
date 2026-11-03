//! The light from outside: what the suns and moons give the site's place on the
//! world map, on everything out in the open, and spilling in from there through
//! openings and down stairs. Worked out once a game hour.

use bevy::prelude::*;

// Split into files (`cargo xtask lint` keeps files small).
mod outside;
mod spill;
mod town_lights;

pub(super) use outside::{OutsideLight, tint_outside, token_ambient, update_outside};
pub(super) use spill::{Spill, light_from_outside};
#[cfg(test)]
use town_lights::lights_delay;
pub(super) use town_lights::{TownLights, update_town_lights};

/// How much of the light clouds take away, as `SHADOW` in `terrain.wesl`.
const CLOUD_SHADOW: f64 = 0.2;

/// Out in the open, tokens have this ambient light in full light: they are lit
/// by real lights, where tiles have it baked in.
const OPEN_AMBIENT_BRIGHTNESS: f32 = 1500.0;

/// Against the outside light, what spills from an open cell at no distance, and
/// how many cells it reaches.
const SPILL_STRENGTH: f32 = 0.5;
const SPILL_REACH: f32 = 4.0;
/// The same down stairs open to the sky, where it falls from above.
const STAIRS_REACH: f32 = 3.0;

/// How far a light low in the sky stretches spilt light along its way, and
/// shortens it the other way: at most this much either way.
const STRETCH: f32 = 1.0;
const LEAST_STRETCH: f32 = 0.5;

/// A town's buildings put their lights on once the suns and moons give flat
/// ground less than this, each up to this many game minutes after that, and off
/// as long after it comes back.
const LIGHTS_ON_BELOW: f32 = 0.3;
const LIGHTS_SPREAD_MINUTES: u64 = 60;
/// What a lit window gives, and how many cells it reaches.
pub(super) const WINDOW_LIGHT: Vec3 = Vec3::new(1.0, 0.85, 0.6);
const WINDOW_REACH: f32 = 3.0;

#[cfg(test)]
mod tests;
