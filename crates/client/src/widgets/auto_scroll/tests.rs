use super::*;

/// Runs it for `seconds`, a tenth of a second at a time.
fn run(mut auto: AutoScroll, mut position: f32, furthest: f32, seconds: f32) -> (AutoScroll, f32) {
    for _ in 0..(seconds * 10.0).round() as u32 {
        (auto, position) = auto.after(0.1, position, furthest);
    }
    (auto, position)
}

#[test]
fn waits_at_the_top_before_going_down() {
    let (auto, at) = run(AutoScroll::default(), 0.0, 100.0, WAIT_AT_TOP - 0.5);
    assert_eq!((auto.phase, at), (Phase::WaitingAtTop, 0.0));
    let (auto, at) = run(auto, at, 100.0, 1.5);
    assert_eq!(auto.phase, Phase::Down);
    assert!(at > 0.0);
}

#[test]
fn stops_at_the_bottom_then_comes_back_up() {
    let down = AutoScroll {
        phase: Phase::Down,
        waited: 0.0,
    };
    let (auto, at) = run(down, 0.0, 30.0, 1.5);
    assert_eq!((auto.phase, at), (Phase::WaitingAtBottom, 30.0));
    let (auto, at) = run(auto, at, 30.0, WAIT_AT_BOTTOM - 0.5);
    assert_eq!((auto.phase, at), (Phase::WaitingAtBottom, 30.0));
    let (auto, at) = run(auto, at, 30.0, 2.0);
    assert_eq!((auto.phase, at), (Phase::WaitingAtTop, 0.0));
}

#[test]
fn does_nothing_while_everything_fits() {
    let (auto, at) = run(AutoScroll::default(), 0.0, 0.0, 10.0);
    assert_eq!((auto, at), (AutoScroll::default(), 0.0));
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    IntoSystem::into_system(auto_scroll).initialize(&mut World::new());
}
