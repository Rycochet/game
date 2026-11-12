use super::*;

#[test]
fn only_a_slow_pace_counts_as_sneaking() {
    let mut sneak = Sneak::default();
    let every = u64::from(SNEAK_TICKS_PER_CELL);
    // A first step after standing still: nothing to say it was quick.
    assert!(sneak.stepped(0, 1));
    assert!(sneak.stepped(every, 1));
    assert!(sneak.stepped(2 * every, 1));
    // Walking: four cells a second.
    assert!(!sneak.stepped(2 * every + 5, 1));
    assert!(!sneak.stepped(2 * every + 10, 1));
    // A minute later the quick steps are forgotten.
    assert!(sneak.stepped(2 * every + 10 + PACE_TICKS + 1, 1));
}

#[test]
fn toggling_tells_the_client() {
    let mut test = crate::connections::tests::TestClient::new();
    while test.end.receive().unwrap().is_some() {}
    assert!(test.client.toggle_sneak());
    assert!(test.client.learning.sneak.on);
    assert!(matches!(
        test.end.receive().unwrap(),
        Some(ServerMessage::Sneaking(true))
    ));
    assert_eq!(test.client.sneak_cost(70, 240), test.client.speed * 40);
    assert!(test.client.toggle_sneak());
    assert_eq!(test.client.sneak_cost(70, 240), 70);
}
