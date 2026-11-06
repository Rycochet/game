use super::*;

#[test]
fn counts_while_about() {
    let mut activity = Activity::joined(1000, 50);
    assert!(activity.count(70));
    assert_eq!(activity.played, 1020);
    // Nothing new in the same tick.
    assert!(!activity.count(70));
    assert_eq!(activity.played, 1020);
}

#[test]
fn stops_counting_after_doing_nothing() {
    let mut activity = Activity::joined(0, 0);
    activity.count(ACTIVE_FOR + 500);
    assert_eq!(activity.played, ACTIVE_FOR);
    assert!(!activity.count(ACTIVE_FOR + 900));
    assert_eq!(activity.idle_for(ACTIVE_FOR + 900), ACTIVE_FOR + 900);
}

#[test]
fn doing_something_starts_it_again() {
    let mut activity = Activity::joined(0, 0);
    activity.count(ACTIVE_FOR + 500);
    activity.did_something(ACTIVE_FOR + 500);
    activity.count(ACTIVE_FOR + 600);
    // The idle stretch between isn't counted.
    assert_eq!(activity.played, ACTIVE_FOR + 100);
    assert_eq!(activity.idle_for(ACTIVE_FOR + 600), 100);
}

#[test]
fn counted_in_pieces_or_at_once_the_same() {
    let mut pieces = Activity::joined(0, 0);
    let mut once = pieces;
    for tick in (0..=ACTIVE_FOR * 2).step_by(7) {
        pieces.count(tick);
    }
    once.count(ACTIVE_FOR * 2);
    assert_eq!(pieces.played, once.played);
}

#[test]
fn what_shows_activity() {
    assert!(shows_activity(&ClientMessage::UseExit));
    assert!(!shows_activity(&ClientMessage::Ping { sent: 0.0 }));
    assert!(!shows_activity(&ClientMessage::LookUpItems(Vec::new())));
}
