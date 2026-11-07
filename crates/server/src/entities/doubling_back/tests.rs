use super::Pacing;

const GOAL: (i64, i64) = (9, 9);
const A: (i64, i64) = (0, 0);
const B: (i64, i64) = (0, 1);

/// One turn making for `goal`, stepping from `from` to `to` if allowed. Whether it
/// stepped.
fn turn(pacing: &mut Pacing, goal: (i64, i64), from: (i64, i64), to: (i64, i64)) -> bool {
    pacing.aim_at(goal);
    let allowed = pacing.allows(to);
    pacing.turn_ended(from, if allowed { to } else { from });
    allowed
}

#[test]
fn a_mob_turns_back_once_then_stands() {
    let mut pacing = Pacing::default();
    assert!(turn(&mut pacing, GOAL, A, B));
    assert!(
        turn(&mut pacing, GOAL, B, A),
        "turning back once is allowed"
    );
    assert!(!turn(&mut pacing, GOAL, A, B), "but not twice");
    assert!(!turn(&mut pacing, GOAL, A, B), "and it keeps standing");
    // Anywhere but back is still fine.
    assert!(turn(&mut pacing, GOAL, A, (1, 0)));
}

#[test]
fn a_new_goal_lets_it_move_again() {
    let mut pacing = Pacing::default();
    turn(&mut pacing, GOAL, A, B);
    turn(&mut pacing, GOAL, B, A);
    assert!(turn(&mut pacing, (5, 5), A, B));
}

#[test]
fn a_turn_spent_otherwise_forgets_the_goal() {
    let mut pacing = Pacing::default();
    turn(&mut pacing, GOAL, A, B);
    turn(&mut pacing, GOAL, B, A);
    // Wandering, fleeing or fighting: not making for anywhere.
    pacing.turn_ended(A, A);
    assert!(turn(&mut pacing, GOAL, A, B));
}

#[test]
fn going_on_and_on_is_never_pacing() {
    let mut pacing = Pacing::default();
    for y in 0..10 {
        assert!(turn(&mut pacing, GOAL, (0, y), (0, y + 1)));
    }
}
