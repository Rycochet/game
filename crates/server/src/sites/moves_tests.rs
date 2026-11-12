//! Moves on levels in units (free positions, PR 3).

use shared::protocol::{ClientMessage, ServerMessage};
use shared::spot::Spot;

use crate::connections::tests::TestClient;

/// In a castle, at the middle of a cell with room to walk two cells east.
fn with_room() -> (TestClient, (i64, i64)) {
    let mut test = TestClient::new();
    test.enter_a_castle();
    let cell = test.client.visit.as_ref().unwrap().room_to_walk_east(2);
    test.send(&[ClientMessage::DevTeleport {
        x: cell.0,
        y: cell.1,
    }]);
    (test, cell)
}

fn moved(test: &mut TestClient, number: u32, to: Spot) -> Vec<ServerMessage> {
    let map = test.client_map();
    test.send_exact(&[ClientMessage::Move { map, number, to }])
}

fn told(answers: &[ServerMessage]) -> Option<Spot> {
    answers.iter().rev().find_map(|answer| match answer {
        &ServerMessage::Position { x, y, .. } => Some(Spot::new(x, y)),
        _ => None,
    })
}

#[test]
fn a_move_can_end_anywhere_with_room() {
    let (mut test, cell) = with_room();
    let to = Spot::middle_of(cell).offset(100, 0);
    let answers = moved(&mut test, 1, to);
    assert_eq!(told(&answers), Some(to));
    assert_eq!(test.client.visit.as_ref().unwrap().spot(), to);
    // Saved as it is, not as the cell.
    assert_eq!(test.client.visit.as_ref().unwrap().save().position, to);
    // On to the next cell's middle, still the same cell rules beyond.
    let next = Spot::middle_of((cell.0 + 1, cell.1));
    assert_eq!(told(&moved(&mut test, 2, next)), Some(next));
    assert_eq!(
        test.client.visit.as_ref().unwrap().position(),
        (cell.0 + 1, cell.1)
    );
}

#[test]
fn moves_too_long_or_into_walls_are_refused() {
    let (mut test, cell) = with_room();
    let start = Spot::middle_of(cell);
    // Three cells in one go is more than the longest jump.
    let far = Spot::middle_of((cell.0 + 3, cell.1));
    assert_eq!(told(&moved(&mut test, 1, far)), Some(start));
    // Nowhere: a move to where the player already is does nothing.
    assert_eq!(told(&moved(&mut test, 2, start)), Some(start));
    let level = test.client.visit.as_ref().unwrap().cells().clone();
    let (min_x, min_y, _, _) = level.bounds();
    let wall = Spot::middle_of((min_x - 1, min_y - 1));
    assert_eq!(told(&moved(&mut test, 3, wall)), Some(start));
}
