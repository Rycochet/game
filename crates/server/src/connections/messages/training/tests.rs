//! Practice from moving, through a connected client.

use shared::movement::{MOVEMENT_SKILLS, RUN_PRACTICE, climb_practice};
use shared::protocol::ServerMessage;
use shared::skill::WINDOW_TICKS;

use crate::connections::tests::TestClient;

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
    assert!(test.client.practise_moving(true, 1, 0));
    assert_eq!(test.client.sheet.skills.practice(climbing), before);
    assert!(test.client.practise_moving(true, 5, WINDOW_TICKS));
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
    assert!(test.client.practise_moving(true, 0, 0));
    assert_eq!(sheets(&mut test), 1);
    // Nothing more inside the window: nothing to tell.
    assert!(test.client.practise_moving(true, 0, 1));
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
    let (stayed, cost) = test.client.wild_step(&test.world, from, to);
    assert!(stayed);
    assert!(cost > site_step_cost(from, to), "{cost} on a hillside");
    assert!(test.client.sheet.skills.practice(climbing) > before);
    // Running is the world map's alone.
    assert_eq!(test.client.sheet.skills.practice(running), ran);
}
