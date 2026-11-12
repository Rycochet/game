use shared::hall::{GatePlace, HALL_LENGTH, Side, WALKWAY_ROWS};
use shared::protocol::{ClientMessage, ServerMessage, cell_of_units};
use shared::spot::Spot;
use shared::world::WorldSeed;

use crate::connections::tests::TestClient;

/// A client sent to the Hall, and where it arrived, as the client counts
/// it, and where that counts from.
fn in_the_hall() -> (TestClient, (i64, i64), i64) {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevToHall {
        seed: None,
        gate: None,
    }]);
    let at = answers
        .iter()
        .find_map(|message| match message {
            ServerMessage::EnterHall { at, origin } => Some((at.cell(), *origin)),
            _ => None,
        })
        .expect("in the Hall");
    (test, at.0, at.1)
}

fn position(answers: &[ServerMessage]) -> Option<(i64, i64)> {
    answers.iter().rev().find_map(|message| match message {
        ServerMessage::Position { x, y, .. } => Some(cell_of_units((*x, *y))),
        _ => None,
    })
}

#[test]
fn dev_tools_can_choose_another_worlds_gate() {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevToHall {
        seed: Some(12345),
        gate: Some(3),
    }]);
    let (x, y, origin) = answers
        .iter()
        .find_map(|message| match message {
            ServerMessage::EnterHall { at, origin } => Some((at.cell().0, at.cell().1, *origin)),
            _ => None,
        })
        .expect("in the Hall");
    let wanted = shared::hall::GateAddress::for_world(WorldSeed::new(12345), 3)
        .expect("a world of the Hall")
        .place();
    assert_eq!((x + origin, y), wanted.front());
}

#[test]
fn dev_tools_put_the_player_before_this_worlds_gate() {
    let (_, (x, y), origin) = in_the_hall();
    assert_eq!(origin % 4, 0);
    assert!(x.abs() < 4, "the client counts from near where it is");
    let gate_y = if y == 0 { -1 } else { WALKWAY_ROWS };
    let gate = GatePlace::at(x + origin, gate_y).expect("in front of a gate");
    assert_eq!(gate.front(), (x + origin, y));
    assert_eq!(gate.address().world(), WorldSeed::new(1));
}

#[test]
fn steps_stay_on_the_floor() {
    let (mut test, (x, y), _) = in_the_hall();
    let along = if y == 0 { 1 } else { -1 };
    // Along the walkway, then off it beside the gate into the void.
    assert_eq!(position(&test.steps(&[(1, 0)])), Some((x + 1, y)));
    let refused = test.steps(&[(0, -along)]);
    assert_eq!(position(&refused), Some((x + 1, y)));
    // Back, and onto the gate's floor.
    assert_eq!(position(&test.steps(&[(-1, -along)])), Some((x, y - along)));
}

#[test]
fn walking_round_the_seam_carries_on() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevToHall {
        seed: None,
        gate: None,
    }]);
    test.client.character.hall = Some(Spot::middle_of((HALL_LENGTH - 1, 1)));
    test.steps(&[(1, 0)]);
    assert_eq!(
        test.client.character.hall,
        Some(Spot::middle_of((HALL_LENGTH, 1)))
    );
    // Coming back, it is the first tile again.
    let far = Spot::middle_of((HALL_LENGTH + 2, 1));
    let at = super::enter(&mut test.client, &mut test.world, far);
    assert!(at);
    assert_eq!(test.client.character.hall, Some(Spot::middle_of((2, 1))));
}

#[test]
fn teleports_land_only_on_the_floor() {
    let (mut test, _, _) = in_the_hall();
    let answers = test.send(&[ClientMessage::DevTeleport { x: 1, y: -1 }]);
    assert!(
        !answers
            .iter()
            .any(|m| matches!(m, ServerMessage::Teleported { .. }))
    );
    let answers = test.send(&[ClientMessage::DevTeleport { x: 8, y: -1 }]);
    assert!(answers.contains(&ServerMessage::Teleported {
        x: 8 * 240,
        y: -240
    }));
    // Anywhere on the floor, not only a tile's middle.
    let answers = test.send_exact(&[ClientMessage::DevTeleport {
        x: 8 * 240 + 50,
        y: -200,
    }]);
    assert!(answers.contains(&ServerMessage::Teleported {
        x: 8 * 240 + 50,
        y: -200
    }));
    assert_eq!(
        GatePlace::at(8, -1).map(|gate| gate.side),
        Some(Side::North)
    );
}

#[test]
fn back_to_start_leaves_the_hall() {
    let (mut test, _, _) = in_the_hall();
    let answers = test.send(&[ClientMessage::DevBackToStart]);
    assert!(
        answers
            .iter()
            .any(|m| matches!(m, ServerMessage::OnWorldMap { .. }))
    );
    assert_eq!(test.client.character.hall, None);
}
