use super::weight_words;

#[test]
fn with_a_limit_both_are_shown_with_no_space_before_the_unit() {
    assert_eq!(weight_words(12_340, Some(45_000)), "12.34kg / 45.00kg");
}

#[test]
fn with_no_limit_only_the_weight_is_shown() {
    assert_eq!(weight_words(3_200, None), "3.20kg");
}
