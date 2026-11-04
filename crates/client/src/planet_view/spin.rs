//! Each frame: the view's own clock, a day a minute from the world's hour on
//! opening, and the globe, suns and moons where it puts them.

use bevy::prelude::*;

use super::HOURS_PER_SECOND;
use super::open::PlanetView;
use crate::common::CurrentWorld;

pub(super) fn spin(
    view: Res<PlanetView>,
    world: Option<Res<CurrentWorld>>,
    time: Res<Time<Real>>,
    mut places: Query<&mut Transform>,
) {
    let (Some(showing), Some(world)) = (view.showing.as_ref(), world) else {
        return;
    };
    let hours = showing.start_hours + (time.elapsed_secs_f64() - showing.opened) * HOURS_PER_SECOND;
    let sky = &showing.sky;
    let scene = &showing.scene;
    if let Ok(mut globe) = places.get_mut(scene.globe) {
        globe.rotation = sky.globe_turn(hours);
    }
    let bodies = world.celestial.at(hours);
    for (number, &light) in scene.suns.iter().enumerate() {
        if let (Ok(mut place), Some(sun)) = (places.get_mut(light), bodies.get(number)) {
            let towards = sky.sun_direction(number, sun, hours);
            *place = Transform::default().looking_to(-towards, Vec3::Y);
        }
    }
    for (moon, number, look) in &scene.moons {
        let (Ok(mut place), Some(now)) = (places.get_mut(*moon), bodies.get(*number)) else {
            continue;
        };
        let period = world.celestial.period(*number).unwrap_or(24.0);
        place.translation = sky.moon_place(now, period);
        place.rotation = look.turn(hours);
    }
}
