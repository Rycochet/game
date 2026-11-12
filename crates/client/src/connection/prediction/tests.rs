use super::*;

fn east() -> Step {
    Step::new(1, 0).unwrap()
}

#[test]
fn confirmed_steps_are_forgotten_and_later_ones_added() {
    let mut prediction = Prediction::default();
    for _ in 0..3 {
        prediction.step(east(), (1, 0));
    }
    // The server has dealt with the first two: the third is on top.
    assert_eq!(prediction.confirmed(0, 2, (10, 0)), Some((11, 0)));
    assert_eq!(prediction.confirmed(0, 3, (11, 0)), Some((11, 0)));
}

#[test]
fn a_jump_counts_as_the_two_cells_it_moved() {
    let mut prediction = Prediction::default();
    prediction.step(east(), (1, 0));
    prediction.step(east(), (2, 0));
    assert_eq!(prediction.confirmed(0, 1, (1, 0)), Some((3, 0)));
}

#[test]
fn a_refused_step_moves_the_player_back() {
    let mut prediction = Prediction::default();
    prediction.step(east(), (1, 0));
    prediction.step(east(), (1, 0));
    // The first was refused: the server still has the player at 0.
    assert_eq!(prediction.confirmed(0, 1, (0, 0)), Some((1, 0)));
    assert_eq!(correction((2, 0), (1, 0), None), Some((1, 0)));
}

#[test]
fn positions_from_a_left_map_are_ignored() {
    let mut prediction = Prediction::default();
    prediction.step(east(), (1, 0));
    prediction.changed_map(false);
    assert_eq!(prediction.confirmed(0, 1, (5, 5)), None);
    assert_eq!(prediction.confirmed(1, 1, (5, 5)), Some((5, 5)));
    let ClientMessage::Step { map, number, .. } = prediction.step(east(), (1, 0)) else {
        unreachable!()
    };
    assert_eq!((map, number), (1, 2));
}

#[test]
fn the_world_map_wraps() {
    // The same place a planet round: nothing to correct.
    assert_eq!(correction((1005, 3), (5, 3), Some(1000)), None);
    // Elsewhere: corrected on the player's copy of the planet.
    assert_eq!(correction((1005, 3), (4, 3), Some(1000)), Some((1004, 3)));
    assert_eq!(correction((5, 3), (995, 3), Some(1000)), Some((-5, 3)));
}

#[test]
fn on_a_level_moves_go_in_units_and_the_last_sent_is_expected() {
    let mut prediction = Prediction::default();
    prediction.changed_map(true);
    let (from, to) = (Spot::new(0, 0), Spot::new(130, -40));
    assert_eq!(
        prediction.moved(from, to),
        ClientMessage::Move {
            map: 1,
            number: 1,
            to
        }
    );
    prediction.moved(to, Spot::new(200, -40));
    // The first taken: the second on top, where the last move sent ends,
    // whatever is shown along a stretch not sent yet.
    assert_eq!(prediction.confirmed(1, 1, (130, -40)), Some((200, -40)));
    assert_eq!(prediction.expected((260, -40)), (200, -40));
    // A new map: shown is expected until a move is sent there.
    prediction.changed_map(true);
    assert_eq!(prediction.expected((7, 7)), (7, 7));
}
