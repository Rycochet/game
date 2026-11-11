use super::*;

#[test]
fn the_box_is_good_markup() {
    SPLIT.assert_good();
}

#[test]
fn opening_takes_half_and_never_none() {
    let stack = Id::for_test(1);
    assert_eq!(Splitting::open(stack, 10, Vec2::ZERO).count, 5);
    assert_eq!(Splitting::open(stack, 1, Vec2::ZERO).count, 1);
}
