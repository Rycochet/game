use bevy::prelude::*;

use super::{PlayerMoving, note_moving};
use crate::player::{Player, PlayerEnergy, Travel};

#[test]
fn moving_while_a_journey_has_way_to_go_or_a_key_is_held() {
    let mut app = App::new();
    app.init_resource::<PlayerEnergy>()
        .init_resource::<PlayerMoving>()
        .add_systems(Update, note_moving);
    let player = app.world_mut().spawn((Player, Travel::default())).id();
    app.update();
    assert!(!app.world().resource::<PlayerMoving>().is_moving());

    let mut travel = app.world_mut().get_mut::<Travel>(player).unwrap();
    travel.route.push_back((1, 0));
    app.update();
    assert!(app.world().resource::<PlayerMoving>().is_moving());

    // Arrived; a key still held keeps it moving.
    let mut travel = app.world_mut().get_mut::<Travel>(player).unwrap();
    travel.route.clear();
    app.world_mut().resource_mut::<PlayerMoving>().keys = true;
    app.update();
    assert!(app.world().resource::<PlayerMoving>().is_moving());
    app.world_mut().resource_mut::<PlayerMoving>().keys = false;
    assert!(!app.world().resource::<PlayerMoving>().is_moving());
}
