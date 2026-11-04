use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use shared::world::WorldSeed;

use super::*;

fn after_a_second(paused: bool) -> u64 {
    let mut app = World::new();
    let worlds = ServerWorlds::with_home(ServerWorld::new(WorldSeed::new(1)));
    if paused {
        worlds.clock().pause();
    }
    app.insert_resource(worlds);
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs(1));
    app.insert_resource(time);
    app.run_system_once(advance_action_clock).unwrap();
    app.resource::<ServerWorlds>().action_clock().ticks()
}

#[test]
fn it_goes_by_real_time_and_stops_when_paused() {
    assert_eq!(after_a_second(false), TICKS_PER_SECOND as u64);
    assert_eq!(after_a_second(true), 0);
}
