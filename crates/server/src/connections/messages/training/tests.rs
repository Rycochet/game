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
    assert!(test.client.practise_moving(1, 0));
    assert_eq!(test.client.sheet.skills.practice(climbing), before);
    assert!(test.client.practise_moving(5, WINDOW_TICKS));
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
    assert!(test.client.practise_moving(0, 0));
    assert_eq!(sheets(&mut test), 1);
    // Nothing more inside the window: nothing to tell.
    assert!(test.client.practise_moving(0, 1));
    assert_eq!(sheets(&mut test), 0);
}
