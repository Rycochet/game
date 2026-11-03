use super::*;

#[test]
fn unset_limits_are_the_usual_ones() {
    assert_eq!(from_values(None, None), StackLimits::default());
}

#[test]
fn set_limits_are_used() {
    let limits = from_values(Some("5000"), Some(" 20 "));
    assert_eq!(
        limits,
        StackLimits {
            grams_per_cell: 5000,
            most: 20
        }
    );
}

#[test]
fn nonsense_leaves_the_usual_limit() {
    let usual = StackLimits::default();
    for value in ["0", "-3", "lots", ""] {
        assert_eq!(from_values(Some(value), Some(value)), usual, "{value}");
    }
}
