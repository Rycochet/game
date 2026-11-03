use super::*;

const MS: Duration = Duration::from_millis(1);

#[test]
fn answers_are_timed_from_their_own_ping() {
    let start = Instant::now();
    let mut pings = Pings::new();
    let first = pings.next(start);
    let second = pings.next(start + 100 * MS);
    assert_ne!(first, second);

    assert_eq!(pings.answered(&first, start + 40 * MS), Some(40 * MS));
    assert_eq!(pings.waiting_since(), Some(start + 100 * MS));
    assert_eq!(pings.answered(&second, start + 150 * MS), Some(50 * MS));
    assert_eq!(pings.waiting_since(), None);
}

#[test]
fn made_up_or_repeated_answers_count_for_nothing() {
    let start = Instant::now();
    let mut pings = Pings::new();
    let ping = pings.next(start);
    assert_eq!(pings.answered(&[], start), None);
    assert_eq!(pings.answered(&[1, 2, 3], start), None);
    assert_eq!(pings.answered(&0u64.to_le_bytes(), start), None);
    assert!(pings.answered(&ping, start).is_some());
    assert_eq!(pings.answered(&ping, start), None);
}

#[test]
fn answering_the_latest_forgets_the_earlier_ones() {
    let start = Instant::now();
    let mut pings = Pings::new();
    let first = pings.next(start);
    let second = pings.next(start);
    assert!(pings.answered(&second, start).is_some());
    assert_eq!(pings.answered(&first, start), None);
}

#[test]
fn unanswered_pings_are_not_remembered_for_ever() {
    let start = Instant::now();
    let mut pings = Pings::new();
    let first = pings.next(start);
    for _ in 0..MAX_WAITING {
        pings.next(start + MS);
    }
    assert_eq!(pings.waiting.len(), MAX_WAITING);
    assert_eq!(pings.answered(&first, start), None);
}

#[test]
fn a_steady_connection_is_no_trouble() {
    let mut latency = Latency::default();
    for _ in 0..100 {
        assert_eq!(latency.measured(50 * MS), None);
    }
    assert!((latency.smoothed_ms().unwrap() - 50.0).abs() < 0.001);
    assert_eq!(latency.jitter_ms(), 0.0);
}

#[test]
fn trouble_is_reported_once_each_way() {
    let mut latency = Latency::default();
    let mut changes = Vec::new();
    for ms in std::iter::repeat_n(50, 5)
        .chain(std::iter::repeat_n(2000, 20))
        .chain(std::iter::repeat_n(50, 60))
    {
        changes.extend(latency.measured(ms * MS));
    }
    assert_eq!(changes, [true, false]);
}

#[test]
fn varying_round_trips_are_trouble_too() {
    let mut latency = Latency::default();
    let mut changes = Vec::new();
    for i in 0..40 {
        changes.extend(latency.measured(if i % 2 == 0 { 20 } else { 380 } * MS));
    }
    assert_eq!(changes, [true]);
}

#[test]
fn no_answer_is_trouble_until_answers_are_good_again() {
    let mut latency = Latency::default();
    latency.measured(50 * MS);
    assert_eq!(latency.no_answer(), Some(true));
    assert_eq!(latency.no_answer(), None);
    assert_eq!(latency.measured(50 * MS), Some(false));
}
