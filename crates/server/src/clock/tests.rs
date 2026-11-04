use super::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[test]
fn the_clock_runs() {
    let clock = ServerClock::new(100.0);
    let other = clock.clone();
    thread::sleep(Duration::from_millis(20));
    assert!(other.seconds() > 100.0 && other.seconds() < 110.0);
}

#[test]
fn a_paused_clock_stands_still_and_carries_on_from_there() {
    let clock = ServerClock::new(100.0);
    clock.pause();
    let stopped = clock.seconds();
    thread::sleep(Duration::from_millis(20));
    assert_eq!(clock.seconds(), stopped);
    // Twice makes no difference.
    clock.pause();
    assert_eq!(clock.seconds(), stopped);

    clock.resume();
    assert!(clock.seconds() < stopped + 1.0);
    thread::sleep(Duration::from_millis(20));
    assert!(clock.seconds() > stopped);
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
