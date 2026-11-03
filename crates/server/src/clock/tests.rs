use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn the_clock_runs_and_can_be_moved_either_way() {
    let clock = WorldClock::new(100.0);
    let other = clock.clone();
    thread::sleep(Duration::from_millis(20));
    assert!(clock.minutes() > 100.0 && clock.minutes() < 110.0);
    other.set(30.0);
    assert!(clock.minutes() >= 30.0 && clock.minutes() < 40.0);
    other.set(-5.0);
    assert!(clock.minutes() >= 0.0 && clock.minutes() < 10.0);
}

#[test]
fn a_paused_clock_stands_still_and_carries_on_from_there() {
    let clock = WorldClock::new(100.0);
    clock.pause();
    let stopped = clock.minutes();
    thread::sleep(Duration::from_millis(20));
    assert_eq!(clock.minutes(), stopped);
    // Twice makes no difference, and being moved doesn't start it.
    clock.pause();
    assert_eq!(clock.minutes(), stopped);
    clock.set(200.0);
    assert_eq!(clock.minutes(), 200.0);

    clock.resume();
    assert!(clock.minutes() < 200.0 + 1.0);
    thread::sleep(Duration::from_millis(20));
    assert!(clock.minutes() > 200.0);
}

#[test]
fn it_repeats_and_runs_once_more_on_stopping() {
    let count = Arc::new(AtomicU32::new(0));
    let counted = count.clone();
    let repeating = Repeating::start(Duration::from_millis(5), move || {
        counted.fetch_add(1, Ordering::Relaxed);
    });
    thread::sleep(Duration::from_millis(50));
    let so_far = count.load(Ordering::Relaxed);
    assert!(so_far >= 1);
    drop(repeating);
    assert!(count.load(Ordering::Relaxed) > so_far);
}
