use super::*;

#[test]
fn runs_with_the_server_clock_at_its_rate() {
    let time = WorldTime::at(100.0, 10.0, ClockRate::NORMAL);
    assert_eq!(time.minutes(10.0), 100.0);
    assert_eq!(time.minutes(70.0), 160.0);
    let fast = WorldTime::at(100.0, 10.0, ClockRate::new(3, 2).unwrap());
    assert_eq!(fast.minutes(30.0), 130.0);
}

#[test]
fn a_new_rate_carries_on_from_now() {
    let time = WorldTime::at(100.0, 0.0, ClockRate::NORMAL);
    let slow = time.with_rate(ClockRate::new(1, 4).unwrap(), 40.0);
    assert_eq!(slow.minutes(40.0), 140.0);
    assert_eq!(slow.minutes(80.0), 150.0);
}

#[test]
fn set_moves_it_but_not_before_the_world_began() {
    let time = WorldTime::at(100.0, 0.0, ClockRate::NORMAL);
    assert_eq!(time.set(500.0, 20.0).minutes(20.0), 500.0);
    assert_eq!(time.set(-5.0, 20.0).minutes(20.0), 0.0);
}

#[test]
fn rates_that_stop_time_are_refused() {
    assert!(ClockRate::new(0, 1).is_none());
    assert!(ClockRate::new(1, 0).is_none());
}
