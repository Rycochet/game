use super::*;
use crate::skill::Skills;

fn skilled(climbing: u32, running: u32) -> Gait {
    Gait {
        climbing,
        running,
        swimming: 0,
    }
}

#[test]
fn swimming_goes_from_an_average_swimmer_to_a_record() {
    let swimmer = |swimming| Gait {
        swimming,
        ..Gait::UNSKILLED
    };
    // 50 m a minute against walking's 85: 1.7 times as long.
    assert_eq!(swimmer(0).swim(1000), 1700);
    // 120 m a minute: quicker than walking.
    assert_eq!(swimmer(FULL).swim(1000), 708);
    assert_eq!(swimmer(FULL / 2).swim_per_mille(), 1204);
}

#[test]
fn unskilled_changes_nothing() {
    assert_eq!(Gait::UNSKILLED.climb(450), 450);
    assert_eq!(Gait::UNSKILLED.run(70), 70);
    assert_eq!(Gait::of(&Skills::default()), Gait::UNSKILLED);
}

#[test]
fn full_climbing_leaves_a_tenth_of_the_slope() {
    assert_eq!(skilled(FULL, 0).climb(450), 45);
    assert_eq!(skilled(FULL / 2, 0).climb(450), 247);
}

#[test]
fn full_running_is_a_marathon_pace() {
    // 4.1 times as fast: 410 becomes 100.
    assert_eq!(skilled(0, FULL).run(410), 100);
    assert_eq!(skilled(0, FULL).run(1), 1);
}

#[test]
fn values_past_full_go_no_further() {
    let mut skills = Skills::default();
    // 120%: 1.2 cubed is 1.728.
    skills.practise(MOVEMENT_SKILLS.climbing, 1_728_000);
    assert_eq!(Gait::of(&skills).climbing, FULL);
}
