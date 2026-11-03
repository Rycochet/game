//! Which of a town's buildings have their lights on.

use bevy::prelude::*;
use shared::celestial::flat_light;
use shared::rng::Seed;
use std::collections::BTreeSet;

use super::{LIGHTS_ON_BELOW, LIGHTS_SPREAD_MINUTES};
use crate::common::{CurrentDungeon, CurrentWorld, ServerClock, WorldReturn};

/// The buildings of the town with their lights on, by `id`. Only for looks, so
/// only the client knows; clients agree as far as their clocks do.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(in crate::dungeon_map) struct TownLights(pub BTreeSet<u16>);

/// How many minutes after the light changes a building's lights follow. From
/// the world and place, so they differ between towns.
pub(super) fn lights_delay(world: Seed, place: (i64, i64), building: u16) -> u64 {
    world.at(&[place.0, place.1, i64::from(building)]) % LIGHTS_SPREAD_MINUTES
}

/// Whether the light was low enough for lights at a time.
pub(super) fn light_is_low(world: &CurrentWorld, minutes: f64) -> bool {
    let bodies = world
        .celestial
        .at(minutes / shared::world::MINUTES_PER_HOUR);
    let flat = flat_light(&bodies);
    flat.into_iter().fold(0.0, f64::max) < f64::from(LIGHTS_ON_BELOW)
}

/// Each game minute, works out which of a town's buildings have their lights on.
pub(in crate::dungeon_map) fn update_town_lights(
    world: Option<Res<CurrentWorld>>,
    clock: Res<ServerClock>,
    place: Res<WorldReturn>,
    dungeon: Res<CurrentDungeon>,
    mut lights: ResMut<TownLights>,
    mut last_minute: Local<Option<i64>>,
) {
    let Some(world) = world else {
        return;
    };
    let minute = clock.minutes as i64;
    if *last_minute == Some(minute) && !dungeon.is_changed() && !world.is_changed() {
        return;
    }
    *last_minute = Some(minute);
    let on: BTreeSet<u16> = dungeon
        .buildings
        .iter()
        .filter(|building| {
            let delay = lights_delay(world.params.seed, (place.0.x, place.0.y), building.id);
            light_is_low(&world, minute as f64 - delay as f64)
        })
        .map(|building| building.id)
        .collect();
    if lights.0 != on {
        lights.0 = on;
    }
}
