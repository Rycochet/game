use super::*;

#[test]
fn the_slider_runs_from_one_to_all() {
    assert_eq!(count_at(0.0, 10), 1);
    assert_eq!(count_at(1.0, 10), 10);
    assert_eq!(count_at(0.5, 3), 2);
    for count in 1..=7 {
        assert_eq!(count_at(value_of(count, 7), 7), count);
    }
}

#[test]
fn a_box_opens_on_half_of_the_stack() {
    let at = Vec2::ZERO;
    assert_eq!(Splitting::open(Id::NOBODY, 9, at).count, 4);
    assert_eq!(Splitting::open(Id::NOBODY, 2, at).count, 1);
}
