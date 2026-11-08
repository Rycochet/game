//! Practice from moving, through a connected client.

use shared::movement::{MOVEMENT_SKILLS, RUN_PRACTICE, climb_practice};
use shared::protocol::ServerMessage;
use shared::skill::WINDOW_TICKS;

use super::Moved;
use crate::connections::tests::TestClient;

fn moved(on_world_map: bool, elevations: u32) -> Moved {
    Moved {
        on_world_map,
        elevations,
        swam: 0,
    }
}

#[test]
fn world_map_steps_practise_running() {
    let mut test = TestClient::new();
    let running = MOVEMENT_SKILLS.running;
    let before = test.client.sheet.skills.practice(running);
    let from = test.client.world_position();
    test.steps(&[(1, 0), (1, 0), (1, 0)]);
    assert_ne!(test.client.world_position(), from, "it walked");
    // Once in the window, however many steps.
    assert_eq!(
        test.client.sheet.skills.practice(running),
        before + RUN_PRACTICE
    );
}

#[test]
fn only_steep_steps_practise_climbing() {
    let mut test = TestClient::new();
    let climbing = MOVEMENT_SKILLS.climbing;
    let before = test.client.sheet.skills.practice(climbing);
    assert!(test.client.practise_moving(moved(true, 1), 0));
    assert_eq!(test.client.sheet.skills.practice(climbing), before);
    assert!(test.client.practise_moving(moved(true, 5), WINDOW_TICKS));
    assert_eq!(
        test.client.sheet.skills.practice(climbing),
        before + climb_practice(5)
    );
}

#[test]
fn the_sheet_is_sent_only_when_moving_changes() {
    let mut test = TestClient::new();
    let sheets = |test: &mut TestClient| {
        let mut count = 0;
        while let Some(message) = test.end.receive().unwrap() {
            count += usize::from(matches!(message, ServerMessage::Sheet(_)));
        }
        count
    };
    sheets(&mut test);
    // Running's value grows from nothing, so the client is told.
    assert!(test.client.practise_moving(moved(true, 0), 0));
    assert_eq!(sheets(&mut test), 1);
    // Nothing more inside the window: nothing to tell.
    assert!(test.client.practise_moving(moved(true, 0), 1));
    assert_eq!(sheets(&mut test), 0);
}

#[test]
fn wilderness_slopes_cost_more_and_practise_climbing() {
    use shared::energy::site_step_cost;
    let mut test = TestClient::new();
    let conditions = test.world.conditions();
    let ground = &test.world.ground;
    // A dry point rising steeply to the east.
    let hillside = (-1500..1500)
        .step_by(8)
        .flat_map(|y| (-1500..1500).step_by(8).map(move |x| (x, y)))
        .find(|&(x, y)| {
            let here = ground.elevation(x, y, conditions);
            let east = ground.elevation(x + 1, y, conditions);
            matches!((here, east), (Some(a), Some(b)) if b - a >= 4)
        })
        .expect("a hillside");
    let from = (hillside.0 * 16 + 8, hillside.1 * 16 + 8);
    let to = (from.0 + 1, from.1);
    let climbing = MOVEMENT_SKILLS.climbing;
    let before = test.client.sheet.skills.practice(climbing);
    let running = MOVEMENT_SKILLS.running;
    let ran = test.client.sheet.skills.practice(running);
    let (stayed, cost) = test.client.level_step(&test.world, (from, to), true, false);
    assert!(stayed);
    assert!(cost > site_step_cost(from, to), "{cost} on a hillside");
    assert!(test.client.sheet.skills.practice(climbing) > before);
    // Running is the world map's alone.
    assert_eq!(test.client.sheet.skills.practice(running), ran);
}

#[test]
fn heavy_loads_practise_weight_training_on_any_map() {
    use shared::sheet::Load;
    let mut test = TestClient::new();
    let node = MOVEMENT_SKILLS.weight_training;
    test.client.sheet.attributes.strength = u8::MAX.into();
    assert_eq!(test.client.sheet.load(&test.client.inventory), Load::Light);
    assert!(test.client.practise_moving(moved(false, 0), 0));
    assert_eq!(test.client.sheet.skills.practice(node), 0, "light: nothing");
    // Too weak for even their clothes.
    test.client.sheet.attributes.strength = 1;
    assert_ne!(test.client.sheet.load(&test.client.inventory), Load::Light);
    assert!(test.client.practise_moving(moved(false, 0), WINDOW_TICKS));
    assert!(test.client.sheet.skills.practice(node) > 0);
}

#[test]
fn wading_is_slower_and_practises_swimming() {
    use shared::energy::site_step_cost;
    let mut test = TestClient::new();
    let swimming = MOVEMENT_SKILLS.swimming;
    let before = test.client.sheet.skills.practice(swimming);
    let (from, to) = ((0, 0), (1, 0));
    let gait = shared::movement::Gait::of(&test.client.sheet.skills);
    let (stayed, cost) = test.client.level_step(&test.world, (from, to), false, true);
    assert!(stayed);
    // Between an untrained swimmer's 1.7 times as long and a record's 0.71.
    assert_eq!(cost, gait.swim(70) * 15);
    assert!(
        cost > site_step_cost(from, to),
        "this one is no great swimmer"
    );
    assert!(test.client.sheet.skills.practice(swimming) > before);
}

#[test]
fn world_map_rivers_are_waded() {
    let mut test = TestClient::new();
    let world = &mut test.world;
    // A river point some way from any road.
    let river = (0..world.rivers.columns())
        .find_map(|x| world.rivers.in_chunk(shared::poi::ChunkPos { x, y: 2 }))
        .expect("a river");
    let to = river.points[river.points.len() / 2];
    let from = (to.0 - 1, to.1);
    let ends = (Some(8), Some(8));
    let swimming = MOVEMENT_SKILLS.swimming;
    let before = test.client.sheet.skills.practice(swimming);
    let gait = shared::movement::Gait::of(&test.client.sheet.skills);
    let (stayed, cost) = test.client.world_step(&mut test.world, (from, to), ends);
    assert!(stayed);
    if test.world.road_steps.kind_of(from, to).is_none() {
        // Waded, and not run.
        assert_eq!(cost, gait.swim(70));
        assert!(test.client.sheet.skills.practice(swimming) > before);
    }
}

#[test]
fn the_sea_is_swum() {
    let mut test = TestClient::new();
    let gait = shared::movement::Gait::of(&test.client.sheet.skills);
    let (from, to) = ((0, 0), (1, 0));
    let (stayed, cost) = test
        .client
        .world_step(&mut test.world, (from, to), (Some(8), None));
    assert!(stayed);
    assert_eq!(cost, gait.swim(70), "swum, and not run");
}
