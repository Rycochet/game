use super::*;

#[test]
fn floor_rounds_down_on_both_sides_of_zero() {
    assert_eq!(floor(2.7), 2);
    assert_eq!(floor(2.0), 2);
    assert_eq!(floor(-0.5), -1);
    assert_eq!(floor(-3.0), -3);
    assert_eq!(floor(-3.2), -4);
}
