//! The Down and Up keys on stairs, and no changing floors while fighting.

use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::site::{Exit, StairsDirection};
use shared::vitals::CALM_AFTER;

use crate::connections::tests::TestClient;

/// In a castle on this floor (it has floors above).
fn castle(level: u32) -> TestClient {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 4,
        level,
        ruin: 0,
    }]);
    test
}

/// Puts the player on the first stairs going this way: where they lead.
fn on_stairs(test: &mut TestClient, direction: StairsDirection) -> Exit {
    let visit = test.client.visit.as_mut().unwrap();
    let stairs = *visit
        .level
        .stairs
        .iter()
        .find(|s| s.direction == direction)
        .expect("such stairs");
    visit.position = (stairs.x, stairs.y);
    stairs.to
}

/// Sends `message`: the level it took the player to, if any.
fn took(test: &mut TestClient, message: ClientMessage) -> Option<u32> {
    let answers = test.send(&[message]);
    answers.iter().find_map(|answer| match answer {
        ServerMessage::EnterLevel(info) => Some(info.index),
        _ => None,
    })
}

#[test]
fn up_takes_stairs_up_and_down_doesnt() {
    let mut test = castle(0);
    let to = on_stairs(&mut test, StairsDirection::Up);
    assert_eq!(took(&mut test, ClientMessage::Down), None);
    let Exit::Level(index) = to else {
        panic!("stairs up lead to {to:?}");
    };
    assert_eq!(took(&mut test, ClientMessage::Up), Some(index));
}

#[test]
fn down_takes_stairs_down_and_up_doesnt() {
    let mut test = castle(1);
    on_stairs(&mut test, StairsDirection::Down);
    assert_eq!(took(&mut test, ClientMessage::Up), None);
    assert!(test.client.visit.is_some(), "Up didn't leave from upstairs");
    assert_eq!(took(&mut test, ClientMessage::Down), Some(0));
}

#[test]
fn no_changing_floors_while_fighting() {
    let mut test = castle(0);
    on_stairs(&mut test, StairsDirection::Up);
    let tick = test.world.action_clock.ticks();
    let limits = test.client.limits(tick);
    test.client.sheet.vitals.in_combat(limits, tick);
    assert_eq!(took(&mut test, ClientMessage::Up), None);
    assert_eq!(took(&mut test, ClientMessage::UseExit), None);
    test.world.action_clock.advance(CALM_AFTER);
    assert!(took(&mut test, ClientMessage::Up).is_some());
}
